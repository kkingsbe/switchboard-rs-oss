//! Workflows API handlers.
//!
//! This module provides HTTP handlers for workflows management endpoints.

use crate::api::error::ApiError;
use crate::api::error::ApiResult;
use crate::api::state::ApiState;
use crate::commands::workflows::types::{WorkflowsApply, WorkflowsInstall, WorkflowsRemove, WorkflowsUpdate, WorkflowsValidate};
use crate::commands::workflows::{apply, install, remove, update, validate};
use crate::commands::workflows::ExitCode;
use crate::commands::workflows::installed::WorkflowsLockfile;
use crate::config::Config;
use crate::workflows::github::GitHubClient;
use crate::workflows::metadata::scan_workflows_directory;
use crate::workflows::WorkflowsError;
use crate::workflows::WORKFLOWS_DIR;
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

/// Workflow info from registry.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct RegistryWorkflowInfo {
    pub name: String,
    pub description: Option<String>,
    pub prompts_count: usize,
}

/// Installed workflow info.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct InstalledWorkflowInfo {
    pub name: String,
    pub description: Option<String>,
    pub prompts: Vec<String>,
    pub installed_at: Option<String>,
    pub source: Option<String>,
}

/// Request to install a workflow.
#[derive(Deserialize, Debug)]
pub struct InstallWorkflowRequest {
    pub workflow_name: String,
    pub yes: Option<bool>,
}

/// Request to validate a workflow.
#[derive(Deserialize, Debug)]
pub struct ValidateWorkflowRequest {
    pub workflow_name: String,
}

/// Request to apply a workflow.
#[derive(Deserialize, Debug)]
pub struct ApplyWorkflowRequest {
    pub workflow_name: String,
    pub prefix: Option<String>,
    pub output: Option<String>,
    pub append: Option<bool>,
    pub yes: Option<bool>,
    pub dry_run: Option<bool>,
}

/// Request to update a workflow.
#[derive(Deserialize, Debug)]
pub struct UpdateWorkflowRequest {
    pub workflow_name: Option<String>,
}

/// Request to remove a workflow.
#[derive(Deserialize, Debug)]
pub struct RemoveWorkflowRequest {
    pub workflow_name: String,
    pub yes: Option<bool>,
}

/// List query parameters.
#[derive(Deserialize, Debug)]
pub struct ListQuery {
    pub search: Option<String>,
    pub limit: Option<u32>,
}

/// Validate response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct ValidateResponse {
    pub valid: bool,
    pub errors: Vec<String>,
    pub workflow_name: String,
}

/// Apply response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct ApplyResponse {
    pub success: bool,
    pub output_path: Option<String>,
    pub agents_created: usize,
    pub message: String,
}

/// Workflow lockfile entry.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct WorkflowLockEntry {
    #[serde(rename = "workflow_name")]
    pub workflow_name: String,
    pub source: String,
    #[serde(rename = "installed_at")]
    pub installed_at: String,
}

/// ============================================================================
// Handlers
// ============================================================================

/// List available workflows from the registry.
///
/// Returns a list of workflows from the switchboard-workflows repository.
#[utoipa::path(
    get,
    path = "/api/v1/workflows",
    tag = "Workflows",
    responses(
        (status = 200, description = "List of available workflows", body = ApiResponse<Vec<RegistryWorkflowInfo>>)
    )
)]
pub async fn list_workflows(
    State(_state): State<Arc<ApiState>>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<ApiResponse<Vec<RegistryWorkflowInfo>>>> {
    // Create GitHub client
    let client = GitHubClient::new();

    // Get list of workflow names from GitHub
    let workflow_names = match client.list_workflows().await {
        Ok(names) => names,
        Err(e) => {
            return Err(ApiError::Internal(format!("Failed to fetch workflows: {}", e)));
        }
    };

    // Fetch info for each workflow
    let mut workflows = Vec::new();
    for name in workflow_names {
        match client.get_workflow_info(&name).await {
            Ok(info) => workflows.push(RegistryWorkflowInfo {
                name: info.name,
                description: info.description,
                prompts_count: info.prompts.len(),
            }),
            Err(WorkflowsError::NotFound(_)) => {
                // Skip workflows that don't have info
            }
            Err(e) => {
                tracing::warn!("Failed to get info for workflow '{}': {}", name, e);
            }
        }
    }

    // Filter by search query if provided
    if let Some(ref search) = query.search {
        let search_lower = search.to_lowercase();
        workflows.retain(|w| w.name.to_lowercase().contains(&search_lower));
    }

    // Limit results if limit is provided
    if let Some(limit) = query.limit {
        let limit_usize = limit as usize;
        if workflows.len() > limit_usize {
            workflows.truncate(limit_usize);
        }
    }

    Ok(Json(ApiResponse::success(workflows)))
}

/// Install a workflow.
///
/// Installs a workflow from the registry.
#[utoipa::path(
    post,
    path = "/api/v1/workflows",
    tag = "Workflows",
    request_body = InstallWorkflowRequest,
    responses(
        (status = 200, description = "Workflow installed", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn install_workflow(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<InstallWorkflowRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let args = WorkflowsInstall {
        workflow_name: request.workflow_name.clone(),
        yes: request.yes.unwrap_or(false),
    };

    match install::run_workflows_install(args, &Config::default()).await {
        ExitCode::Success => Ok(Json(ApiResponse::success(format!(
            "Workflow '{}' installed successfully",
            request.workflow_name
        )))),
        ExitCode::Error => Err(ApiError::Internal(format!(
            "Failed to install workflow '{}'",
            request.workflow_name
        ))),
    }
}

/// List installed workflows.
///
/// Returns a list of all installed workflows.
#[utoipa::path(
    get,
    path = "/api/v1/workflows/installed",
    tag = "Workflows",
    responses(
        (status = 200, description = "List of installed workflows", body = ApiResponse<Vec<InstalledWorkflowInfo>>)
    )
)]
pub async fn list_installed_workflows(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<Vec<InstalledWorkflowInfo>>>> {
    let workflows_dir = PathBuf::from(WORKFLOWS_DIR);

    // Scan the workflows directory
    let workflows = match scan_workflows_directory(&workflows_dir) {
        Ok(workflows) => workflows,
        Err(e) => {
            return Err(ApiError::Internal(format!(
                "Failed to scan workflows directory: {}",
                e
            )));
        }
    };

    // Read lockfile for installation timestamps
    let lockfile = read_workflows_lockfile(&workflows_dir);

    // Convert to API response format
    let installed_workflows: Vec<InstalledWorkflowInfo> = workflows
        .into_iter()
        .map(|w| {
            let lock_entry = lockfile
                .as_ref()
                .and_then(|lf| lf.workflows.get(&w.name));

            InstalledWorkflowInfo {
                name: w.name,
                description: Some(w.description),
                prompts: w.prompts,
                installed_at: lock_entry.map(|e| e.installed_at.clone()),
                source: lock_entry.map(|e| e.source.clone()),
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(installed_workflows)))
}

/// Update a workflow.
///
/// Updates an installed workflow to its latest version.
#[utoipa::path(
    put,
    path = "/api/v1/workflows/{workflow_name}",
    tag = "Workflows",
    responses(
        (status = 200, description = "Workflow updated", body = ApiResponse<String>),
        (status = 404, description = "Workflow not found")
    )
)]
pub async fn update_workflow(
    State(_state): State<Arc<ApiState>>,
    Path(workflow_name): Path<String>,
    Json(_request): Json<UpdateWorkflowRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let args = WorkflowsUpdate {
        workflow_name: Some(workflow_name.clone()),
    };

    match update::handle_workflows_update(args, &Config::default()).await {
        ExitCode::Success => Ok(Json(ApiResponse::success(format!(
            "Workflow '{}' updated successfully",
            workflow_name
        )))),
        ExitCode::Error => Err(ApiError::Internal(format!(
            "Failed to update workflow '{}'",
            workflow_name
        ))),
    }
}

/// Remove a workflow.
///
/// Removes an installed workflow.
#[utoipa::path(
    delete,
    path = "/api/v1/workflows/{workflow_name}",
    tag = "Workflows",
    responses(
        (status = 200, description = "Workflow removed", body = ApiResponse<String>),
        (status = 404, description = "Workflow not found")
    )
)]
pub async fn remove_workflow(
    State(_state): State<Arc<ApiState>>,
    Path(workflow_name): Path<String>,
    Json(request): Json<RemoveWorkflowRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let args = WorkflowsRemove {
        workflow_name: workflow_name.clone(),
        yes: request.yes.unwrap_or(false),
    };

    match remove::run_workflows_remove(args, &Config::default()).await {
        ExitCode::Success => Ok(Json(ApiResponse::success(format!(
            "Workflow '{}' removed successfully",
            workflow_name
        )))),
        ExitCode::Error => Err(ApiError::Internal(format!(
            "Failed to remove workflow '{}'",
            workflow_name
        ))),
    }
}

/// Validate a workflow.
///
/// Validates a workflow's manifest.toml file.
#[utoipa::path(
    post,
    path = "/api/v1/workflows/validate",
    tag = "Workflows",
    request_body = ValidateWorkflowRequest,
    responses(
        (status = 200, description = "Validation result", body = ApiResponse<ValidateResponse>)
    )
)]
pub async fn validate_workflow(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<ValidateWorkflowRequest>,
) -> ApiResult<Json<ApiResponse<ValidateResponse>>> {
    let args = WorkflowsValidate {
        workflow_name: request.workflow_name.clone(),
    };

    match validate::run_workflows_validate(args, &Config::default()).await {
        ExitCode::Success => Ok(Json(ApiResponse::success(ValidateResponse {
            valid: true,
            errors: vec![],
            workflow_name: request.workflow_name,
        }))),
        ExitCode::Error => Ok(Json(ApiResponse::success(ValidateResponse {
            valid: false,
            errors: vec!["Validation failed. Check workflow manifest.".to_string()],
            workflow_name: request.workflow_name,
        }))),
    }
}

/// Apply a workflow.
///
/// Applies a workflow's manifest to generate switchboard.toml entries.
#[utoipa::path(
    post,
    path = "/api/v1/workflows/apply",
    tag = "Workflows",
    request_body = ApplyWorkflowRequest,
    responses(
        (status = 200, description = "Workflow applied", body = ApiResponse<ApplyResponse>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn apply_workflow(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<ApplyWorkflowRequest>,
) -> ApiResult<Json<ApiResponse<ApplyResponse>>> {
    let args = WorkflowsApply {
        workflow_name: request.workflow_name.clone(),
        prefix: request.prefix,
        output: request.output.clone(),
        append: request.append.unwrap_or(false),
        yes: request.yes.unwrap_or(false),
        dry_run: request.dry_run.unwrap_or(false),
    };

    match apply::run_workflows_apply(args, &Config::default()).await {
        ExitCode::Success => {
            let output_path = request.output.unwrap_or_else(|| "switchboard.toml".to_string());
            Ok(Json(ApiResponse::success(ApplyResponse {
                success: true,
                output_path: Some(output_path),
                agents_created: 0, // Would need to parse output to get exact count
                message: format!("Workflow '{}' applied successfully", request.workflow_name),
            })))
        }
        ExitCode::Error => Err(ApiError::Internal(format!(
            "Failed to apply workflow '{}'",
            request.workflow_name
        ))),
    }
}

/// ============================================================================
// Helper Functions
// ============================================================================

/// Reads the workflows lockfile from the workflows directory.
fn read_workflows_lockfile(workflows_dir: &PathBuf) -> Option<WorkflowsLockfile> {
    use std::fs;

    let lockfile_path = workflows_dir.join("workflows.lock.json");

    if !lockfile_path.exists() {
        return None;
    }

    match fs::read_to_string(&lockfile_path) {
        Ok(contents) => serde_json::from_str(&contents).ok(),
        Err(_) => None,
    }
}
