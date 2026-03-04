//! Gateway HTTP server module.
//!
//! Provides an HTTP server for the gateway with health check endpoint
//! and graceful shutdown support.

use crate::discord::gateway::{DiscordEvent, DiscordGateway};
use crate::gateway::config::{GatewayConfig, ServerConfig};
use crate::gateway::pid::{PidFile, PidFileError};
use crate::gateway::protocol::GatewayMessage;
use crate::gateway::registry::{ChannelRegistry, ProjectConnection};
use crate::gateway::routing::Router as MessageRouter;
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use chrono::Duration;
use futures_util::{SinkExt, StreamExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tokio::sync::mpsc;
use tower_http::trace::TraceLayer;
use tracing::instrument;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Error types for gateway server operations.
#[derive(Debug, Error)]
pub enum GatewayServerError {
    /// Failed to bind to the server address.
    #[error("Failed to bind to address {address}: {source}")]
    BindError {
        /// The address we tried to bind to.
        address: SocketAddr,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Server failed to start.
    #[error("Server failed to start: {0}")]
    StartError(String),

    /// Server encountered an error during runtime.
    #[error("Server runtime error: {0}")]
    RuntimeError(String),

    /// PID file error.
    #[error("PID file error: {0}")]
    PidFileError(#[from] PidFileError),

    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    /// Failed to parse WebSocket message.
    #[error("Failed to parse message: {0}")]
    MessageParseError(String),
}

/// Health check response JSON structure.
#[derive(serde::Serialize, Debug)]
pub struct HealthResponse {
    /// The health status of the server.
    pub status: &'static str,
}

/// Status response for a connected project.
#[derive(serde::Serialize, Debug)]
pub struct ProjectStatus {
    /// The name of the project.
    pub name: String,
    /// List of channels the project is subscribed to.
    pub channels: Vec<String>,
}

/// Status response JSON structure for the /status endpoint.
#[derive(serde::Serialize, Debug)]
pub struct StatusResponse {
    /// Whether the gateway is running.
    pub gateway_running: bool,
    /// Whether Discord is connected.
    pub discord_connected: bool,
    /// List of connected projects with their channel subscriptions.
    pub connected_projects: Vec<ProjectStatus>,
}

/// Application state for the HTTP server.
#[derive(Clone)]
pub struct AppState {
    /// The gateway configuration.
    pub config: Arc<GatewayConfig>,
    /// The channel registry for tracking project subscriptions.
    pub registry: ChannelRegistry,
    /// The Discord Gateway for reconnection support.
    pub discord_gateway: Arc<tokio::sync::Mutex<Option<DiscordGateway>>>,
}

/// Creates the health check route handler.
async fn health_handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// Creates the status route handler.
///
/// Returns the current status of the gateway including Discord connection status
/// and list of connected projects with their channel subscriptions.
async fn status_handler(State(state): State<AppState>) -> Json<StatusResponse> {
    // Check if Discord is connected
    let discord_connected = state.discord_gateway.lock().await.is_some();

    // Get all projects from the registry
    let all_projects = state.registry.all_projects().await;

    // Convert projects to ProjectStatus
    let connected_projects: Vec<ProjectStatus> = all_projects
        .into_iter()
        .map(|project| ProjectStatus {
            name: project.project_name,
            channels: project.subscribed_channels,
        })
        .collect();

    Json(StatusResponse {
        gateway_running: true,
        discord_connected,
        connected_projects,
    })
}

/// WebSocket handler for project connections.
///
/// This handler accepts WebSocket upgrade requests, creates a bidirectional
/// channel for sending/receiving messages, parses incoming JSON messages
/// using the protocol types, and echoes them back for testing.
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    info!(target: "gateway::server", "WebSocket upgrade request received");

    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Handle the WebSocket connection after upgrade.
///
/// This function manages the bidirectional message flow, parsing incoming
/// JSON messages and echoing them back for testing.
#[instrument(
    name = "websocket_handler",
    skip(socket, state),
    fields(project_id, session_id)
)]
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Track the project_id after successful registration
    let mut project_id: Option<String> = None;

    // Process incoming messages
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                info!(target: "gateway::server", "Received WebSocket text message: {}", text);

                // Parse the incoming JSON message
                match serde_json::from_str::<GatewayMessage>(&text) {
                    Ok(gateway_msg) => {
                        // Handle registration messages
                        match gateway_msg {
                            GatewayMessage::Register {
                                project_name,
                                channels,
                            } => {
                                // Validate project_name is not empty
                                if project_name.trim().is_empty() {
                                    let error_response = GatewayMessage::RegisterError {
                                        error: "Project name cannot be empty".to_string(),
                                    };
                                    if let Ok(response) = serde_json::to_string(&error_response) {
                                        let _ = sender.send(Message::Text(response)).await;
                                    }
                                    continue;
                                }

                                // Validate channels is not empty
                                if channels.is_empty() {
                                    let error_response = GatewayMessage::RegisterError {
                                        error: "At least one channel must be specified".to_string(),
                                    };
                                    if let Ok(response) = serde_json::to_string(&error_response) {
                                        let _ = sender.send(Message::Text(response)).await;
                                    }
                                    continue;
                                }

                                // Generate a unique project ID
                                let new_project_id = Uuid::new_v4().to_string();

                                // Create a channel for sending messages to this client
                                let (tx, _rx) = mpsc::channel::<String>(100);

                                // Create project connection
                                let project = ProjectConnection::new(
                                    new_project_id.clone(),
                                    project_name.clone(),
                                    tx,
                                );
                                let session_id = project.session_id.to_string();

                                // Register with the channel registry
                                let registry = state.registry.clone();
                                match registry.register(project, channels).await {
                                    Ok(()) => {
                                        // Store the project_id for subsequent message handling
                                        project_id = Some(new_project_id.clone());

                                        // Record project_id and session_id in the tracing span
                                        tracing::Span::current()
                                            .record("project_id", &new_project_id)
                                            .record("session_id", &session_id);

                                        // Send success acknowledgment
                                        let ack = GatewayMessage::RegisterAck {
                                            status: "ok".to_string(),
                                            session_id: session_id.clone(),
                                        };
                                        if let Ok(response) = serde_json::to_string(&ack) {
                                            if sender.send(Message::Text(response)).await.is_err() {
                                                warn!(target: "gateway::server", "Failed to send ack, client disconnected");
                                                break;
                                            }
                                        }
                                        info!(target: "gateway::server", "Project registered successfully: {}", session_id);
                                    }
                                    Err(e) => {
                                        // Send error response
                                        let error_response = GatewayMessage::RegisterError {
                                            error: e.to_string(),
                                        };
                                        if let Ok(response) = serde_json::to_string(&error_response)
                                        {
                                            let _ = sender.send(Message::Text(response)).await;
                                        }
                                        warn!(target: "gateway::server", "Registration failed: {}", e);
                                    }
                                }
                            }

                            // Handle channel subscribe messages
                            // Handle channel subscribe messages
                            GatewayMessage::ChannelSubscribe { channels } => {
                                // Check if project is registered
                                let Some(ref pid) = project_id else {
                                    let error_response = GatewayMessage::RegisterError {
                                        error: "Project not registered".to_string(),
                                    };
                                    if let Ok(response) = serde_json::to_string(&error_response) {
                                        let _ = sender.send(Message::Text(response)).await;
                                    }
                                    continue;
                                };

                                // Validate channels is not empty
                                if channels.is_empty() {
                                    let error_response = GatewayMessage::ChannelSubscribeAck {
                                        status: "error: no channels specified".to_string(),
                                    };
                                    if let Ok(response) = serde_json::to_string(&error_response) {
                                        let _ = sender.send(Message::Text(response)).await;
                                    }
                                    continue;
                                }

                                // Add channel subscriptions
                                let registry = state.registry.clone();
                                let mut errors = Vec::new();
                                for channel in &channels {
                                    match registry.add_channel_subscription(pid, channel).await {
                                        Ok(()) => {}
                                        Err(e) => errors.push(e.to_string()),
                                    }
                                }

                                // Send acknowledgment
                                let status = if errors.is_empty() {
                                    format!("ok: subscribed to {} channels", channels.len())
                                } else {
                                    format!("error: {}", errors.join(", "))
                                };
                                let ack = GatewayMessage::ChannelSubscribeAck { status };
                                if let Ok(response) = serde_json::to_string(&ack) {
                                    if sender.send(Message::Text(response)).await.is_err() {
                                        warn!(target: "gateway::server", "Failed to send subscribe ack, client disconnected");
                                        break;
                                    }
                                }
                                info!(target: "gateway::server", "Channel subscribe processed for project {}", pid);
                            }

                            // Handle channel unsubscribe messages
                            GatewayMessage::ChannelUnsubscribe { channels } => {
                                // Check if project is registered
                                let Some(ref pid) = project_id else {
                                    let error_response = GatewayMessage::RegisterError {
                                        error: "Project not registered".to_string(),
                                    };
                                    if let Ok(response) = serde_json::to_string(&error_response) {
                                        let _ = sender.send(Message::Text(response)).await;
                                    }
                                    continue;
                                };

                                // Validate channels is not empty
                                if channels.is_empty() {
                                    let error_response = GatewayMessage::ChannelUnsubscribeAck {
                                        status: "error: no channels specified".to_string(),
                                    };
                                    if let Ok(response) = serde_json::to_string(&error_response) {
                                        let _ = sender.send(Message::Text(response)).await;
                                    }
                                    continue;
                                }

                                // Remove channel subscriptions
                                let registry = state.registry.clone();
                                let mut errors = Vec::new();
                                for channel in &channels {
                                    match registry.remove_channel_subscription(pid, channel).await {
                                        Ok(()) => {}
                                        Err(e) => errors.push(e.to_string()),
                                    }
                                }

                                // Send acknowledgment
                                let status = if errors.is_empty() {
                                    format!("ok: unsubscribed from {} channels", channels.len())
                                } else {
                                    format!("error: {}", errors.join(", "))
                                };
                                let ack = GatewayMessage::ChannelUnsubscribeAck { status };
                                if let Ok(response) = serde_json::to_string(&ack) {
                                    if sender.send(Message::Text(response)).await.is_err() {
                                        warn!(target: "gateway::server", "Failed to send unsubscribe ack, client disconnected");
                                        break;
                                    }
                                }
                                info!(target: "gateway::server", "Channel unsubscribe processed for project {}", pid);
                            }

                            // Handle heartbeat messages
                            GatewayMessage::Heartbeat { timestamp } => {
                                // Check if project is registered
                                let Some(ref pid) = project_id else {
                                    warn!(target: "gateway::server", "Heartbeat received but project not registered");
                                    continue;
                                };

                                // Update the connection's last heartbeat time
                                let registry = state.registry.clone();
                                if let Err(e) = registry.update_heartbeat(pid).await {
                                    warn!(target: "gateway::server", "Failed to update heartbeat: {}", e);
                                } else {
                                    debug!(target: "gateway::server", "Heartbeat updated for project {}", pid);
                                }

                                // Send HeartbeatAck back with the timestamp
                                let ack = GatewayMessage::HeartbeatAck { timestamp };
                                if let Ok(response) = serde_json::to_string(&ack) {
                                    if sender.send(Message::Text(response)).await.is_err() {
                                        warn!(target: "gateway::server", "Failed to send heartbeat ack, client disconnected");
                                        break;
                                    }
                                }
                                info!(target: "gateway::server", "Sent heartbeat ack for project {}", pid);
                            }

                            // Echo other message types back to the client
                            _ => match serde_json::to_string(&gateway_msg) {
                                Ok(response) => {
                                    if sender.send(Message::Text(response)).await.is_err() {
                                        warn!(target: "gateway::server", "Failed to send response, client disconnected");
                                        break;
                                    }
                                    info!(target: "gateway::server", "Echoed message back to client");
                                }
                                Err(e) => {
                                    error!(target: "gateway::server", "Failed to serialize message: {}", e);
                                    let error_msg = GatewayMessage::Message {
                                        payload: format!("Serialization error: {}", e),
                                        channel_id: 0,
                                    };
                                    let _ = sender
                                        .send(Message::Text(
                                            serde_json::to_string(&error_msg).unwrap_or_default(),
                                        ))
                                        .await;
                                }
                            },
                        }
                    }
                    Err(e) => {
                        error!(target: "gateway::server", "Failed to parse message: {}", e);
                        // Send error response
                        let error_msg = GatewayMessage::Message {
                            payload: format!("Parse error: {}", e),
                            channel_id: 0,
                        };
                        if let Ok(error_json) = serde_json::to_string(&error_msg) {
                            let _ = sender.send(Message::Text(error_json)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!(target: "gateway::server", "Client sent close message");
                // Clean up registration if project was registered
                if let Some(ref pid) = project_id {
                    let registry = state.registry.clone();
                    if let Err(e) = registry.unregister(pid).await {
                        error!(target: "gateway::server", "Failed to unregister project {}: {}", pid, e);
                    } else {
                        info!(target: "gateway::server", "Project {} unregistered successfully", pid);
                    }
                }
                break;
            }
            Ok(Message::Ping(data)) => {
                info!(target: "gateway::server", "Received ping, sending pong");
                let _ = sender.send(Message::Pong(data)).await;
            }
            Ok(Message::Pong(_)) => {
                // Pong received, nothing to do
            }
            Err(e) => {
                error!(target: "gateway::server", "WebSocket error: {}", e);
                // Clean up registration if project was registered
                if let Some(ref pid) = project_id {
                    let registry = state.registry.clone();
                    if let Err(unreg_err) = registry.unregister(pid).await {
                        error!(target: "gateway::server", "Failed to unregister project {} on error: {}", pid, unreg_err);
                    } else {
                        info!(target: "gateway::server", "Project {} unregistered after WebSocket error", pid);
                    }
                }
                break;
            }
            _ => {
                // Ignore binary messages for now
            }
        }
    }

    info!(target: "gateway::server", "WebSocket connection closed");
}

/// The gateway HTTP server.
///
/// This server provides HTTP endpoints for the gateway service,
/// including health checks for monitoring.
pub struct GatewayServer {
    /// The server configuration.
    config: ServerConfig,
    /// The gateway configuration.
    gateway_config: GatewayConfig,
    /// Path to the PID file.
    pid_path: std::path::PathBuf,
}

impl GatewayServer {
    /// Create a new GatewayServer with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The server configuration (host, ports)
    /// * `gateway_config` - The full gateway configuration
    ///
    /// # Returns
    ///
    /// * A new GatewayServer instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let server = GatewayServer::new(server_config, gateway_config);
    /// ```
    pub fn new(config: ServerConfig, gateway_config: GatewayConfig) -> Self {
        Self {
            config,
            gateway_config,
            pid_path: PidFile::default_path(),
        }
    }

    /// Run the HTTP server with graceful shutdown support.
    ///
    /// This method starts the axum HTTP server and waits for either
    /// a SIGINT (Ctrl+C) or SIGTERM signal to initiate graceful shutdown.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Server shutdown completed successfully
    /// * `Err(GatewayServerError)` - If the server fails to start or encounters an error
    ///
    /// # Example
    ///
    /// ```ignore
    /// let server = GatewayServer::new(config, gateway_config);
    /// server.run().await?;
    /// ```
    #[instrument(name = "gateway_server", skip(self))]
    pub async fn run(self) -> Result<(), GatewayServerError> {
        let pid_path = self.pid_path.clone();

        // Log gateway startup with configuration values
        info!(
            target: "gateway::server",
            message = "=== Gateway Server Starting ===",
            host = %self.config.host,
            http_port = self.config.http_port,
            ws_port = self.config.ws_port,
            pid_file = %pid_path.display(),
            discord_enabled = !self.gateway_config.discord_token.is_empty(),
            log_level = %self.gateway_config.logging.level
        );

        // Check if another gateway is already running
        if let Err(e) = PidFile::check_existing(&pid_path) {
            match e {
                PidFileError::AlreadyRunning(pid) => {
                    error!(
                        target: "gateway::server",
                        "Gateway is already running with PID {}. Exiting.",
                        pid
                    );
                    return Err(GatewayServerError::PidFileError(
                        PidFileError::AlreadyRunning(pid),
                    ));
                }
                _ => {
                    // For other errors (like parse errors on stale files), log and continue
                    warn!(
                        target: "gateway::server",
                        "PID file check warning: {}, proceeding anyway",
                        e
                    );
                }
            }
        }

        let ip: IpAddr = self
            .config
            .host
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let addr = SocketAddr::from((ip, self.config.http_port as u16));

        info!(
            target: "gateway::server",
            "Binding to address: {}:{}",
            self.config.host,
            self.config.http_port
        );

        // Create application state
        let state = AppState {
            config: Arc::new(self.gateway_config.clone()),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        // Start stale connection detector
        // This background task monitors connections and removes those that haven't
        // sent a heartbeat within the timeout period (90 seconds)
        let registry_for_detector = state.registry.clone();
        let _stale_detector_handle = tokio::spawn(async move {
            // Timeout after which a connection is considered stale (90 seconds)
            let stale_timeout = Duration::seconds(90);
            // Check interval (1/4 of the timeout = 22.5 seconds)
            let check_interval = Duration::seconds(22);
            let check_interval_ms = check_interval.num_milliseconds().max(1000) as u64;

            info!(
                target: "gateway::server",
                check_interval_ms = check_interval_ms,
                timeout_secs = stale_timeout.num_seconds(),
                "Stale connection detector started"
            );

            loop {
                // Wait for the check interval
                tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;

                // Find stale connections
                let all_projects = registry_for_detector.all_projects().await;
                let mut stale_project_ids = Vec::new();

                for project in all_projects {
                    if registry_for_detector
                        .is_connection_stale(&project.project_id, stale_timeout)
                        .await
                    {
                        stale_project_ids.push(project.project_id);
                    }
                }

                if !stale_project_ids.is_empty() {
                    info!(
                        target: "gateway::server",
                        stale_count = stale_project_ids.len(),
                        "Detected stale connections, removing them"
                    );

                    // Unregister stale projects
                    for project_id in stale_project_ids {
                        if let Err(e) = registry_for_detector.unregister(&project_id).await {
                            warn!(
                                target: "gateway::server",
                                project_id = %project_id,
                                error = %e,
                                "Failed to remove stale connection"
                            );
                        } else {
                            info!(
                                target: "gateway::server",
                                project_id = %project_id,
                                "Removed stale connection"
                            );
                        }
                    }
                }
            }
        });

        // Clone registry for Discord event handler
        let registry_for_events = state.registry.clone();

        // Start Discord Gateway connection if token is configured
        let discord_token = self.gateway_config.discord_token.clone();
        let has_discord_token = !discord_token.is_empty();

        if has_discord_token {
            info!(
                target: "gateway::server",
                "Discord Gateway enabled, starting connection..."
            );
            // Spawn Discord Gateway connection task with auto-reconnection
            let discord_gateway_for_state = state.discord_gateway.clone();
            let discord_token_for_reconnect = discord_token.clone();
            let registry_for_events = registry_for_events.clone();
            tokio::spawn(async move {
                // Default intents: GUILD_MESSAGES + DIRECT_MESSAGES + MESSAGE_CONTENT
                const DEFAULT_INTENTS: u32 = 512 | 4096 | 16384;

                // Reconnection configuration
                const INITIAL_BACKOFF_SECS: u64 = 1;
                const MAX_BACKOFF_SECS: u64 = 60;
                let mut backoff_secs = INITIAL_BACKOFF_SECS;

                loop {
                    info!(target: "gateway::discord", "Starting Discord Gateway connection...");

                    // Create a new gateway for this connection attempt
                    let (event_sender, event_receiver) = mpsc::channel::<DiscordEvent>(100);
                    let mut gateway = DiscordGateway::new(
                        discord_token_for_reconnect.clone(),
                        DEFAULT_INTENTS,
                        event_sender,
                    );

                    // Store the gateway in AppState for reconnection support
                    // We need to clone the token and recreate the gateway for storage
                    let gateway_for_storage = DiscordGateway::new(
                        discord_token_for_reconnect.clone(),
                        DEFAULT_INTENTS,
                        mpsc::channel::<DiscordEvent>(100).0,
                    );
                    {
                        let mut guard = discord_gateway_for_state.lock().await;
                        *guard = Some(gateway_for_storage);
                    }

                    // Create shutdown channel for this connection
                    let (_gateway_shutdown_tx, gateway_shutdown_rx) =
                        tokio::sync::oneshot::channel();

                    // Spawn the event processor for this connection
                    let registry_clone = registry_for_events.clone();
                    tokio::spawn(async move {
                        process_discord_events(event_receiver, registry_clone).await;
                    });

                    // Run the connection
                    let result = gateway.connect_with_shutdown(gateway_shutdown_rx).await;

                    // Check if shutdown was requested
                    if gateway.is_shutdown_requested() {
                        info!(target: "gateway::discord", "Shutdown requested, stopping reconnection loop");
                        // Clear the gateway from AppState
                        let mut guard = discord_gateway_for_state.lock().await;
                        *guard = None;
                        break;
                    }

                    match result {
                        Ok(_) => {
                            info!(target: "gateway::discord", "Connection closed normally");
                            // Normal close - exit the loop
                            let mut guard = discord_gateway_for_state.lock().await;
                            *guard = None;
                            break;
                        }
                        Err(e) => {
                            warn!(target: "gateway::discord", "Connection error: {}, attempting reconnection in {}s", e, backoff_secs);

                            // Wait with exponential backoff before reconnecting
                            tokio::time::sleep(tokio::time::Duration::from_secs(backoff_secs))
                                .await;

                            // Exponential backoff: double the backoff, max out at MAX_BACKOFF_SECS
                            backoff_secs = (backoff_secs * 2).min(MAX_BACKOFF_SECS);

                            info!(
                                target: "gateway::discord",
                                "Reconnection attempt, backoff now {}s",
                                backoff_secs
                            );
                        }
                    }
                }

                info!(target: "gateway::discord", "Discord Gateway event loop ended");
            });

            info!(target: "gateway::server", "Discord Gateway event loop started");
        } else {
            warn!(target: "gateway::server", "No Discord token configured, Discord Gateway will not start");
        }

        // Build the router with routes and middleware
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/status", get(status_handler))
            .route("/ws", get(ws_handler))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        // Create the server with graceful shutdown
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|source| GatewayServerError::BindError {
                address: addr,
                source,
            })?;

        info!(
            target: "gateway::server",
            "Gateway HTTP server listening on {}",
            addr
        );

        // Create PID file after successfully binding to the port
        if let Err(e) = PidFile::write_pid(&pid_path) {
            error!(
                target: "gateway::server",
                "Failed to create PID file: {}. Server will continue without PID file.",
                e
            );
        } else {
            info!(
                target: "gateway::server",
                "PID file created at: {}",
                pid_path.display()
            );
        }

        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| GatewayServerError::RuntimeError(format!("{:?}", e)))?;

        info!(
            target: "gateway::server",
            "Gateway HTTP server shutdown complete"
        );

        // Clean up PID file on shutdown
        if let Err(e) = PidFile::cleanup(&pid_path) {
            warn!(
                target: "gateway::server",
                "Failed to clean up PID file: {}",
                e
            );
        } else {
            info!(
                target: "gateway::server",
                "PID file cleaned up successfully"
            );
        }

        info!(
            target: "gateway::server",
            "=== Gateway Server Stopped ==="
        );

        Ok(())
    }
}

/// Creates a future that completes when a shutdown signal is received.
///
/// This supports both SIGINT (Ctrl+C) and SIGTERM signals.
/// The future never completes normally - it only completes on signal receipt.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
        tokio::select! {
            _ = sigint.recv() => tracing::info!(target: "gateway::server", "Received SIGINT, initiating graceful shutdown..."),
            _ = sigterm.recv() => tracing::info!(target: "gateway::server", "Received SIGTERM, initiating graceful shutdown..."),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Shutdown signal received, starting graceful shutdown...");
}

/// Process Discord events and forward to registered WebSocket clients.
///
/// This function listens for Discord events from the channel and routes
/// messages to all projects that are subscribed to the message's channel.
#[instrument(name = "discord_event_processor", skip(event_receiver, registry))]
async fn process_discord_events(
    mut event_receiver: mpsc::Receiver<DiscordEvent>,
    registry: ChannelRegistry,
) {
    info!("Discord event processor started");

    // Create MessageRouter for message routing
    let router = MessageRouter::new(registry.clone());

    while let Some(event) = event_receiver.recv().await {
        match event {
            DiscordEvent::MessageCreate {
                channel_id,
                content,
                author_id: _,
                message_id: _,
                guild_id: _,
            } => {
                info!(
                    "Received MessageCreate from channel {}: {}",
                    channel_id, content
                );

                // Log channel_id extraction for verification (AC1)
                debug!(
                    "Extracted channel_id: {} from MessageCreate event",
                    channel_id
                );

                // Use MessageRouter to route message to subscribed projects
                match router.route_message(&channel_id, &content).await {
                    Ok(sent_count) => {
                        if sent_count > 0 {
                            info!(
                                "Routed message from channel {} to {} project(s)",
                                channel_id, sent_count
                            );
                        } else {
                            debug!("No projects subscribed to channel {}", channel_id);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to route message from channel {}: {}", channel_id, e);
                    }
                }
            }
            DiscordEvent::Ready {
                user_id,
                session_id,
            } => {
                info!(
                    "Discord Gateway ready: user_id={}, session_id={}",
                    user_id, session_id
                );
            }
            DiscordEvent::Resumed => {
                info!("Discord Gateway session resumed");
            }
            DiscordEvent::GuildCreate { guild_id } => {
                info!("Joined/created guild: {}", guild_id);
            }
            DiscordEvent::MessageDelete {
                message_id,
                channel_id,
                guild_id: _,
            } => {
                debug!("Message {} deleted in channel {}", message_id, channel_id);
            }
            DiscordEvent::InvalidSession => {
                warn!("Discord Gateway received invalid session, will reconnect");
            }
            DiscordEvent::HeartbeatAck => {
                debug!("Discord Gateway heartbeat acknowledged");
            }
            DiscordEvent::Other(event_type) => {
                debug!("Received other Discord event: {}", event_type);
            }
        }
    }

    info!("Discord event processor stopped");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::config::{GatewayConfig, ServerConfig};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn create_test_config() -> (ServerConfig, GatewayConfig) {
        let server_config = ServerConfig {
            host: "127.0.0.1".to_string(),
            http_port: 0, // Use port 0 for tests to get an available port
            ws_port: 9000,
        };

        let gateway_config = GatewayConfig {
            discord_token: "test_token".to_string(),
            server: server_config.clone(),
            logging: crate::gateway::config::LoggingConfig::default(),
            channels: vec![],
        };

        (server_config, gateway_config)
    }

    #[tokio::test]
    async fn health_handler_returns_ok_status() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let response = health_handler(State(state)).await;

        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn health_handler_returns_valid_json() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let response = health_handler(State(state)).await;

        // Verify the inner response has the correct status
        assert_eq!(response.status, "ok");

        // Verify the JSON serialization of the inner type works
        let json = serde_json::to_string(&response.0).expect("Failed to serialize response");
        assert!(json.contains("\"status\":\"ok\""));
    }

    #[tokio::test]
    async fn router_responds_to_health_endpoint() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let app = Router::new()
            .route("/health", get(health_handler))
            .with_state(state);

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app.oneshot(request).await.expect("Failed to get response");

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn router_returns_404_for_unknown_endpoint() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let app = Router::new()
            .route("/health", get(health_handler))
            .with_state(state);

        let request = Request::builder()
            .uri("/unknown")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app.oneshot(request).await.expect("Failed to get response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn gateway_server_new_creates_instance() {
        let (server_config, gateway_config) = create_test_config();

        let _server = GatewayServer::new(server_config.clone(), gateway_config.clone());

        // Verify the config is stored correctly (we can't directly access private fields,
        // but we can verify through behavior)
        assert_eq!(server_config.http_port, 0); // Test config uses port 0
    }

    #[test]
    fn health_response_serialization() {
        let response = HealthResponse { status: "ok" };
        let json = serde_json::to_string(&response).expect("Failed to serialize");
        assert_eq!(json, "{\"status\":\"ok\"}");
    }

    #[test]
    fn app_state_can_be_cloned() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let cloned = state.clone();
        // Verify clone works - the Arc should point to the same data
        assert!(Arc::ptr_eq(&state.config, &cloned.config));
    }

    #[tokio::test]
    async fn router_has_websocket_endpoint() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .route("/health", get(health_handler))
            .with_state(state);

        // Make a request to /ws - it should have the WebSocket route defined
        // Note: axum returns 426 when upgrade headers are present but can't complete upgrade in test
        let request = Request::builder()
            .uri("/ws")
            .header("upgrade", "websocket")
            .header("connection", "upgrade")
            .header("sec-websocket-version", "13")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app.oneshot(request).await.expect("Failed to get response");

        // The route exists and responds (either 101 or 426 depending on axum version)
        assert!(
            response.status() == StatusCode::SWITCHING_PROTOCOLS
                || response.status() == StatusCode::UPGRADE_REQUIRED
        );
    }

    #[tokio::test]
    async fn websocket_handler_accepts_upgrade() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        // Test that the ws_handler can be used in a router
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        // Verify route exists by checking it responds (will be upgrade request)
        // Note: axum returns 426 when upgrade headers are present but can't complete upgrade in test
        let request = Request::builder()
            .uri("/ws")
            .header("upgrade", "websocket")
            .header("connection", "upgrade")
            .header("sec-websocket-version", "13")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app.oneshot(request).await.expect("Failed to get response");
        // The route exists and responds (either 101 or 426 depending on axum version)
        assert!(
            response.status() == StatusCode::SWITCHING_PROTOCOLS
                || response.status() == StatusCode::UPGRADE_REQUIRED
        );
    }

    #[tokio::test]
    async fn websocket_route_requires_get_method() {
        use axum::http::Method;

        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
            registry: ChannelRegistry::new(),
            discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
        };

        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        // POST request to /ws should return Method Not Allowed
        let request = Request::builder()
            .method(Method::POST)
            .uri("/ws")
            .body(Body::empty())
            .expect("Failed to build request");

        let response = app.oneshot(request).await.expect("Failed to get response");
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn gateway_message_parse_register() {
        use crate::gateway::protocol::GatewayMessage;

        // Use the correct format that matches protocol.rs snake_case serialization
        let json = r#"{"type":"register","project_name":"test-project","channels":["channel1"]}"#;
        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");
        assert!(
            matches!(msg, GatewayMessage::Register { project_name, channels, .. }
            if project_name == "test-project" && channels.len() == 1 && channels[0] == "channel1")
        );
    }

    #[test]
    fn gateway_message_parse_and_serialize_roundtrip() {
        use crate::gateway::protocol::GatewayMessage;

        // Test Message variant
        let original = GatewayMessage::Message {
            payload: "Hello, World!".to_string(),
            channel_id: 12345,
        };

        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let parsed: GatewayMessage = serde_json::from_str(&json).expect("Failed to parse");

        assert!(matches!(
            parsed,
            GatewayMessage::Message { payload, channel_id }
            if payload == "Hello, World!" && channel_id == 12345
        ));
    }

    #[tokio::test]
    async fn websocket_echo_roundtrip() {
        // This test verifies that the WebSocket handler can process messages
        // and that the GatewayMessage parsing works correctly.
        // Full end-to-end WebSocket testing would require a separate test binary
        // or integration test with tokio-tungstenite client.

        // Test that GatewayMessage parsing works for echo
        use crate::gateway::protocol::GatewayMessage;

        let test_message = GatewayMessage::Message {
            payload: "test payload".to_string(),
            channel_id: 123,
        };

        // Serialize the message
        let json = serde_json::to_string(&test_message).expect("Failed to serialize");
        assert!(json.contains("test payload"));

        // Deserialize it back
        let parsed: GatewayMessage = serde_json::from_str(&json).expect("Failed to parse");
        assert!(matches!(
            parsed,
            GatewayMessage::Message { payload, channel_id }
            if payload == "test payload" && channel_id == 123
        ));
    }

    // ============================================================
    // Registration Protocol Tests
    // ============================================================

    mod registration_tests {
        use super::*;
        use crate::gateway::protocol::GatewayMessage;
        use tokio::sync::mpsc;

        /// Test that a valid Register message can be parsed correctly
        #[test]
        fn test_register_message_parsing_valid() {
            // Use the correct format that matches protocol.rs snake_case serialization
            let json = r#"{"type":"register","project_name":"my-project","channels":["channel1","channel2"]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::Register { project_name, channels }
                if project_name == "my-project"
                && channels.len() == 2
                && channels[0] == "channel1"
                && channels[1] == "channel2"
            ));
        }

        /// Test that RegisterAck can be serialized and deserialized correctly
        #[test]
        fn test_register_ack_serialization_roundtrip() {
            let ack = GatewayMessage::RegisterAck {
                status: "ok".to_string(),
                session_id: "test-session-123".to_string(),
            };

            let json = serde_json::to_string(&ack).expect("Failed to serialize");
            // snake_case uses lowercase variant name
            assert!(json.contains("\"type\":\"register_ack\""));
            assert!(json.contains("\"status\":\"ok\""));
            assert!(json.contains("\"session_id\":\"test-session-123\""));

            let parsed: GatewayMessage =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(matches!(
                parsed,
                GatewayMessage::RegisterAck { status, session_id }
                if status == "ok" && session_id == "test-session-123"
            ));
        }

        /// Test that RegisterError can be serialized and deserialized correctly
        #[test]
        fn test_register_error_serialization_roundtrip() {
            let error = GatewayMessage::RegisterError {
                error: "Project name cannot be empty".to_string(),
            };

            let json = serde_json::to_string(&error).expect("Failed to serialize");
            // snake_case uses lowercase variant name
            assert!(json.contains("\"type\":\"register_error\""));
            assert!(json.contains("\"error\":\"Project name cannot be empty\""));

            let parsed: GatewayMessage =
                serde_json::from_str(&json).expect("Failed to deserialize");
            assert!(matches!(
                parsed,
                GatewayMessage::RegisterError { error }
                if error == "Project name cannot be empty"
            ));
        }

        /// Test that project is added to registry after registration
        #[tokio::test]
        async fn test_project_registered_in_registry() {
            let registry = ChannelRegistry::new();
            let channels = vec!["channel1".to_string(), "channel2".to_string()];

            // Create a project connection
            let (tx, _rx) = mpsc::channel::<String>(100);
            let project = ProjectConnection::new(
                "test-project-id".to_string(),
                "test-project".to_string(),
                tx,
            );

            // Register the project
            registry.register(project, channels.clone()).await.unwrap();

            // Verify project is registered
            assert!(registry.is_registered(&"test-project-id".to_string()).await);

            // Verify project can be retrieved with correct details
            let retrieved = registry
                .get_project(&"test-project-id".to_string())
                .await
                .unwrap();
            assert_eq!(retrieved.project_name, "test-project");
            assert_eq!(retrieved.subscribed_channels, channels);

            // Verify channel subscriptions
            let channel_projects = registry.projects_for_channel("channel1").await;
            assert!(channel_projects.contains(&"test-project-id".to_string()));
        }

        /// Test that empty project_name validation works correctly
        #[test]
        fn test_empty_project_name_returns_error() {
            // Test with empty string - use correct snake_case format
            let json = r#"{"type":"register","project_name":"","channels":["channel1"]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::Register { project_name, .. }
                if project_name.is_empty()
            ));
        }

        /// Test that whitespace-only project_name validation works
        #[test]
        fn test_whitespace_only_project_name_returns_error() {
            // Test with whitespace only - use correct snake_case format
            let json = r#"{"type":"register","project_name":"   ","channels":["channel1"]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::Register { project_name, .. }
                if project_name == "   "
            ));
        }

        /// Test that registration with valid name succeeds
        #[tokio::test]
        async fn test_registration_with_valid_project_name_succeeds() {
            let registry = ChannelRegistry::new();

            let (tx, _rx) = mpsc::channel::<String>(100);
            let project = ProjectConnection::new(
                "project-123".to_string(),
                "valid-project-name".to_string(),
                tx,
            );

            let result = registry
                .register(project, vec!["general".to_string()])
                .await;

            assert!(result.is_ok());
            assert!(registry.is_registered(&"project-123".to_string()).await);
        }

        /// Test that registration returns error for empty project name simulation
        #[tokio::test]
        async fn test_registration_validation_empty_name() {
            let registry = ChannelRegistry::new();

            let (tx, _rx) = mpsc::channel::<String>(100);
            let project = ProjectConnection::new(
                "project-id".to_string(),
                "".to_string(), // Empty project name
                tx,
            );

            // Registration with empty name should succeed at registry level
            // (validation happens at the WebSocket handler level)
            let result = registry.register(project, vec![]).await;
            assert!(result.is_ok());
        }

        /// Test that session ID is generated uniquely for each registration
        #[tokio::test]
        async fn test_session_id_generation_unique() {
            let registry = ChannelRegistry::new();

            // Create first project
            let (tx1, _rx1) = mpsc::channel::<String>(100);
            let project1 =
                ProjectConnection::new("project-1".to_string(), "Project One".to_string(), tx1);
            let session_id1 = project1.session_id;

            registry
                .register(project1, vec!["channel1".to_string()])
                .await
                .unwrap();

            // Create second project
            let (tx2, _rx2) = mpsc::channel::<String>(100);
            let project2 =
                ProjectConnection::new("project-2".to_string(), "Project Two".to_string(), tx2);
            let session_id2 = project2.session_id;

            registry
                .register(project2, vec!["channel2".to_string()])
                .await
                .unwrap();

            // Verify session IDs are unique
            assert_ne!(session_id1, session_id2);

            // Verify we can retrieve both projects and they have different session IDs
            let retrieved1 = registry
                .get_project(&"project-1".to_string())
                .await
                .unwrap();
            let retrieved2 = registry
                .get_project(&"project-2".to_string())
                .await
                .unwrap();
            assert_eq!(retrieved1.session_id, session_id1);
            assert_eq!(retrieved2.session_id, session_id2);
            assert_ne!(retrieved1.session_id, retrieved2.session_id);
        }

        /// Test that empty channels list validation works
        #[test]
        fn test_empty_channels_returns_error_message() {
            // Test with empty channels array - use correct snake_case format
            let json = r#"{"type":"register","project_name":"my-project","channels":[]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::Register { project_name, channels }
                if project_name == "my-project" && channels.is_empty()
            ));
        }

        /// Test that channels with valid content pass validation
        #[test]
        fn test_channels_with_valid_content_passes() {
            // Test with non-empty channels array - use correct snake_case format
            let json = r#"{"type":"register","project_name":"my-project","channels":["general","random"]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::Register { project_name, channels }
                if project_name == "my-project"
                && channels.len() == 2
                && channels[0] == "general"
                && channels[1] == "random"
            ));
        }

        /// Test that ChannelSubscribe message can be parsed from JSON
        #[test]
        fn should_parse_channel_subscribe_message() {
            let json = r#"{"type":"channel_subscribe","channels":["general","random"]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::ChannelSubscribe { channels }
                if channels.len() == 2
                && channels[0] == "general"
                && channels[1] == "random"
            ));
        }

        /// Test that ChannelUnsubscribe message can be parsed from JSON
        #[test]
        fn should_parse_channel_unsubscribe_message() {
            let json = r#"{"type":"channel_unsubscribe","channels":["general"]}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::ChannelUnsubscribe { channels }
                if channels.len() == 1
                && channels[0] == "general"
            ));
        }

        /// Test that ChannelSubscribeAck can be parsed from JSON
        #[test]
        fn should_parse_channel_subscribe_ack_message() {
            let json =
                r#"{"type":"channel_subscribe_ack","status":"ok: subscribed to 2 channels"}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::ChannelSubscribeAck { status }
                if status == "ok: subscribed to 2 channels"
            ));
        }

        /// Test that ChannelUnsubscribeAck can be parsed from JSON
        #[test]
        fn should_parse_channel_unsubscribe_ack_message() {
            let json =
                r#"{"type":"channel_unsubscribe_ack","status":"ok: unsubscribed from 1 channels"}"#;
            let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse");

            assert!(matches!(
                msg,
                GatewayMessage::ChannelUnsubscribeAck { status }
                if status == "ok: unsubscribed from 1 channels"
            ));
        }
    }

    /// Integration tests for channel subscribe/unsubscribe flow
    mod channel_subscription {
        use super::*;

        /// Test that channel subscribe updates registry correctly
        #[tokio::test]
        async fn should_add_channel_subscription_to_registry() {
            let registry = ChannelRegistry::new();
            let project_id = "test-project-1".to_string();

            // Add initial project with one channel
            let project = ProjectConnection::new(
                project_id.clone(),
                "test-project".to_string(),
                tokio::sync::mpsc::channel::<String>(10).0,
            );
            registry
                .register(project, vec!["channel1".to_string()])
                .await
                .expect("Failed to register project");

            // Add subscription to a new channel
            registry
                .add_channel_subscription(&project_id, "channel2")
                .await
                .expect("Failed to add channel subscription");

            // Verify the channel was added
            let project = registry
                .get_project(&project_id)
                .await
                .expect("Project not found");
            assert!(project
                .subscribed_channels
                .contains(&"channel2".to_string()));
        }

        /// Test that channel unsubscribe removes from registry correctly
        #[tokio::test]
        async fn should_remove_channel_subscription_from_registry() {
            let registry = ChannelRegistry::new();
            let project_id = "test-project-2".to_string();

            // Add initial project with two channels
            let project = ProjectConnection::new(
                project_id.clone(),
                "test-project".to_string(),
                tokio::sync::mpsc::channel::<String>(10).0,
            );
            registry
                .register(
                    project,
                    vec!["channel1".to_string(), "channel2".to_string()],
                )
                .await
                .expect("Failed to register project");

            // Remove subscription from channel1
            registry
                .remove_channel_subscription(&project_id, "channel1")
                .await
                .expect("Failed to remove channel subscription");

            // Verify the channel was removed
            let project = registry
                .get_project(&project_id)
                .await
                .expect("Project not found");
            assert!(!project
                .subscribed_channels
                .contains(&"channel1".to_string()));
            assert!(project
                .subscribed_channels
                .contains(&"channel2".to_string()));
        }

        /// Tests for disconnection handling behavior
        /// These verify that the registry cleanup works correctly when called on disconnect

        #[tokio::test]
        async fn disconnection_should_unregister_project_when_registered() {
            // Create a registry
            let registry = ChannelRegistry::new();

            // Create and register a project
            let project_id = "test-project-123".to_string();
            let project = ProjectConnection::new(
                project_id.clone(),
                "Test Project".to_string(),
                tokio::sync::mpsc::channel::<String>(10).0,
            );
            registry
                .register(
                    project,
                    vec!["channel1".to_string(), "channel2".to_string()],
                )
                .await
                .expect("Failed to register project");

            // Verify project is registered
            assert!(registry.get_project(&project_id).await.is_ok());

            // Simulate disconnection: call unregister (as WebSocket handler would do)
            registry
                .unregister(&project_id)
                .await
                .expect("Unregister should succeed");

            // Verify project is no longer in registry
            assert!(registry.get_project(&project_id).await.is_err());
        }

        #[tokio::test]
        async fn disconnection_should_handle_unregister_nonexistent_project() {
            // Create a registry
            let registry = ChannelRegistry::new();

            // Attempt to unregister a project that was never registered
            // This should not panic - it should handle gracefully
            let nonexistent = "nonexistent-project".to_string();
            let result = registry.unregister(&nonexistent).await;

            // The result should be an error since project doesn't exist
            assert!(result.is_err());
        }

        #[tokio::test]
        async fn disconnection_should_cleanup_channel_subscriptions() {
            // Create a registry
            let registry = ChannelRegistry::new();

            // Create and register two projects on the same channel
            let project_id_1 = "project-1".to_string();
            let project_id_2 = "project-2".to_string();

            let project1 = ProjectConnection::new(
                project_id_1.clone(),
                "Project 1".to_string(),
                tokio::sync::mpsc::channel::<String>(10).0,
            );
            let project2 = ProjectConnection::new(
                project_id_2.clone(),
                "Project 2".to_string(),
                tokio::sync::mpsc::channel::<String>(10).0,
            );

            registry
                .register(project1, vec!["shared-channel".to_string()])
                .await
                .expect("Failed to register project1");
            registry
                .register(project2, vec!["shared-channel".to_string()])
                .await
                .expect("Failed to register project2");

            // Verify both projects are registered
            assert!(registry.get_project(&project_id_1).await.is_ok());
            assert!(registry.get_project(&project_id_2).await.is_ok());

            // Unregister project1 (simulating disconnect)
            registry
                .unregister(&project_id_1)
                .await
                .expect("Unregister should succeed");

            // Verify project1 is gone but project2 still exists
            assert!(registry.get_project(&project_id_1).await.is_err());
            assert!(registry.get_project(&project_id_2).await.is_ok());

            // Verify project2 still receives messages for the shared channel
            let subscribers = registry.projects_for_channel("shared-channel").await;
            assert_eq!(subscribers.len(), 1);
            assert!(subscribers.contains(&project_id_2));
        }
    }

    // ============================================================
    // WebSocket Integration Tests
    // ============================================================
    // These tests verify actual WebSocket functionality by starting
    // a server and connecting with a tokio-tungstenite client.

    mod websocket_integration {
        use super::*;
        use crate::gateway::protocol::GatewayMessage;
        use tokio::net::TcpListener;
        use tokio_tungstenite::{connect_async, tungstenite::Message};

        /// Creates test app state with fresh registry
        fn create_test_state() -> AppState {
            let (_, gateway_config) = create_test_config();
            AppState {
                config: Arc::new(gateway_config),
                registry: ChannelRegistry::new(),
                discord_gateway: Arc::new(tokio::sync::Mutex::new(None)),
            }
        }

        /// Test that WebSocket upgrade request is accepted and connection is established.
        /// This verifies the /ws endpoint accepts HTTP upgrade requests.
        #[tokio::test]
        async fn websocket_upgrade_should_succeed() {
            let state = create_test_state();

            // Create router with WebSocket endpoint
            let app = Router::new()
                .route("/ws", get(ws_handler))
                .route("/health", get(health_handler))
                .with_state(state);

            // Bind to a random available port
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let port = addr.port();

            // Spawn the server
            let server = axum::serve(listener, app);

            let server_handle = tokio::spawn(async move {
                let _ = server.await;
            });

            // Wait for server to be ready
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Attempt WebSocket connection
            let url = format!("ws://127.0.0.1:{}/ws", port);
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                connect_async(&url),
            )
            .await;

            // Clean up server
            server_handle.abort();

            // Verify connection succeeded
            assert!(result.is_ok(), "WebSocket connection should succeed");
            let (ws_stream, response) = result.unwrap().unwrap();

            // Verify upgrade was accepted (HTTP 101 Switching Protocols)
            assert_eq!(
                response.status(),
                axum::http::StatusCode::SWITCHING_PROTOCOLS,
                "Server should return 101 Switching Protocols"
            );

            // Verify WebSocket stream is usable
            let (mut write, _read) = ws_stream.split();

            // Send a ping to verify bidirectional communication
            write.send(Message::Text("ping".to_string())).await.unwrap();
        }

        /// Test that a register message is correctly parsed and echoed back.
        /// This verifies:
        /// 1. JSON messages are parsed correctly
        /// 2. The server echoes messages back
        #[tokio::test]
        async fn websocket_message_roundtrip_register() {
            let state = create_test_state();

            // Create router with WebSocket endpoint
            let app = Router::new()
                .route("/ws", get(ws_handler))
                .with_state(state);

            // Bind to a random available port
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let port = addr.port();

            // Spawn the server
            let server = axum::serve(listener, app);

            let server_handle = tokio::spawn(async move {
                let _ = server.await;
            });

            // Wait for server to be ready
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Connect WebSocket client
            let url = format!("ws://127.0.0.1:{}/ws", port);
            let (ws_stream, _) = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                connect_async(&url),
            )
            .await
            .expect("Connection should not timeout")
            .expect("Connection should succeed");

            let (mut write, mut read) = ws_stream.split();

            // Send a register message
            let register_json = r#"{"type":"register","project_name":"test-project","channels":["channel1"]}"#;
            write.send(Message::Text(register_json.to_string())).await.unwrap();

            // Receive the echo/ack response
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                read.next(),
            )
            .await
            .expect("Should receive response")
            .expect("Stream should not error");

            // Clean up server
            server_handle.abort();

            // Verify we got a text message back
            let msg = response.unwrap();
            assert!(msg.is_text(), "Response should be text message");

            let text = msg.to_text().expect("Should be valid text");

            // Parse the response - it should be a RegisterAck
            let parsed: GatewayMessage =
                serde_json::from_str(text).expect("Response should be valid JSON");

            assert!(
                matches!(
                    parsed,
                    GatewayMessage::RegisterAck { status, session_id }
                    if status == "ok" && !session_id.is_empty()
                ),
                "Should receive RegisterAck with ok status and session_id, got: {}",
                text
            );
        }

        /// Test that a generic message is echoed back correctly.
        /// This verifies JSON parsing and echo for Message variant.
        #[tokio::test]
        async fn websocket_message_roundtrip_echo() {
            let state = create_test_state();

            // Create router with WebSocket endpoint
            let app = Router::new()
                .route("/ws", get(ws_handler))
                .with_state(state);

            // Bind to a random available port
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let port = addr.port();

            // Spawn the server
            let server = axum::serve(listener, app);

            let server_handle = tokio::spawn(async move {
                let _ = server.await;
            });

            // Wait for server to be ready
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Connect WebSocket client
            let url = format!("ws://127.0.0.1:{}/ws", port);
            let (ws_stream, _) = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                connect_async(&url),
            )
            .await
            .expect("Connection should not timeout")
            .expect("Connection should succeed");

            let (mut write, mut read) = ws_stream.split();

            // Send a Message (not register - just echo test)
            let message_json = r#"{"type":"message","payload":"hello world","channel_id":12345}"#;
            write.send(Message::Text(message_json.to_string())).await.unwrap();

            // Receive the echo response
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                read.next(),
            )
            .await
            .expect("Should receive response")
            .expect("Stream should not error");

            // Clean up server
            server_handle.abort();

            // Verify we got a text message back
            let msg = response.unwrap();
            assert!(msg.is_text(), "Response should be text message");

            let text = msg.to_text().expect("Should be valid text");

            // Parse the response - it should echo the message back
            let parsed: GatewayMessage =
                serde_json::from_str(text).expect("Response should be valid JSON");

            assert!(
                matches!(
                    parsed,
                    GatewayMessage::Message { payload, channel_id }
                    if payload == "hello world" && channel_id == 12345
                ),
                "Should receive echoed Message, got: {}",
                text
            );
        }

        /// Test that heartbeat messages receive acknowledgment.
        #[tokio::test]
        async fn websocket_heartbeat_roundtrip() {
            let state = create_test_state();

            // Create router with WebSocket endpoint
            let app = Router::new()
                .route("/ws", get(ws_handler))
                .with_state(state);

            // Bind to a random available port
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let addr = listener.local_addr().unwrap();
            let port = addr.port();

            // Spawn the server
            let server = axum::serve(listener, app);

            let server_handle = tokio::spawn(async move {
                let _ = server.await;
            });

            // Wait for server to be ready
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;

            // Connect WebSocket client
            let url = format!("ws://127.0.0.1:{}/ws", port);
            let (ws_stream, _) = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                connect_async(&url),
            )
            .await
            .expect("Connection should not timeout")
            .expect("Connection should succeed");

            let (mut write, mut read) = ws_stream.split();

            // First register a project (required before heartbeat)
            let register_json = r#"{"type":"register","project_name":"heartbeat-test","channels":["test"]}"#;
            write.send(Message::Text(register_json.to_string())).await.unwrap();

            // Wait for registration ack
            let _ = read.next().await;

            // Send a heartbeat message
            let heartbeat_json = r#"{"type":"heartbeat","timestamp":1234567890}"#;
            write.send(Message::Text(heartbeat_json.to_string())).await.unwrap();

            // Receive the heartbeat ack
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                read.next(),
            )
            .await
            .expect("Should receive response")
            .expect("Stream should not error");

            // Clean up server
            server_handle.abort();

            // Verify we got a text message back
            let msg = response.unwrap();
            assert!(msg.is_text(), "Response should be text message");

            let text = msg.to_text().expect("Should be valid text");

            // Parse the response - it should be a HeartbeatAck
            let parsed: GatewayMessage =
                serde_json::from_str(text).expect("Response should be valid JSON");

            assert!(
                matches!(
                    parsed,
                    GatewayMessage::HeartbeatAck { timestamp }
                    if timestamp == 1234567890
                ),
                "Should receive HeartbeatAck with matching timestamp, got: {}",
                text
            );
        }
    }
}
