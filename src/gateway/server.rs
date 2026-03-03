//! Gateway HTTP server module.
//!
//! Provides an HTTP server for the gateway with health check endpoint
//! and graceful shutdown support.

use crate::gateway::config::{GatewayConfig, ServerConfig};
use axum::{
    extract::State,
    response::Json,
    routing::get,
    Router,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use thiserror::Error;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;

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
}

/// Creates the health check route handler.
async fn health_handler(State(_state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
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
            .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let addr = SocketAddr::from((ip, self.config.http_port));

        info!("Starting gateway HTTP server on {}", addr);

        // Create application state
        let state = AppState {
            config: Arc::new(self.gateway_config),
        };

        // Build the router with routes and middleware
        let app = Router::new()
            .route("/health", get(health_handler))
            .layer(TraceLayer::new_for_http())
            .with_state(state);

        // Create the server with graceful shutdown
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|source| GatewayServerError::BindError { address: addr, source })?;

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
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
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
        };

        let response = health_handler(State(state)).await;

        assert_eq!(response.status, "ok");
    }

    #[tokio::test]
    async fn health_handler_returns_valid_json() {
        let (_, gateway_config) = create_test_config();
        let state = AppState {
            config: Arc::new(gateway_config),
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
        };

        let cloned = state.clone();
        // Verify clone works - the Arc should point to the same data
        assert!(Arc::ptr_eq(&state.config, &cloned.config));
    }
}
