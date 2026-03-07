//! Project & Workflow Init API handlers.
//!
//! This module provides HTTP handlers for project initialization and workflow initialization endpoints.

use crate::api::error::ApiError;
use crate::api::error::ApiResult;
use crate::api::state::ApiState;
use crate::commands::project::types::ProjectInit;
use crate::commands::project::init::run_project_init;
use crate::commands::project::types::ExitCode as ProjectExitCode;
use crate::commands::workflow_init::types::WorkflowInit;
use crate::commands::workflow_init::init::run_workflow_init;
use crate::commands::workflow_init::types::ExitCode as WorkflowExitCode;
use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use utoipa::ToSchema;

/// Generic API response wrapper.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message.into()),
        }
    }
}

/// Request to initialize a project.
#[derive(Deserialize, Debug)]
pub struct ProjectInitRequest {
    pub path: Option<String>,
    pub name: Option<String>,
    pub force: Option<bool>,
    pub minimal: Option<bool>,
}

/// Response for project init.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct ProjectInitResponse {
    pub path: String,
    pub name: String,
    pub message: String,
}

/// Request to initialize a workflow.
#[derive(Deserialize, Debug)]
pub struct WorkflowInitRequest {
    pub name: String,
    pub agents: Option<String>,
    pub schedule: Option<String>,
    pub path: Option<String>,
}

/// Response for workflow init.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct WorkflowInitResponse {
    pub name: String,
    pub path: String,
    pub message: String,
}

/// ============================================================================
// Handlers
// ============================================================================

/// Initialize a new project.
///
/// Creates a new Switchboard project with the standard directory structure.
#[utoipa::path(
    post,
    path = "/api/v1/project/init",
    tag = "Project",
    request_body = ProjectInitRequest,
    responses(
        (status = 200, description = "Project initialized", body = ApiResponse<ProjectInitResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn init_project(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<ProjectInitRequest>,
) -> ApiResult<Json<ApiResponse<ProjectInitResponse>>> {
    // Clone values before moving into args
    let path = request.path.clone().unwrap_or_else(|| ".".to_string());
    let name = request.name.clone();
    
    let args = ProjectInit {
        path: path.clone(),
        name: name.clone(),
        force: request.force.unwrap_or(false),
        minimal: request.minimal.unwrap_or(false),
    };

    match run_project_init(args).await {
        ProjectExitCode::Success => {
            let project_name = name.unwrap_or_else(|| "switchboard-project".to_string());
            let project_path = path;
            
            Ok(Json(ApiResponse::success(ProjectInitResponse {
                path: project_path.clone(),
                name: project_name.clone(),
                message: format!(
                    "Project '{}' initialized successfully at {}",
                    project_name, project_path
                ),
            })))
        }
        ProjectExitCode::Error => Err(ApiError::Internal(
            "Failed to initialize project".to_string(),
        )),
    }
}

/// Initialize a new workflow.
///
/// Creates a new Switchboard workflow with the standard directory structure.
#[utoipa::path(
    post,
    path = "/api/v1/workflow/init",
    tag = "Workflow Init",
    request_body = WorkflowInitRequest,
    responses(
        (status = 200, description = "Workflow initialized", body = ApiResponse<WorkflowInitResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn init_workflow(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<WorkflowInitRequest>,
) -> ApiResult<Json<ApiResponse<WorkflowInitResponse>>> {
    // Clone values before moving into args
    let workflow_name = request.name.clone();
    let path = request.path.clone().unwrap_or_else(|| ".".to_string());
    
    let args = WorkflowInit {
        name: workflow_name.clone(),
        agents: request.agents,
        schedule: request.schedule,
        path: path.clone(),
    };

    match run_workflow_init(args).await {
        WorkflowExitCode::Success => {
            let workflow_path = path;
            
            Ok(Json(ApiResponse::success(WorkflowInitResponse {
                name: workflow_name.clone(),
                path: format!("{}/{}", workflow_path, workflow_name),
                message: format!(
                    "Workflow '{}' initialized successfully",
                    workflow_name
                ),
            })))
        }
        WorkflowExitCode::Error => Err(ApiError::Internal(
            format!("Failed to initialize workflow '{}'", workflow_name),
        )),
    }
}
