//! Agent API handlers.
//!
//! This module provides HTTP handlers for agent management endpoints.

use crate::api::error::ApiError;
use crate::api::error::ApiResult;
use crate::api::state::ApiState;
use crate::config::Agent;
use crate::metrics::{AllMetrics, MetricsStore};
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader};
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

/// Agent info response for API.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct AgentInfo {
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub readonly: Option<bool>,
    pub timeout: Option<String>,
    pub skills: Option<Vec<String>>,
    pub prompt_preview: Option<String>,
}

impl From<&Agent> for AgentInfo {
    fn from(agent: &Agent) -> Self {
        let enabled = !agent.schedule.is_empty();
        let prompt_preview = agent.prompt.as_ref().map(|p| {
            if p.len() > 100 {
                format!("{}...", &p[..100])
            } else {
                p.clone()
            }
        });

        Self {
            name: agent.name.clone(),
            schedule: agent.schedule.clone(),
            enabled,
            readonly: agent.readonly.clone(),
            timeout: agent.timeout.clone(),
            skills: agent.skills.clone(),
            prompt_preview,
        }
    }
}

/// Detailed agent info response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct AgentDetail {
    pub name: String,
    pub schedule: String,
    pub enabled: bool,
    pub readonly: Option<bool>,
    pub timeout: Option<String>,
    pub skills: Option<Vec<String>>,
    pub prompt: Option<String>,
    pub prompt_file: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub overlap_mode: Option<String>,
}

impl From<&Agent> for AgentDetail {
    fn from(agent: &Agent) -> Self {
        let enabled = !agent.schedule.is_empty();
        let overlap_mode = agent.overlap_mode.as_ref().map(|m| format!("{:?}", m));

        Self {
            name: agent.name.clone(),
            schedule: agent.schedule.clone(),
            enabled,
            readonly: agent.readonly.clone(),
            timeout: agent.timeout.clone(),
            skills: agent.skills.clone(),
            prompt: agent.prompt.clone(),
            prompt_file: agent.prompt_file.clone(),
            env: agent.env.clone(),
            overlap_mode,
        }
    }
}

/// Agent run response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct AgentRunResponse {
    pub operation_id: String,
    pub agent_name: String,
    pub status: String,
}

/// Logs query parameters.
#[derive(Deserialize, Debug)]
pub struct LogsQuery {
    #[serde(default = "default_tail")]
    pub tail: usize,
    pub since: Option<String>,
}

fn default_tail() -> usize {
    50
}

/// Logs response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct LogsResponse {
    pub agent_name: String,
    pub lines: Vec<String>,
    pub total_lines: usize,
}

/// Metrics summary response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct MetricsSummary {
    pub agents: Vec<AgentMetricsSummary>,
}

#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct AgentMetricsSummary {
    pub name: String,
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub success_rate: f64,
    pub avg_duration_ms: u64,
    pub last_run: Option<i64>,
}

/// Scheduler status response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct SchedulerStatus {
    pub running: bool,
    pub instance_id: String,
    pub uptime_seconds: Option<u64>,
}

/// Shutdown response.
#[derive(Serialize, Deserialize, Debug)]
#[derive(utoipa::ToSchema)]
pub struct ShutdownResponse {
    pub message: String,
}

/// ============================================================================
// Handlers
// ============================================================================

/// List all agents.
///
/// Returns a list of all configured agents with their basic information.
#[utoipa::path(
    get,
    path = "/api/v1/agents",
    tag = "Agents",
    responses(
        (status = 200, description = "List of agents", body = ApiResponse<Vec<AgentInfo>>)
    )
)]
pub async fn list_agents(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<Vec<AgentInfo>>>> {
    let config = state
        .switchboard_config
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Switchboard config not loaded".to_string()))?;

    let agents: Vec<AgentInfo> = config.agents.iter().map(AgentInfo::from).collect();

    Ok(Json(ApiResponse::success(agents)))
}

/// Get agent details.
///
/// Returns detailed information about a specific agent.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_name}",
    tag = "Agents",
    responses(
        (status = 200, description = "Agent details", body = ApiResponse<AgentDetail>),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn get_agent(
    State(state): State<Arc<ApiState>>,
    Path(agent_name): Path<String>,
) -> ApiResult<Json<ApiResponse<AgentDetail>>> {
    let config = state
        .switchboard_config
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Switchboard config not loaded".to_string()))?;

    let agent = config
        .agents
        .iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", agent_name)))?;

    Ok(Json(ApiResponse::success(AgentDetail::from(agent))))
}

/// Run agent immediately.
///
/// Triggers an immediate run of the specified agent.
/// Returns an operation ID that can be used to track status.
#[utoipa::path(
    post,
    path = "/api/v1/agents/{agent_name}/run",
    tag = "Agents",
    responses(
        (status = 200, description = "Agent run initiated", body = ApiResponse<AgentRunResponse>),
        (status = 404, description = "Agent not found")
    )
)]
pub async fn run_agent(
    State(state): State<Arc<ApiState>>,
    Path(agent_name): Path<String>,
) -> ApiResult<Json<ApiResponse<AgentRunResponse>>> {
    let config = state
        .switchboard_config
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Switchboard config not loaded".to_string()))?;

    // Verify agent exists
    let _agent = config
        .agents
        .iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", agent_name)))?;

    // Generate operation ID
    let operation_id = uuid::Uuid::new_v4().to_string();

    // TODO: Implement actual agent run trigger
    // For now, return the operation ID with a pending status
    // In a full implementation, this would:
    // 1. Queue the agent run with the scheduler
    // 2. Or spawn a new task to run the agent directly

    Ok(Json(ApiResponse::success(AgentRunResponse {
        operation_id,
        agent_name,
        status: "queued".to_string(),
    })))
}

/// Get agent logs.
///
/// Returns log content for a specific agent.
#[utoipa::path(
    get,
    path = "/api/v1/agents/{agent_name}/logs",
    tag = "Agents",
    responses(
        (status = 200, description = "Agent logs", body = ApiResponse<LogsResponse>),
        (status = 404, description = "Agent not found or no logs")
    )
)]
pub async fn get_agent_logs(
    State(state): State<Arc<ApiState>>,
    Path(agent_name): Path<String>,
    Query(query): Query<LogsQuery>,
) -> ApiResult<Json<ApiResponse<LogsResponse>>> {
    let config = state
        .switchboard_config
        .as_ref()
        .ok_or_else(|| ApiError::NotFound("Switchboard config not loaded".to_string()))?;

    // Verify agent exists
    let _agent = config
        .agents
        .iter()
        .find(|a| a.name == agent_name)
        .ok_or_else(|| ApiError::NotFound(format!("Agent '{}' not found", agent_name)))?;

    // Get log directory
    let log_dir = state.log_dir();
    let agent_log_dir = log_dir.join(&agent_name);

    // Find the most recent log file
    let log_files = find_log_files(&agent_log_dir)?;
    let log_path = log_files.first().ok_or_else(|| {
        ApiError::NotFound(format!("No log files found for agent '{}'", agent_name))
    })?;

    // Read log content
    let lines = read_log_file(log_path, Some(query.tail))?;

    Ok(Json(ApiResponse::success(LogsResponse {
        agent_name,
        lines,
        total_lines: 0, // Would need to read full file to get this
    })))
}

/// Get metrics.
///
/// Returns execution metrics for all agents.
#[utoipa::path(
    get,
    path = "/api/v1/metrics",
    tag = "Metrics",
    responses(
        (status = 200, description = "Metrics summary", body = ApiResponse<MetricsSummary>)
    )
)]
pub async fn get_metrics(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<MetricsSummary>>> {
    let log_dir = state.log_dir();
    let store = MetricsStore::new(log_dir);

    let all_metrics = match store.load() {
        Ok(m) => m,
        Err(crate::metrics::MetricsError::FileNotFound(_)) => {
            // Return empty metrics if file doesn't exist
            return Ok(Json(ApiResponse::success(MetricsSummary {
                agents: vec![],
            })));
        }
        Err(e) => {
            return Err(ApiError::Internal(format!(
                "Failed to load metrics: {}",
                e
            )));
        }
    };

    let agent_summaries = transform_metrics(&all_metrics);

    Ok(Json(ApiResponse::success(MetricsSummary {
        agents: agent_summaries,
    })))
}

/// Get scheduler status.
///
/// Returns the current status of the scheduler.
#[utoipa::path(
    get,
    path = "/api/v1/status",
    tag = "Status",
    responses(
        (status = 200, description = "Scheduler status", body = ApiResponse<SchedulerStatus>)
    )
)]
pub async fn get_status(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<SchedulerStatus>>> {
    // TODO: Implement actual scheduler status check
    // For now, return a basic status
    // In a full implementation, this would check if the scheduler process is running

    Ok(Json(ApiResponse::success(SchedulerStatus {
        running: false, // API doesn't directly manage scheduler
        instance_id: state.instance_id.clone(),
        uptime_seconds: None,
    })))
}

/// Shutdown scheduler and containers.
///
/// Gracefully stops the scheduler and any running containers.
#[utoipa::path(
    post,
    path = "/api/v1/shutdown",
    tag = "Control",
    responses(
        (status = 200, description = "Shutdown initiated", body = ApiResponse<ShutdownResponse>)
    )
)]
pub async fn shutdown(
    State(_state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<ShutdownResponse>>> {
    // TODO: Implement graceful shutdown
    // This would need to:
    // 1. Signal the scheduler to stop
    // 2. Wait for running containers to complete
    // 3. Stop any remaining containers

    Ok(Json(ApiResponse::success(ShutdownResponse {
        message: "Shutdown signal sent".to_string(),
    })))
}

/// ============================================================================
// Helper Functions
// ============================================================================

/// Find log files in a directory, sorted by modification time (newest first).
fn find_log_files(dir: &PathBuf) -> ApiResult<Vec<PathBuf>> {
    if !dir.exists() || !dir.is_dir() {
        return Ok(vec![]);
    }

    let mut files: Vec<(PathBuf, std::time::SystemTime)> = vec![];

    let entries = std::fs::read_dir(dir).map_err(|e| ApiError::Internal(e.to_string()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("log") {
            if let Ok(metadata) = std::fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    files.push((path, modified));
                }
            }
        }
    }

    // Sort by modification time (newest first)
    files.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(files.into_iter().map(|(p, _)| p).collect())
}

/// Read lines from a log file.
fn read_log_file(path: &PathBuf, tail: Option<usize>) -> ApiResult<Vec<String>> {
    let file = File::open(path).map_err(|e| ApiError::NotFound(e.to_string()))?;
    let reader = BufReader::new(file);

    let all_lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .collect();

    match tail {
        Some(n) if n > 0 => {
            let start = if all_lines.len() > n {
                all_lines.len() - n
            } else {
                0
            };
            Ok(all_lines[start..].to_vec())
        }
        _ => Ok(all_lines),
    }
}

/// Transform metrics data to API response format.
fn transform_metrics(all_metrics: &AllMetrics) -> Vec<AgentMetricsSummary> {
    all_metrics
        .agents
        .iter()
        .map(|(name, data)| {
            let total_runs = data.total_runs;
            let success_rate = if total_runs > 0 {
                ((total_runs - data.failed_runs) as f64 / total_runs as f64) * 100.0
            } else {
                0.0
            };

            let avg_duration_ms = if total_runs > 0 {
                data.total_duration_ms / total_runs
            } else {
                0
            };

            let last_run = data.runs.iter().max_by_key(|r| r.timestamp).map(|r| r.timestamp as i64);

            AgentMetricsSummary {
                name: name.clone(),
                total_runs,
                successful_runs: data.successful_runs,
                failed_runs: data.failed_runs,
                success_rate,
                avg_duration_ms,
                last_run,
            }
        })
        .collect()
}
