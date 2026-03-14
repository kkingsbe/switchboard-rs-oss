//! Skills API handlers.
//!
//! This module provides HTTP handlers for skills management endpoints.

use crate::api::error::ApiError;
use crate::api::error::ApiResult;
use crate::api::state::ApiState;
use crate::commands::skills::install::perform_post_install_move;
use crate::commands::skills::install::extract_skill_name;
use crate::commands::skills::types::{SkillsRemove, SkillsUpdate};
use crate::commands::skills::remove::run_skills_remove;
use crate::commands::skills::update::handle_skills_update;
use crate::commands::skills::ExitCode;
use crate::skills::{
    create_npx_command, scan_global_skills, scan_project_skills, skills_sh_search,
    SkillMetadata, SkillsManager, NPX_NOT_FOUND_ERROR,
};
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

/// Skill info from registry search.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct RegistrySkillInfo {
    pub name: String,
    pub id: String,
    pub source: String,
    pub installs: u64,
}

/// Request to install a skill.
#[derive(Deserialize, Debug)]
pub struct InstallSkillRequest {
    pub name: String,
    pub source: Option<String>,
}

/// Request to update a skill.
#[derive(Deserialize, Debug)]
pub struct UpdateSkillRequest {
    pub skill_name: Option<String>,
}

/// Request to remove a skill.
#[derive(Deserialize, Debug)]
pub struct RemoveSkillRequest {
    pub skill_name: String,
    pub global: Option<bool>,
}

/// List query parameters.
#[derive(Deserialize, Debug)]
pub struct ListQuery {
    pub search: Option<String>,
    pub limit: Option<u32>,
}

/// Installed skill info for API response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct InstalledSkillInfo {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub authors: Vec<String>,
    pub source: Option<String>,
    pub global: bool,
}

impl From<SkillMetadata> for InstalledSkillInfo {
    fn from(skill: SkillMetadata) -> Self {
        Self {
            name: skill.name,
            description: skill.description,
            version: skill.version,
            authors: skill.authors,
            source: skill.source,
            global: false, // Will be set appropriately
        }
    }
}

/// ============================================================================
// Handlers
// ============================================================================

/// List available skills from the registry.
///
/// Returns a list of skills from the skills.sh registry.
#[utoipa::path(
    get,
    path = "/api/v1/skills",
    tag = "Skills",
    responses(
        (status = 200, description = "List of available skills", body = ApiResponse<Vec<RegistrySkillInfo>>)
    )
)]
pub async fn list_skills(
    State(_state): State<Arc<ApiState>>,
    Query(query): Query<ListQuery>,
) -> ApiResult<Json<ApiResponse<Vec<RegistrySkillInfo>>>> {
    // Check if npx is available
    let mut skills_manager = SkillsManager::new(None);
    if skills_manager.check_npx_available().is_err() {
        return Err(ApiError::Internal(NPX_NOT_FOUND_ERROR.to_string()));
    }

    // Use search query if provided, otherwise use default "ai"
    let search_query = query.search.unwrap_or_else(|| "ai".to_string());
    let limit = query.limit.unwrap_or(10);

    // Call the skills.sh API
    match skills_sh_search(&search_query, Some(limit)).await {
        Ok(results) => {
            let skills: Vec<RegistrySkillInfo> = results
                .into_iter()
                .map(|skill| RegistrySkillInfo {
                    name: skill.name,
                    id: skill.id,
                    source: skill.source,
                    installs: skill.installs,
                })
                .collect();

            Ok(Json(ApiResponse::success(skills)))
        }
        Err(e) => Err(ApiError::Internal(format!("Failed to search skills: {}", e))),
    }
}

/// Install a skill.
///
/// Installs a skill from the specified source.
#[utoipa::path(
    post,
    path = "/api/v1/skills",
    tag = "Skills",
    request_body = InstallSkillRequest,
    responses(
        (status = 200, description = "Skill installed", body = ApiResponse<String>),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn install_skill(
    State(_state): State<Arc<ApiState>>,
    Json(request): Json<InstallSkillRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    // Check if npx is available
    let mut skills_manager = SkillsManager::new(None);
    if skills_manager.check_npx_available().is_err() {
        return Err(ApiError::Internal(NPX_NOT_FOUND_ERROR.to_string()));
    }

    // Determine the source
    let source = request.source.unwrap_or_else(|| request.name.clone());

    // Build the install command similar to the CLI
    let skill_name = extract_skill_name(&source);
    let skills_dir = skills_manager.skills_dir.clone();
    let skill_path = skills_dir.join(&skill_name);

    // Check if destination already exists
    if skill_path.exists() {
        return Err(ApiError::BadRequest(format!(
            "Skill '{}' already exists at {}. Use update to reinstall.",
            skill_name,
            skill_path.display()
        )));
    }

    // Build and run the npx skills add command
    let mut cmd = create_npx_command();
    cmd.arg("skills").arg("add");

    // Parse source to handle @skill-name format
    if let Some(at_pos) = source.rfind('@') {
        let repo = &source[..at_pos];
        let skill_name_from_source = &source[at_pos + 1..];
        cmd.arg(repo);
        cmd.arg("--skill");
        cmd.arg(skill_name_from_source);
    } else {
        cmd.arg(&source);
    }

    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y");

    // Run the command
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return Err(ApiError::Internal(format!("Failed to execute npx skills add: {}", e)));
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::Internal(format!("Failed to install skill: {}", stderr)));
    }

    // Post-install: verify the skill in the Kilo-local skills directory
    if let Err(e) = perform_post_install_move(&skills_dir, &skill_name, &source) {
        return Err(ApiError::Internal(format!("Post-install verification failed: {}", e)));
    }

    Ok(Json(ApiResponse::success(format!(
        "Skill '{}' installed successfully",
        skill_name
    ))))
}

/// List installed skills.
///
/// Returns a list of all installed skills in both project and global scopes.
#[utoipa::path(
    get,
    path = "/api/v1/skills/installed",
    tag = "Skills",
    responses(
        (status = 200, description = "List of installed skills", body = ApiResponse<Vec<InstalledSkillInfo>>)
    )
)]
pub async fn list_installed_skills(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<Vec<InstalledSkillInfo>>>> {
    let mut all_skills: Vec<InstalledSkillInfo> = Vec::new();

    // Scan project skills
    match scan_project_skills() {
        Ok((skills, _)) => {
            for skill in skills {
                let mut info = InstalledSkillInfo::from(skill);
                info.global = false;
                all_skills.push(info);
            }
        }
        Err(e) => {
            // Log warning but continue
            tracing::warn!("Failed to scan project skills: {}", e);
        }
    }

    // Scan global skills
    match scan_global_skills() {
        Ok((skills, _)) => {
            for skill in skills {
                let mut info = InstalledSkillInfo::from(skill);
                info.global = true;
                all_skills.push(info);
            }
        }
        Err(e) => {
            // Log warning but continue
            tracing::warn!("Failed to scan global skills: {}", e);
        }
    }

    Ok(Json(ApiResponse::success(all_skills)))
}

/// Update a skill.
///
/// Updates an installed skill to its latest version.
#[utoipa::path(
    put,
    path = "/api/v1/skills/{skill_name}",
    tag = "Skills",
    responses(
        (status = 200, description = "Skill updated", body = ApiResponse<String>),
        (status = 404, description = "Skill not found")
    )
)]
pub async fn update_skill(
    State(_state): State<Arc<ApiState>>,
    Path(skill_name): Path<String>,
    Json(_request): Json<UpdateSkillRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let args = SkillsUpdate {
        skill_name: Some(skill_name.clone()),
    };

    match handle_skills_update(args, &crate::config::Config::default()).await {
        ExitCode::Success => Ok(Json(ApiResponse::success(format!(
            "Skill '{}' updated successfully",
            skill_name
        )))),
        ExitCode::Error => Err(ApiError::Internal(format!(
            "Failed to update skill '{}'",
            skill_name
        ))),
    }
}

/// Remove a skill.
///
/// Removes an installed skill.
#[utoipa::path(
    delete,
    path = "/api/v1/skills/{skill_name}",
    tag = "Skills",
    responses(
        (status = 200, description = "Skill removed", body = ApiResponse<String>),
        (status = 404, description = "Skill not found")
    )
)]
pub async fn remove_skill(
    State(_state): State<Arc<ApiState>>,
    Path(skill_name): Path<String>,
    Json(request): Json<RemoveSkillRequest>,
) -> ApiResult<Json<ApiResponse<String>>> {
    let global = request.global.unwrap_or(false);
    let args = SkillsRemove {
        skill_name: skill_name.clone(),
        global,
        yes: true, // Skip confirmation for API
    };

    match run_skills_remove(args, &crate::config::Config::default()).await {
        ExitCode::Success => Ok(Json(ApiResponse::success(format!(
            "Skill '{}' removed successfully",
            skill_name
        )))),
        ExitCode::Error => Err(ApiError::Internal(format!(
            "Failed to remove skill '{}'",
            skill_name
        ))),
    }
}

/// ============================================================================
// Helper Functions
// ============================================================================

/// Cleans up empty legacy `.agents/skills/` and `.agents/` directories.
#[allow(dead_code)]
fn cleanup_agents_directory() -> Result<(), String> {
    use std::fs;

    let agents_skills_dir = PathBuf::from(".agents/skills");
    let agents_dir = PathBuf::from(".agents");

    // Remove .agents/skills/ if it exists and is empty
    if agents_skills_dir.exists() {
        if let Ok(mut entries) = fs::read_dir(&agents_skills_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&agents_skills_dir);
            }
        }
    }

    // Remove .agents/ if it exists and is empty
    if agents_dir.exists() {
        if let Ok(mut entries) = fs::read_dir(&agents_dir) {
            if entries.next().is_none() {
                let _ = fs::remove_dir(&agents_dir);
            }
        }
    }

    Ok(())
}
