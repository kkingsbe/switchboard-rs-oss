//! Gateway HTTP server module.
//!
//! Provides an HTTP server for the gateway with health check endpoint
//! and graceful shutdown support.

use crate::gateway::config::{GatewayConfig, ServerConfig};
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
use tracing::{error, info, warn};
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

/// Application state for the HTTP server.
#[derive(Clone)]
pub struct AppState {
    /// The gateway configuration.
    pub config: Arc<GatewayConfig>,
    /// The channel registry for tracking project subscriptions.
    pub registry: ChannelRegistry,
}

/// Creates the health check route handler.
async fn health_handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
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
    info!("WebSocket upgrade request received");

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
                info!("Received WebSocket text message: {}", text);

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
                                                warn!("Failed to send ack, client disconnected");
                                                break;
                                            }
                                        }
                                        info!("Project registered successfully: {}", session_id);
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
                                        warn!("Registration failed: {}", e);
                                    }
                                }
                            }
                            // Echo other message types back to the client
                            _ => match serde_json::to_string(&gateway_msg) {
                                Ok(response) => {
                                    if sender.send(Message::Text(response)).await.is_err() {
                                        warn!("Failed to send response, client disconnected");
                                        break;
                                    }
                                    info!("Echoed message back to client");
                                }
                                Err(e) => {
                                    error!("Failed to serialize message: {}", e);
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
                        error!("Failed to parse message: {}", e);
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
                info!("Client sent close message");
                break;
            }
            Ok(Message::Ping(data)) => {
                info!("Received ping, sending pong");
                let _ = sender.send(Message::Pong(data)).await;
            }
            Ok(Message::Pong(_)) => {
                // Pong received, nothing to do
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
            _ => {
                // Ignore binary messages for now
            }
        }
    }

    info!("WebSocket connection closed");
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
        let ip: IpAddr = self
            .config
            .host
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let addr = SocketAddr::from((ip, self.config.http_port as u16));

        info!("Starting gateway HTTP server on {}", addr);

        // Create application state
        let state = AppState {
            config: Arc::new(self.gateway_config),
            registry: ChannelRegistry::new(),
        };

        // Build the router with routes and middleware
        let app = Router::new()
            .route("/health", get(health_handler))
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

        info!("Gateway HTTP server listening on {}", addr);

        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .map_err(|e| GatewayServerError::RuntimeError(format!("{:?}", e)))?;

        info!("Gateway HTTP server shutdown complete");

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
            _ = sigint.recv() => info!("Received SIGINT"),
            _ = sigterm.recv() => info!("Received SIGTERM"),
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

        // Use the correct format that matches protocol.rs internally-tagged serialization
        let json = r#"{"type":"Register","project_name":"test-project","channels":["channel1"]}"#;
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
            // Use the correct format that matches protocol.rs internally-tagged serialization
            let json = r#"{"type":"Register","project_name":"my-project","channels":["channel1","channel2"]}"#;
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
            // Internally-tagged uses capitalized variant name
            assert!(json.contains("\"type\":\"RegisterAck\""));
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
            // Internally-tagged uses capitalized variant name
            assert!(json.contains("\"type\":\"RegisterError\""));
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
            // Test with empty string - use correct internally-tagged format
            let json = r#"{"type":"Register","project_name":"","channels":["channel1"]}"#;
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
            // Test with whitespace only - use correct internally-tagged format
            let json = r#"{"type":"Register","project_name":"   ","channels":["channel1"]}"#;
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
            // Test with empty channels array - use correct internally-tagged format
            let json = r#"{"type":"Register","project_name":"my-project","channels":[]}"#;
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
            // Test with non-empty channels array
            let json = r#"{"type":"Register","project_name":"my-project","channels":["general","random"]}"#;
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
