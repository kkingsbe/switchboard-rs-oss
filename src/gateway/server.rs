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

use crate::gateway::config::{GatewayConfig, ServerConfig};

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
/// This function echoes received messages back to the client.
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
                    // Echo the message back to the client
                    if sender.send(Message::Text(text)).await.is_err() {
                        warn!("Failed to send message to client, connection may be closed");
                        break;
                    }
                } else if let Message::Close(_)= msg {
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
}
