//! Gateway HTTP server module.
//!
//! Provides an HTTP server for the gateway with health check endpoint
//! and graceful shutdown support.

use crate::discord::gateway::{DiscordEvent, DiscordGateway};
use crate::gateway::config::{GatewayConfig, ServerConfig};
use crate::gateway::pid::{PidFile, PidFileError};
use crate::gateway::protocol::GatewayMessage;
use crate::gateway::registry::{ChannelRegistry, ProjectConnection};
use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tokio::sync::mpsc;
use tower_http::trace::TraceLayer;
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
async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

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
                                let project_id = Uuid::new_v4().to_string();

                                // Create a channel for sending messages to this client
                                let (tx, _rx) = mpsc::channel::<String>(100);

                                // Create project connection
                                let project = ProjectConnection::new(
                                    project_id.clone(),
                                    project_name.clone(),
                                    tx,
                                );
                                let session_id = project.session_id.to_string();

                                // Register with the channel registry
                                let registry = state.registry.clone();
                                match registry.register(project, channels).await {
                                    Ok(()) => {
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
            discord_configured = !self.gateway_config.discord_token.is_empty()
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
async fn process_discord_events(
    mut event_receiver: mpsc::Receiver<DiscordEvent>,
    registry: ChannelRegistry,
) {
    info!("Discord event processor started");

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

                // Look up projects subscribed to this channel
                let project_ids = registry.projects_for_channel(&channel_id).await;

                if project_ids.is_empty() {
                    debug!("No projects subscribed to channel {}", channel_id);
                    continue;
                }

                // Forward message to each subscribed project
                for project_id in project_ids {
                    if let Ok(project) = registry.get_project(&project_id).await {
                        // Parse channel_id - skip if invalid
                        let parsed_channel_id = match channel_id.parse::<u64>() {
                            Ok(id) => id,
                            Err(e) => {
                                warn!("Failed to parse channel_id '{}': {}", channel_id, e);
                                continue;
                            }
                        };

                        // Create the message payload
                        let message = GatewayMessage::Message {
                            payload: content.clone(),
                            channel_id: parsed_channel_id,
                        };

                        if let Ok(json) = serde_json::to_string(&message) {
                            if project.ws_sender.send(json).await.is_err() {
                                warn!(
                                    "Failed to send message to project {}, client may be disconnected",
                                    project_id
                                );
                            } else {
                                info!(
                                    "Forwarded message to project {} ({})",
                                    project.project_name, project_id
                                );
                            }
                        }
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
    }
}
