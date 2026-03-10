//! Switchboard REST API Module.
//!
//! This module provides the HTTP REST API for Switchboard local IPC.
//! It includes endpoints for health checks, configuration validation,
//! agents, skills, workflows, project/workflow initialization, and gateway management.
//!
//! # Features
//!
//! - `health` - Health check endpoint at `/health`
//! - `validate` - Configuration validation at `/api/v1/validate`
//! - `agents` - Agents management at `/api/v1/agents`
//! - `metrics` - Execution metrics at `/api/v1/metrics`
//! - `status` - Scheduler status at `/api/v1/status`
//! - `shutdown` - Shutdown endpoint at `/api/v1/shutdown`
//! - `skills` - Skills management at `/api/v1/skills`
//!   - GET /api/v1/skills - List available skills from registry
//!   - POST /api/v1/skills - Install a skill
//!   - GET /api/v1/skills/installed - List installed skills
//!   - PUT /api/v1/skills/:skill_name - Update a skill
//!   - DELETE /api/v1/skills/:skill_name - Remove a skill
//! - `workflows` - Workflows management at `/api/v1/workflows`
//!   - GET /api/v1/workflows - List available workflows
//!   - POST /api/v1/workflows - Install a workflow
//!   - GET /api/v1/workflows/installed - List installed workflows
//!   - PUT /api/v1/workflows/:workflow_name - Update a workflow
//!   - DELETE /api/v1/workflows/:workflow_name - Remove a workflow
//!   - POST /api/v1/workflows/validate - Validate a workflow
//!   - POST /api/v1/workflows/apply - Apply a workflow
//! - `project` - Project initialization at `/api/v1/project/init`
//! - `workflow_init` - Workflow initialization at `/api/v1/workflow/init`
//! - `gateway` - Gateway management at `/api/v1/gateway/*` (requires `gateway` feature)
//!   - POST /api/v1/gateway/up - Start gateway
//!   - GET /api/v1/gateway/status - Get gateway status
//!   - POST /api/v1/gateway/down - Stop gateway
//! - `scheduler` - Scheduler management at `/api/v1/scheduler/*`
//!   - POST /api/v1/scheduler/up - Start scheduler
//!   - POST /api/v1/scheduler/down - Stop scheduler
//!   - POST /api/v1/scheduler/restart - Restart scheduler
//!   - GET /api/v1/scheduler/status - Get scheduler status
//!
//! # Usage
//!
//! The API module requires the `api` feature to be enabled:
//!
//! ```toml
//! [dependencies]
//! switchboard = { version = "0.1", features = ["api"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use switchboard::config::ApiConfig;
//! use switchboard::api::router::serve;
//!
//! async fn start_api() {
//!     let config = ApiConfig::default();
//!     serve(config).await.unwrap();
//! }
//! ```

pub mod error;
pub mod handlers;
pub mod registry;
pub mod router;
pub mod state;

// Re-export types for external use
pub use error::{ApiError, ApiErrorResponse, ApiResult};
pub use registry::{
    get_instance_dir, get_instance_log_dir, get_instance_metrics_file,
    get_instance_pid_file, derive_instance_id_from_config, ensure_instance_dirs,
    InstanceRegistration, InstanceRegistry, InstanceStatus, RegistryError,
};
pub use router::{create_router, serve, serve_with_config, ApiServerError};
pub use state::ApiState;
