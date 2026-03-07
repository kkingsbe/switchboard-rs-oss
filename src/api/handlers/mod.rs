//! API handlers module.
//!
//! This module contains HTTP handlers for the REST API endpoints.

pub mod agents;
pub mod skills;
pub mod workflows;
pub mod projects;
pub mod scheduler;
pub mod gateway;

// Re-export specific items to avoid ambiguous glob re-exports
pub use agents::{
    ApiResponse as AgentsApiResponse,
    list_agents,
    get_agent,
    run_agent,
    get_agent_logs,
    get_metrics,
    get_status,
    shutdown,
};
pub use skills::{
    ApiResponse as SkillsApiResponse,
    list_skills,
    install_skill,
    list_installed_skills,
    update_skill,
    remove_skill,
};
pub use workflows::{
    ApiResponse as WorkflowsApiResponse,
    list_workflows,
    install_workflow,
    list_installed_workflows,
    update_workflow,
    remove_workflow,
    validate_workflow,
    apply_workflow,
};
pub use projects::{
    ApiResponse as ProjectsApiResponse,
    init_project,
    init_workflow,
};
pub use gateway::{
    ApiResponse as GatewayApiResponse,
    gateway_up,
    gateway_status,
    gateway_down,
};
pub use scheduler::{
    ApiResponse as SchedulerApiResponse,
    SchedulerStartRequest,
    SchedulerStartResponse,
    SchedulerStopRequest,
    SchedulerStopResponse,
    SchedulerStatusResponse,
    scheduler_up,
    scheduler_down,
    scheduler_status,
    scheduler_restart,
};
