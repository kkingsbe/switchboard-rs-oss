//! Axum Router for REST API.
//!
//! This module provides the HTTP router for the Switchboard REST API,
//! including health checks, validation, and endpoints for agents, metrics,
//! status, shutdown, skills, workflows, and project/workflow management.

use crate::api::error::ApiResult;
use crate::api::handlers::agents::*;
use crate::api::handlers::skills::*;
use crate::api::handlers::workflows::*;
use crate::api::handlers::projects::*;
use crate::api::handlers::scheduler::*;
use crate::api::registry::{
    get_instance_dir, get_instance_log_dir, InstanceRegistry,
};
use crate::api::handlers::gateway::*;
use crate::api::state::ApiState;
use axum::{
    extract::State,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};

use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

// ============================================================================
// OpenAPI Specification
// ============================================================================

/// OpenAPI specification for the Switchboard API.
#[derive(OpenApi)]
#[openapi(
    paths(
        health_handler,
        validate_handler,
        instance_info_handler,
        crate::api::handlers::agents::list_agents,
        crate::api::handlers::agents::get_agent,
        crate::api::handlers::agents::run_agent,
        crate::api::handlers::agents::get_agent_logs,
        crate::api::handlers::agents::get_metrics,
        crate::api::handlers::agents::get_status,
        crate::api::handlers::agents::shutdown,
        crate::api::handlers::skills::list_skills,
        crate::api::handlers::skills::install_skill,
        crate::api::handlers::skills::list_installed_skills,
        crate::api::handlers::skills::update_skill,
        crate::api::handlers::skills::remove_skill,
        crate::api::handlers::workflows::list_workflows,
        crate::api::handlers::workflows::install_workflow,
        crate::api::handlers::workflows::list_installed_workflows,
        crate::api::handlers::workflows::update_workflow,
        crate::api::handlers::workflows::remove_workflow,
        crate::api::handlers::workflows::validate_workflow,
        crate::api::handlers::workflows::apply_workflow,
        crate::api::handlers::projects::init_project,
        crate::api::handlers::projects::init_workflow,
        crate::api::handlers::gateway::gateway_up,
        crate::api::handlers::gateway::gateway_status,
        crate::api::handlers::gateway::gateway_down,
        crate::api::handlers::scheduler::scheduler_up,
        crate::api::handlers::scheduler::scheduler_down,
        crate::api::handlers::scheduler::scheduler_status,
        crate::api::handlers::scheduler::scheduler_restart,
    ),
    tags(
        (name = "Health", description = "Health check and status endpoints"),
        (name = "Configuration", description = "Configuration validation endpoints"),
        (name = "Instance", description = "Instance management endpoints"),
        (name = "Agents", description = "Agent management endpoints"),
        (name = "Skills", description = "Skill management endpoints"),
        (name = "Workflows", description = "Workflow management endpoints"),
        (name = "Projects", description = "Project initialization endpoints"),
        (name = "Gateway", description = "Discord gateway endpoints"),
        (name = "Scheduler", description = "Scheduler management endpoints")
    )
)]
pub struct ApiDoc;

// ============================================================================
// Response Types
// ============================================================================

/// Health check response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct HealthResponse {
    /// Service status.
    pub status: &'static str,
    /// Instance identifier.
    pub instance_id: String,
    /// API version.
    pub version: String,
}

/// Validation request body.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct ValidateRequest {
    /// Configuration content to validate.
    pub config: String,
}

/// Validation response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct ValidateResponse {
    /// Whether validation passed.
    pub valid: bool,
    /// List of validation errors (empty if valid).
    pub errors: Vec<ValidationError>,
}

/// Validation error details.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct ValidationError {
    /// Error message.
    pub message: String,
    /// Field where error occurred (if applicable).
    pub field: Option<String>,
}

/// Generic list response for placeholder endpoints.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct ListResponse<T> {
    /// List of items.
    pub items: Vec<T>,
    /// Total count.
    pub total: usize,
}

/// Instance info response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct InstanceInfoResponse {
    /// Instance identifier.
    pub instance_id: String,
    /// API server port.
    pub port: u16,
    /// Host address.
    pub host: String,
    /// Instance log directory.
    pub log_dir: String,
    /// Instance data directory.
    pub data_dir: String,
    /// Config file path.
    pub config_path: Option<String>,
}

/// List all instances response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct InstancesListResponse {
    /// List of instances.
    pub instances: Vec<InstanceInfoResponse>,
    /// Total count.
    pub total: usize,
}

/// Placeholder response for endpoints under development.
#[derive(Serialize, Deserialize, Debug)]
#[derive(ToSchema)]
pub struct PlaceholderResponse {
    /// Placeholder message.
    pub message: String,
    /// Endpoint path.
    pub path: String,
}

// ============================================================================
// Handlers
// ============================================================================

/// Health check handler.
///
/// Returns a JSON response indicating the service is healthy.
///
/// # Arguments
///
/// * `state` - Application state.
///
/// # Returns
///
/// * `Json<HealthResponse>` - A JSON response with health status.
#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
async fn health_handler(State(state): State<Arc<ApiState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        instance_id: state.instance_id.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Configuration validation handler.
///
/// Validates a Switchboard configuration TOML string.
///
/// # Arguments
///
/// * `state` - Application state.
/// * `request` - Validation request containing config TOML.
///
/// # Returns
///
/// * `Json<ValidateResponse>` - Validation result.
#[utoipa::path(
    post,
    path = "/api/v1/validate",
    tag = "Configuration",
    request_body = ValidateRequest,
    responses(
        (status = 200, description = "Validation result", body = ValidateResponse)
    )
)]
async fn validate_handler(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<ValidateRequest>,
) -> ApiResult<Json<ValidateResponse>> {
    // Try to parse the TOML configuration
    match toml::from_str::<toml::Value>(&request.config) {
        Ok(_toml) => {
            // Basic TOML parsing succeeded
            // TODO: Add more comprehensive validation against schema
            Ok(Json(ValidateResponse {
                valid: true,
                errors: vec![],
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();
            // Extract line/column info if available
            let (field, message) = if let Some(span) = e.span() {
                (Some(format!("line {}", span.start)), error_msg)
            } else {
                (None, error_msg)
            };

            Ok(Json(ValidateResponse {
                valid: false,
                errors: vec![ValidationError { message, field }],
            }))
        }
    }
}

/// Get instance info handler.
///
/// Returns information about the current instance.
///
/// # Arguments
///
/// * `state` - Application state.
///
/// # Returns
///
/// * `Json<InstanceInfoResponse>` - A JSON response with instance info.
#[utoipa::path(
    get,
    path = "/api/v1/instance",
    tag = "Instance",
    responses(
        (status = 200, description = "Instance info", body = InstanceInfoResponse)
    )
)]
async fn instance_info_handler(
    State(state): State<Arc<ApiState>>,
) -> Json<InstanceInfoResponse> {
    Json(InstanceInfoResponse {
        instance_id: state.instance_id.clone(),
        port: state.config.port,
        host: state.config.host.clone(),
        log_dir: state.instance_log_dir.to_string_lossy().to_string(),
        data_dir: state.instance_dir.to_string_lossy().to_string(),
        config_path: state.config_path.as_ref().map(|p| p.to_string_lossy().to_string()),
    })
}

/// List all instances handler.
///
/// Returns information about all registered instances from the registry.
///
/// # Arguments
///
/// * `state` - Application state.
///
/// # Returns
///
/// * `Json<InstancesListResponse>` - A JSON response with all instances.
async fn list_instances_handler(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<InstancesListResponse>> {
    match InstanceRegistry::load() {
        Ok(registry) => {
            let instances: Vec<InstanceInfoResponse> = registry
                .instances()
                .iter()
                .map(|i| InstanceInfoResponse {
                    instance_id: i.instance_id.clone(),
                    port: i.port,
                    host: i.host.clone(),
                    log_dir: get_instance_log_dir(&i.instance_id).to_string_lossy().to_string(),
                    data_dir: get_instance_dir(&i.instance_id).to_string_lossy().to_string(),
                    config_path: Some(i.config_path.clone()),
                })
                .collect();

            Ok(Json(InstancesListResponse {
                total: instances.len(),
                instances,
            }))
        }
        Err(e) => {
            warn!("Failed to load instance registry: {}", e);
            Ok(Json(InstancesListResponse {
                total: 0,
                instances: vec![],
            }))
        }
    }
}

// ============================================================================
// Router Creation
// ============================================================================

/// Create the Axum router with all routes configured.
///
/// # Arguments
///
/// * `state` - The application state.
///
/// # Returns
///
/// * `Router` - The configured Axum router.
pub fn create_router(state: Arc<ApiState>) -> Router {
    // Get rate limit configuration and swagger setting
    let _rate_limit_config = state.config.rate_limit.clone();
    let swagger_enabled = state.config.swagger;
    
    let mut router = Router::new()
        // Health check endpoint
        .route("/health", get(health_handler))
        // API v1 endpoints
        .route("/api/v1/validate", post(validate_handler))
        // Instance endpoints
        .route("/api/v1/instance", get(instance_info_handler))
        .route("/api/v1/instances", get(list_instances_handler))
        // Agent endpoints
        .route("/api/v1/agents", get(list_agents))
        .route("/api/v1/agents/:agent_name", get(get_agent))
        .route("/api/v1/agents/:agent_name/run", post(run_agent))
        .route("/api/v1/agents/:agent_name/logs", get(get_agent_logs))
        // Metrics endpoint
        .route("/api/v1/metrics", get(get_metrics))
        // Status endpoint
        .route("/api/v1/status", get(get_status))
        // Shutdown endpoint
        .route("/api/v1/shutdown", post(shutdown))
        // Skills endpoints
        .route("/api/v1/skills", get(list_skills))
        .route("/api/v1/skills", post(install_skill))
        .route("/api/v1/skills/installed", get(list_installed_skills))
        .route("/api/v1/skills/:skill_name", put(update_skill))
        .route("/api/v1/skills/:skill_name", delete(remove_skill))
        // Workflows endpoints
        .route("/api/v1/workflows", get(list_workflows))
        .route("/api/v1/workflows", post(install_workflow))
        .route("/api/v1/workflows/installed", get(list_installed_workflows))
        .route("/api/v1/workflows/:workflow_name", put(update_workflow))
        .route("/api/v1/workflows/:workflow_name", delete(remove_workflow))
        .route("/api/v1/workflows/validate", post(validate_workflow))
        .route("/api/v1/workflows/apply", post(apply_workflow))
        // Project init endpoint
        .route("/api/v1/project/init", post(init_project))
        // Workflow init endpoint
        .route("/api/v1/workflow/init", post(init_workflow))
        // Gateway endpoints
        .route("/api/v1/gateway/up", post(gateway_up))
        .route("/api/v1/gateway/status", get(gateway_status))
        .route("/api/v1/gateway/down", post(gateway_down))
        // Scheduler endpoints
        .route("/api/v1/scheduler/up", post(scheduler_up))
        .route("/api/v1/scheduler/down", post(scheduler_down))
        .route("/api/v1/scheduler/status", get(scheduler_status))
        .route("/api/v1/scheduler/restart", post(scheduler_restart))
        .with_state(state);

    // Add Swagger UI if enabled
    if swagger_enabled {
        eprintln!("DEBUG: Adding Swagger UI...");
        let openapi_spec = ApiDoc::openapi();
        eprintln!("DEBUG: OpenAPI spec generated successfully");
        eprintln!("DEBUG: Creating SwaggerUi...");
        let swagger = SwaggerUi::new("/docs").url("/docs/openapi.json", openapi_spec);
        eprintln!("DEBUG: SwaggerUi created, about to merge...");
        
        // Try to catch any panic during merge
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            router.clone().merge(swagger)
        }));
        
        match result {
            Ok(new_router) => {
                router = new_router;
                eprintln!("DEBUG: Swagger UI merged into router successfully");
            }
            Err(panic_info) => {
                // Try to extract the panic message
                if let Some(s) = panic_info.downcast_ref::<&str>() {
                    eprintln!("DEBUG: PANIC during merge: {}", s);
                } else if let Some(s) = panic_info.downcast_ref::<String>() {
                    eprintln!("DEBUG: PANIC during merge: {}", s);
                } else {
                    eprintln!("DEBUG: PANIC during merge: {:?}", panic_info);
                }
                // Continue without swagger
            }
        }
    }

    // Add middleware
    let service_builder = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http());

    router.layer(service_builder)
}

/// Run the API HTTP server.
///
/// This function starts an Axum HTTP server with the configured host and port.
/// It handles graceful shutdown on SIGINT (Ctrl+C) and SIGTERM signals.
///
/// # Arguments
///
/// * `config` - The API configuration.
///
/// # Returns
///
/// * `Result<(), ApiServerError>` - Ok if the server ran successfully, or an error.
pub async fn serve(config: crate::config::ApiConfig) -> Result<(), ApiServerError> {
    serve_with_config(config, None).await
}

/// Run the API HTTP server with a switchboard config.
///
/// This function starts an Axum HTTP server with the configured host and port.
/// It handles graceful shutdown on SIGINT (Ctrl+C) and SIGTERM signals.
/// Includes the switchboard config for agent management endpoints.
///
/// # Arguments
///
/// * `api_config` - The API configuration.
/// * `config_path` - Optional path to the switchboard config file.
///
/// # Returns
///
/// * `Result<(), ApiServerError>` - Ok if the server ran successfully, or an error.
pub async fn serve_with_config(
    api_config: crate::config::ApiConfig,
    config_path: Option<&str>,
) -> Result<(), ApiServerError> {
    use std::net::SocketAddr;
    use std::path::PathBuf;
    use tracing::{error, info, warn};

    let host = api_config.host.clone();
    let port = api_config.port;

    // Create the address to bind to
    let addr: SocketAddr = format!("{}:{}", host, port).parse().map_err(|e| {
        ApiServerError::BindError {
            host: host.clone(),
            port,
            source: std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        }
    })?;

    // Try to load switchboard config if path provided
    let switchboard_config = if let Some(path) = config_path {
        match crate::config::Config::from_toml(&PathBuf::from(path)) {
            Ok(cfg) => {
                info!("Loaded switchboard config from: {}", path);
                Some(cfg)
            }
            Err(e) => {
                warn!("Failed to load switchboard config: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Create application state with optional config
    let state = if let Some(cfg) = switchboard_config {
        let config_path = config_path.map(PathBuf::from);
        ApiState::new_arc_with_config(api_config, cfg, config_path.unwrap_or_default())
    } else {
        ApiState::new_arc(api_config)
    };

    // Create the router
    let app = create_router(state.clone());

    // Create the Axum server
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|source| ApiServerError::BindError {
            host: host.clone(),
            port,
            source,
        })?;

    info!("API HTTP server starting on {}", addr);

    // Configure graceful shutdown
    let server = axum::serve(listener, app);

    // Wait for either the server to complete or a shutdown signal
    tokio::select! {
        result = server => {
            match result {
                Ok(_) => {
                    info!("API HTTP server stopped normally");
                    Ok(())
                }
                Err(e) => {
                    error!("API HTTP server error: {}", e);
                    Err(ApiServerError::ShutdownError(e.to_string()))
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
async fn shutdown_signal() {
    use tokio::signal::ctrl_c;
    use tracing::warn;

    let ctrl_c = async {
        ctrl_c()
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

// ============================================================================
// Server Errors
// ============================================================================

/// Error types for the API HTTP server.
#[derive(Debug, thiserror::Error)]
pub enum ApiServerError {
    /// Failed to bind to the specified address.
    #[error("Failed to bind to {host}:{port}: {source}")]
    BindError {
        /// The host that was being bound to.
        host: String,
        /// The port that was being bound to.
        port: u16,
        /// The underlying IO error.
        source: std::io::Error,
    },

    /// Server was stopped unexpectedly.
    #[error("Server stopped: {0}")]
    ShutdownError(String),
}
