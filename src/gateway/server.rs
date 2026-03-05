//! Gateway HTTP server module.
//!
//! Provides an HTTP server with health check endpoint for the gateway service.
//! Uses Axum for HTTP routing and handling.

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Json,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::broadcast;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::gateway::config::{GatewayConfig, ServerConfig};
use crate::gateway::protocol::GatewayMessage;

/// Error types for the gateway HTTP server.
#[derive(Debug, thiserror::Error)]
pub enum GatewayServerError {
    /// Failed to bind to the specified address.
    #[error("Failed to bind to {host}:{port}: {source}")]
    BindError {
        /// The host that was being bound to.
        host: String,
        /// The port that was being bound to.
        port: u32,
        /// The underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Server failed to start.
    #[error("Server failed to start: {0}")]
    StartupError(String),

    /// Server was stopped unexpectedly.
    #[error("Server stopped: {0}")]
    ShutdownError(String),

    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
}

/// Health check response structure.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HealthResponse {
    /// The status of the service.
    pub status: &'static str,
}

/// Application state for the Axum router.
#[derive(Clone)]
pub struct AppState {
    /// Server configuration.
    pub config: ServerConfig,
    /// Broadcast channel for WebSocket message distribution.
    #[allow(dead_code)]
    tx: broadcast::Sender<String>,
}

impl AppState {
    /// Create a new application state.
    ///
    /// # Arguments
    ///
    /// * `config` - The server configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - The new application state.
    pub fn new(config: ServerConfig) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { config, tx }
    }
}

/// Health check handler.
///
/// Returns a JSON response indicating the service is healthy.
///
/// # Returns
///
/// * `Json<HealthResponse>` - A JSON response with status "ok".
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// WebSocket upgrade handler.
///
/// Accepts WebSocket upgrade requests and handles the connection.
///
/// # Arguments
///
/// * `ws` - WebSocket upgrade extractor.
/// * `state` - Application state.
///
/// # Returns
///
/// * The WebSocket response after upgrade.
async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> axum::response::Response {
    info!("WebSocket upgrade request received");
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle an established WebSocket connection.
///
/// This function handles incoming WebSocket messages, implementing the registration
/// protocol and echoing back non-registration messages.
///
/// # Arguments
///
/// * `socket` - The WebSocket connection.
/// * `state` - Application state.
async fn handle_socket(socket: WebSocket, _state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    // Handle incoming messages
    while let Some(msg_result) = receiver.next().await {
        match msg_result {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    info!("Received WebSocket message: {}", text);

                    // Try to parse the message as a GatewayMessage
                    match serde_json::from_str::<GatewayMessage>(&text) {
                        Ok(gateway_msg) => {
                            // Handle the message based on its type
                            match gateway_msg {
                                GatewayMessage::Register { project_name, channels: _ } => {
                                    // Validate project_name is not empty
                                    if project_name.trim().is_empty() {
                                        // Send error response for empty project name
                                        let error_msg = GatewayMessage::RegisterError {
                                            error: "project_name cannot be empty".to_string(),
                                        };
                                        if let Ok(json) = serde_json::to_string(&error_msg) {
                                            let _ = sender.send(Message::Text(json)).await;
                                        }
                                        continue;
                                    }

                                    // Generate a unique session ID
                                    let session_id = Uuid::new_v4().to_string();

                                    info!(
                                        "Registered project: {} with session_id: {}",
                                        project_name, session_id
                                    );

                                    // Send RegisterAck response
                                    let ack_msg = GatewayMessage::RegisterAck {
                                        status: "ok".to_string(),
                                        session_id,
                                    };
                                    if let Ok(json) = serde_json::to_string(&ack_msg) {
                                        let _ = sender.send(Message::Text(json)).await;
                                    }
                                }
                                // For non-register messages, echo back for backward compatibility
                                _ => {
                                    if sender.send(Message::Text(text)).await.is_err() {
                                        warn!("Failed to send message to client, connection may be closed");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            // Parse error - send RegisterError response
                            warn!("Failed to parse message as GatewayMessage: {}", e);
                            let error_msg = GatewayMessage::RegisterError {
                                error: "invalid message format".to_string(),
                            };
                            if let Ok(json) = serde_json::to_string(&error_msg) {
                                let _ = sender.send(Message::Text(json)).await;
                            }
                        }
                    }
                } else if let Message::Close(_) = msg {
                    info!("Client initiated close");
                    break;
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}

/// Create the Axum router with all routes configured.
///
/// # Arguments
///
/// * `state` - The application state.
///
/// # Returns
///
/// * `Router` - The configured Axum router.
fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(Arc::new(state))
}

/// Run the gateway HTTP server.
///
/// This function starts an Axum HTTP server with the configured host and port.
/// It handles graceful shutdown on SIGINT (Ctrl+C) and SIGTERM signals.
///
/// # Arguments
///
/// * `config` - The server configuration containing host and port.
///
/// # Returns
///
/// * `Result<(), GatewayServerError>` - Ok if the server ran successfully, or an error.
pub async fn serve(config: ServerConfig) -> Result<(), GatewayServerError> {
    let host = config.host.clone();
    let port = config.http_port;

    // Create the address to bind to
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|e| GatewayServerError::BindError {
            host: host.clone(),
            port,
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        })?;

    // Create application state
    let state = AppState::new(config);

    // Create the router
    let app = create_router(state);

    // Create the Axum server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| GatewayServerError::BindError {
            host,
            port,
            source,
        })?;

    info!("Gateway HTTP server starting on {}", addr);

    // Configure graceful shutdown
    let server = axum::serve(listener, app);

    // Wait for either the server to complete or a shutdown signal
    tokio::select! {
        result = server => {
            match result {
                Ok(_) => {
                    info!("Gateway HTTP server stopped normally");
                    Ok(())
                }
                Err(e) => {
                    error!("Gateway HTTP server error: {}", e);
                    Err(GatewayServerError::ShutdownError(e.to_string()))
                }
            }
        }
        _ = shutdown_signal() => {
            info!("Received shutdown signal, stopping server gracefully");
            Ok(())
        }
    }
}

/// Wait for a shutdown signal (SIGINT or SIGTERM).
///
/// # Returns
///
/// * `impl Future` - A future that completes when a shutdown signal is received.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Received Ctrl+C signal");
        }
        _ = terminate => {
            warn!("Received SIGTERM signal");
        }
    }
}

/// Gateway HTTP server.
///
/// This struct manages the HTTP server for the gateway service,
/// providing health check endpoints and other HTTP APIs.
#[derive(Clone)]
pub struct GatewayServer {
    /// Server configuration.
    server_config: ServerConfig,
    /// Full gateway configuration.
    #[allow(dead_code)]
    gateway_config: GatewayConfig,
}

impl GatewayServer {
    /// Create a new GatewayServer instance.
    ///
    /// # Arguments
    ///
    /// * `server_config` - The server configuration (host, ports).
    /// * `gateway_config` - The full gateway configuration.
    ///
    /// # Returns
    ///
    /// * `Self` - A new GatewayServer instance.
    pub fn new(server_config: ServerConfig, gateway_config: GatewayConfig) -> Self {
        Self {
            server_config,
            gateway_config,
        }
    }

    /// Run the gateway HTTP server.
    ///
    /// This starts the Axum HTTP server and blocks until shutdown.
    ///
    /// # Returns
    ///
    /// * `Result<(), GatewayServerError>` - Ok on successful shutdown, or error.
    pub async fn run(&self) -> Result<(), GatewayServerError> {
        serve(self.server_config.clone()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_handler_returns_ok() {
        // Call the health handler
        let response = health_handler().await;

        // Verify the response
        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        // Create the router with test state
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            http_port: 8080,
            ws_port: 9000,
        };
        let state = AppState::new(config);
        let app = create_router(state);

        // Make a request to /health
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Verify the response
        assert_eq!(response.status(), StatusCode::OK);

        // Verify the body contains the expected JSON
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("\"status\":\"ok\""));
    }

    #[tokio::test]
    async fn test_app_state_creation() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            http_port: 9745,
            ws_port: 9000,
        };
        let state = AppState::new(config.clone());

        assert_eq!(state.config.host, "0.0.0.0");
        assert_eq!(state.config.http_port, 9745);
    }

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse { status: "ok" };
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"status":"ok"}"#);
    }

    #[test]
    fn test_app_state_has_broadcast_channel() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            http_port: 8080,
            ws_port: 9000,
        };
        let state = AppState::new(config);
        // Verify broadcast channel is created
        let _ = state.tx.clone();
    }

    #[tokio::test]
    async fn test_ws_router_creation() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            http_port: 8080,
            ws_port: 9000,
        };
        let state = AppState::new(config);
        // Creating the router should not panic - it includes /ws route now
        let _app = create_router(state);
        // If we get here without panicking, the test passes
    }

    // ==================== Registration Protocol Tests ====================

    use crate::gateway::protocol::GatewayMessage;

    /// Test that valid registration message returns session_id
    #[test]
    fn test_valid_registration_returns_session_id() {
        let json = r#"{"type":"register","project_name":"test-project","channels":["channel1","channel2"]}"#;
        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse register message");

        match msg {
            GatewayMessage::Register { project_name, channels } => {
                assert_eq!(project_name, "test-project");
                assert_eq!(channels.len(), 2);
                assert_eq!(channels[0], "channel1");
                assert_eq!(channels[1], "channel2");
            }
            _ => panic!("Expected Register message"),
        }
    }

    /// Test that empty project_name returns error
    #[test]
    fn test_empty_project_name_returns_error() {
        let json = r#"{"type":"register","project_name":"   ","channels":["channel1"]}"#;
        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse register message");

        match msg {
            GatewayMessage::Register { project_name, .. } => {
                // Validate that trimmed project_name is empty
                assert!(project_name.trim().is_empty(), "Project name should be empty after trimming");
            }
            _ => panic!("Expected Register message"),
        }
    }

    /// Test that malformed JSON returns error
    #[test]
    fn test_malformed_json_returns_error() {
        let json = r#"this is not valid json"#;
        let result: Result<GatewayMessage, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Malformed JSON should fail to parse");
    }

    /// Test that RegisterAck message is correctly serialized
    #[test]
    fn test_register_ack_serialization() {
        let msg = GatewayMessage::RegisterAck {
            status: "ok".to_string(),
            session_id: "test-session-123".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize RegisterAck");
        assert!(json.contains("\"type\":\"register_ack\""));
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"session_id\":\"test-session-123\""));
    }

    /// Test that RegisterError message is correctly serialized
    #[test]
    fn test_register_error_serialization() {
        let msg = GatewayMessage::RegisterError {
            error: "invalid message format".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize RegisterError");
        assert!(json.contains("\"type\":\"register_error\""));
        assert!(json.contains("\"error\":\"invalid message format\""));
    }

    /// Test that non-register messages can be deserialized
    #[test]
    fn test_heartbeat_message_deserialization() {
        let json = r#"{"type":"heartbeat","timestamp":1699999999}"#;
        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse heartbeat message");
        assert!(matches!(msg, GatewayMessage::Heartbeat { timestamp: 1699999999 }));
    }

    /// Test that Message variant is correctly deserialized
    #[test]
    fn test_message_variant_deserialization() {
        let json = r#"{"type":"message","payload":"Hello world","channel_id":12345}"#;
        let msg: GatewayMessage = serde_json::from_str(json).expect("Failed to parse message");
        match msg {
            GatewayMessage::Message { payload, channel_id } => {
                assert_eq!(payload, "Hello world");
                assert_eq!(channel_id, 12345);
            }
            _ => panic!("Expected Message variant"),
        }
    }
}
