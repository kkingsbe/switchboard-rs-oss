//! Event types for observability
//!
//! This module provides the core event types including:
//! - EventType: enumeration of different event types
//! - EventData: enum containing payload data for different event types
//! - Event: main event struct containing id, timestamp, event_type, and payload

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::observability::error::{EventError, JsonSerializationError};

/// Information about a single git commit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Full hash of the commit
    pub hash: String,
    /// Commit message
    pub message: String,
    /// Number of files changed in this commit
    pub files_changed: u32,
    /// Number of lines inserted
    pub insertions: u32,
    /// Number of lines deleted
    pub deletions: u32,
}

/// Represents the type of event being emitted
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Agent started event
    AgentStarted,
    /// Agent stopped event
    AgentStopped,
    /// Agent error event
    AgentError,
    /// Workflow started event
    WorkflowStarted,
    /// Workflow completed event
    WorkflowCompleted,
    /// Workflow failed event
    WorkflowFailed,
    /// Task started event
    TaskStarted,
    /// Task completed event
    TaskCompleted,
    /// Task failed event
    TaskFailed,
    /// System event (e.g., startup, shutdown)
    SystemEvent,
    /// Scheduler started event
    SchedulerStarted,
    /// Scheduler stopped event
    SchedulerStopped,
    /// Container started event - emitted when Switchboard launches an agent container
    ContainerStarted,
    /// Container exited event - emitted when a container exits
    ContainerExited,
    /// Container skipped event - emitted when a cron trigger fires but the agent is skipped
    ContainerSkipped,
    /// Container queued event - emitted when a cron trigger fires and the run is queued
    ContainerQueued,
    /// Git diff event - emitted after each agent container exits
    GitDiff,
    /// Custom event type for extensibility
    Custom(String),
}

impl EventType {
    /// Validate that the event type is valid for the system
    pub fn validate(&self) -> Result<(), EventError> {
        match self {
            EventType::Custom(name) if name.is_empty() => {
                Err(EventError::ValidationError("Custom event type cannot be empty".to_string()))
            }
            _ => Ok(()),
        }
    }
}

impl Default for EventType {
    fn default() -> Self {
        EventType::SystemEvent
    }
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::AgentStarted => write!(f, "agent_started"),
            EventType::AgentStopped => write!(f, "agent_stopped"),
            EventType::AgentError => write!(f, "agent_error"),
            EventType::WorkflowStarted => write!(f, "workflow_started"),
            EventType::WorkflowCompleted => write!(f, "workflow_completed"),
            EventType::WorkflowFailed => write!(f, "workflow_failed"),
            EventType::TaskStarted => write!(f, "task_started"),
            EventType::TaskCompleted => write!(f, "task_completed"),
            EventType::TaskFailed => write!(f, "task_failed"),
            EventType::SystemEvent => write!(f, "system_event"),
            EventType::SchedulerStarted => write!(f, "scheduler_started"),
            EventType::SchedulerStopped => write!(f, "scheduler_stopped"),
            EventType::ContainerStarted => write!(f, "container_started"),
            EventType::ContainerExited => write!(f, "container_exited"),
            EventType::ContainerSkipped => write!(f, "container_skipped"),
            EventType::ContainerQueued => write!(f, "container_queued"),
            EventType::GitDiff => write!(f, "git_diff"),
            EventType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// Data payload for different event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventData {
    /// Agent event data
    Agent {
        /// Agent identifier
        agent_id: String,
        /// Agent name
        name: Option<String>,
        /// Agent metadata
        metadata: Option<serde_json::Value>,
    },
    /// Workflow event data
    Workflow {
        /// Workflow identifier
        workflow_id: String,
        /// Workflow name
        name: Option<String>,
        /// Workflow status or result
        status: Option<String>,
    },
    /// Task event data
    Task {
        /// Task identifier
        task_id: String,
        /// Task name
        name: Option<String>,
        /// Task duration in milliseconds
        duration_ms: Option<u64>,
        /// Task result or error
        result: Option<String>,
    },
    /// System event data
    System {
        /// System message
        message: String,
        /// System component
        component: Option<String>,
    },
    /// Scheduler started event data
    SchedulerStarted {
        /// List of agent names
        agents: Vec<String>,
        /// Number of agents
        agent_count: usize,
        /// Switchboard version
        version: String,
        /// Config file name
        config_file: String,
    },
    /// Scheduler stopped event data
    SchedulerStopped {
        /// Shutdown reason (e.g., "sigint", "sigterm")
        reason: String,
        /// Uptime in seconds
        uptime_seconds: u64,
    },
    /// Container started event data - emitted when Switchboard launches an agent container
    ContainerStarted {
        /// Docker image used
        image: String,
        /// Trigger type: "cron" (scheduled) or "manual"
        trigger: String,
        /// Cron schedule expression (if trigger is "cron")
        schedule: Option<String>,
        /// Docker container ID
        container_id: String,
    },
    /// Container exited event data - emitted when a container exits
    ContainerExited {
        /// Exit code from the container
        exit_code: i32,
        /// Duration of container execution in seconds
        duration_seconds: u64,
        /// Whether the container was terminated due to timeout
        timeout_hit: bool,
    },
    /// Container skipped event data - emitted when a cron trigger fires but the agent is skipped
    ContainerSkipped {
        /// Reason for skipping (e.g., "overlap_skip")
        reason: String,
        /// Run ID of the currently running container that caused this skip
        running_run_id: Option<String>,
    },
    /// Container queued event data - emitted when a cron trigger fires and the run is queued
    ContainerQueued {
        /// Position in the queue (1-based)
        queue_position: u32,
        /// Run ID of the currently running container
        running_run_id: Option<String>,
    },
    /// Git diff event data - emitted after each agent container exits
    GitDiff {
        /// Number of commits made
        commit_count: u32,
        /// List of commit information
        commits: Vec<CommitInfo>,
        /// Total lines inserted
        total_insertions: u32,
        /// Total lines deleted
        total_deletions: u32,
        /// Total files changed
        total_files_changed: u32,
    },
    /// Custom event data
    Custom {
        /// Custom event name
        event_name: String,
        /// Custom payload
        payload: serde_json::Value,
    },
}

impl EventData {
    /// Validate the event data
    pub fn validate(&self) -> Result<(), EventError> {
        match self {
            EventData::Agent { agent_id, .. } if agent_id.is_empty() => {
                Err(EventError::ValidationError("agent_id cannot be empty".to_string()))
            }
            EventData::Workflow { workflow_id, .. } if workflow_id.is_empty() => {
                Err(EventError::ValidationError("workflow_id cannot be empty".to_string()))
            }
            EventData::Task { task_id, .. } if task_id.is_empty() => {
                Err(EventError::ValidationError("task_id cannot be empty".to_string()))
            }
            EventData::Custom { event_name, .. } if event_name.is_empty() => {
                Err(EventError::ValidationError("custom event_name cannot be empty".to_string()))
            }
            EventData::SchedulerStarted { agents, version, .. } if agents.is_empty() => {
                Err(EventError::ValidationError("scheduler started event must have at least one agent".to_string()))
            }
            EventData::SchedulerStarted { version, .. } if version.is_empty() => {
                Err(EventError::ValidationError("scheduler started event must have a version".to_string()))
            }
            EventData::SchedulerStopped { reason, .. } if reason.is_empty() => {
                Err(EventError::ValidationError("scheduler stopped event must have a reason".to_string()))
            }
            EventData::ContainerStarted { image, trigger, .. } if image.is_empty() => {
                Err(EventError::ValidationError("container started event must have an image".to_string()))
            }
            EventData::ContainerStarted { trigger, .. } if trigger.is_empty() => {
                Err(EventError::ValidationError("container started event must have a trigger".to_string()))
            }
            EventData::ContainerSkipped { reason, .. } if reason.is_empty() => {
                Err(EventError::ValidationError("container skipped event must have a reason".to_string()))
            }
            _ => Ok(()),
        }
    }

    /// Create agent event data
    pub fn agent(agent_id: impl Into<String>) -> Self {
        EventData::Agent {
            agent_id: agent_id.into(),
            name: None,
            metadata: None,
        }
    }

    /// Create workflow event data
    pub fn workflow(workflow_id: impl Into<String>) -> Self {
        EventData::Workflow {
            workflow_id: workflow_id.into(),
            name: None,
            status: None,
        }
    }

    /// Create task event data
    pub fn task(task_id: impl Into<String>) -> Self {
        EventData::Task {
            task_id: task_id.into(),
            name: None,
            duration_ms: None,
            result: None,
        }
    }

    /// Create system event data
    pub fn system(message: impl Into<String>) -> Self {
        EventData::System {
            message: message.into(),
            component: None,
        }
    }

    /// Create scheduler started event data
    pub fn scheduler_started(agents: Vec<String>, version: impl Into<String>, config_file: impl Into<String>) -> Self {
        let agent_count = agents.len();
        EventData::SchedulerStarted {
            agents,
            agent_count,
            version: version.into(),
            config_file: config_file.into(),
        }
    }

    /// Create scheduler stopped event data
    pub fn scheduler_stopped(reason: impl Into<String>, uptime_seconds: u64) -> Self {
        EventData::SchedulerStopped {
            reason: reason.into(),
            uptime_seconds,
        }
    }

    /// Create container started event data
    pub fn container_started(
        image: impl Into<String>,
        trigger: impl Into<String>,
        schedule: Option<String>,
        container_id: impl Into<String>,
    ) -> Self {
        EventData::ContainerStarted {
            image: image.into(),
            trigger: trigger.into(),
            schedule,
            container_id: container_id.into(),
        }
    }

    /// Create container exited event data
    pub fn container_exited(exit_code: i32, duration_seconds: u64, timeout_hit: bool) -> Self {
        EventData::ContainerExited {
            exit_code,
            duration_seconds,
            timeout_hit,
        }
    }

    /// Create container skipped event data
    pub fn container_skipped(reason: impl Into<String>, running_run_id: Option<String>) -> Self {
        EventData::ContainerSkipped {
            reason: reason.into(),
            running_run_id,
        }
    }

    /// Create container queued event data
    pub fn container_queued(queue_position: u32, running_run_id: Option<String>) -> Self {
        EventData::ContainerQueued {
            queue_position,
            running_run_id,
        }
    }

    /// Create git diff event data
    pub fn git_diff(commits: Vec<CommitInfo>) -> Self {
        let commit_count = commits.len() as u32;
        let total_insertions: u32 = commits.iter().map(|c| c.insertions).sum();
        let total_deletions: u32 = commits.iter().map(|c| c.deletions).sum();
        let total_files_changed: u32 = commits.iter().map(|c| c.files_changed).sum();

        EventData::GitDiff {
            commit_count,
            commits,
            total_insertions,
            total_deletions,
            total_files_changed,
        }
    }
}

/// Core event structure for observability
///
/// # Fields
/// - `id`: Unique identifier for the event (UUID)
/// - `timestamp`: When the event occurred (UTC)
/// - `event_type`: Type of the event
/// - `payload`: Event data/payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for the event
    pub id: Uuid,
    /// Timestamp when the event occurred
    pub timestamp: DateTime<Utc>,
    /// Type of the event
    pub event_type: EventType,
    /// Event payload/data
    pub payload: EventData,
}

impl Event {
    /// Create a new event with a generated UUID and current timestamp
    pub fn new(event_type: EventType, payload: EventData) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type,
            payload,
        }
    }

    /// Create a new event with a specific UUID
    pub fn with_id(id: Uuid, event_type: EventType, payload: EventData) -> Self {
        Self {
            id,
            timestamp: Utc::now(),
            event_type,
            payload,
        }
    }

    /// Create a new event with a specific timestamp
    pub fn with_timestamp(
        timestamp: DateTime<Utc>,
        event_type: EventType,
        payload: EventData,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp,
            event_type,
            payload,
        }
    }

    /// Validate the event
    pub fn validate(&self) -> Result<(), EventError> {
        // Validate event type
        self.event_type.validate()?;

        // Validate payload
        self.payload.validate()?;

        Ok(())
    }

    /// Serialize the event to JSON string
    pub fn to_json(&self) -> Result<String, EventError> {
        serde_json::to_string(self).map_err(|e| EventError::SerializationError(JsonSerializationError(e)))
    }

    /// Deserialize an event from a JSON string
    pub fn from_json(json: &str) -> Result<Self, EventError> {
        serde_json::from_str(json).map_err(EventError::DeserializationError)
    }
}

impl Default for Event {
    fn default() -> Self {
        Self::new(EventType::SystemEvent, EventData::system("default event"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // ===== EventType Tests =====

    #[test]
    fn event_type_default_should_return_system_event() {
        let event_type = EventType::default();
        assert_eq!(event_type, EventType::SystemEvent);
    }

    #[test]
    fn event_type_validate_should_pass_for_standard_types() {
        let result = EventType::AgentStarted.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn event_type_validate_should_fail_for_empty_custom_type() {
        let event_type = EventType::Custom(String::new());
        let result = event_type.validate();
        assert!(result.is_err());
    }

    #[test]
    fn event_type_validate_should_pass_for_non_empty_custom_type() {
        let event_type = EventType::Custom("my_custom_event".to_string());
        let result = event_type.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn event_type_display_should_format_correctly() {
        assert_eq!(EventType::AgentStarted.to_string(), "agent_started");
        assert_eq!(EventType::WorkflowCompleted.to_string(), "workflow_completed");
        assert_eq!(EventType::Custom("test".to_string()).to_string(), "custom:test");
    }

    // ===== EventData Tests =====

    #[test]
    fn event_data_agent_should_create_valid_payload() {
        let payload = EventData::agent("agent-123");
        assert!(matches!(payload, EventData::Agent { agent_id, .. } if agent_id == "agent-123"));
    }

    #[test]
    fn event_data_workflow_should_create_valid_payload() {
        let payload = EventData::workflow("workflow-456");
        assert!(matches!(payload, EventData::Workflow { workflow_id, .. } if workflow_id == "workflow-456"));
    }

    #[test]
    fn event_data_task_should_create_valid_payload() {
        let payload = EventData::task("task-789");
        assert!(matches!(payload, EventData::Task { task_id, .. } if task_id == "task-789"));
    }

    #[test]
    fn event_data_system_should_create_valid_payload() {
        let payload = EventData::system("System started");
        assert!(matches!(payload, EventData::System { message, .. } if message == "System started"));
    }

    #[test]
    fn event_data_validate_should_fail_for_empty_agent_id() {
        let payload = EventData::Agent {
            agent_id: String::new(),
            name: None,
            metadata: None,
        };
        let result = payload.validate();
        assert!(result.is_err());
    }

    #[test]
    fn event_data_validate_should_fail_for_empty_workflow_id() {
        let payload = EventData::Workflow {
            workflow_id: String::new(),
            name: None,
            status: None,
        };
        let result = payload.validate();
        assert!(result.is_err());
    }

    #[test]
    fn event_data_validate_should_fail_for_empty_task_id() {
        let payload = EventData::Task {
            task_id: String::new(),
            name: None,
            duration_ms: None,
            result: None,
        };
        let result = payload.validate();
        assert!(result.is_err());
    }

    #[test]
    fn event_data_validate_should_fail_for_empty_custom_event_name() {
        let payload = EventData::Custom {
            event_name: String::new(),
            payload: serde_json::json!({}),
        };
        let result = payload.validate();
        assert!(result.is_err());
    }

    // ===== Event Tests =====

    #[test]
    fn event_new_should_create_event_with_generated_uuid() {
        let event = Event::new(EventType::AgentStarted, EventData::agent("test-agent"));
        
        // UUID should be valid (not nil)
        assert_ne!(event.id, Uuid::nil());
    }

    #[test]
    fn event_new_should_create_event_with_current_timestamp() {
        let before = Utc::now();
        let event = Event::new(EventType::AgentStarted, EventData::agent("test-agent"));
        let after = Utc::now();
        
        assert!(event.timestamp >= before && event.timestamp <= after);
    }

    #[test]
    fn event_with_id_should_use_provided_uuid() {
        let custom_uuid = Uuid::new_v4();
        let event = Event::with_id(
            custom_uuid,
            EventType::AgentStopped,
            EventData::agent("test-agent"),
        );
        
        assert_eq!(event.id, custom_uuid);
    }

    #[test]
    fn event_with_timestamp_should_use_provided_timestamp() {
        let custom_timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let event = Event::with_timestamp(
            custom_timestamp,
            EventType::SystemEvent,
            EventData::system("test"),
        );
        
        assert_eq!(event.timestamp, custom_timestamp);
    }

    #[test]
    fn event_validate_should_pass_for_valid_event() {
        let event = Event::new(EventType::AgentStarted, EventData::agent("valid-agent"));
        let result = event.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn event_to_json_should_serialize_correctly() {
        let event = Event::new(EventType::AgentStarted, EventData::agent("test-agent"));
        let json = event.to_json().expect("Failed to serialize event");
        
        // Verify JSON contains expected fields
        assert!(json.contains("id"));
        assert!(json.contains("timestamp"));
        assert!(json.contains("event_type"));
        assert!(json.contains("agent_started"));
        assert!(json.contains("test-agent"));
    }

    #[test]
    fn event_from_json_should_deserialize_correctly() {
        let original = Event::new(EventType::WorkflowCompleted, EventData::workflow("wf-123"));
        let json = original.to_json().expect("Failed to serialize event");
        
        let deserialized = Event::from_json(&json).expect("Failed to deserialize event");
        
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.event_type, deserialized.event_type);
    }

    #[test]
    fn event_json_serialization_roundtrip_should_preserve_all_fields() {
        let original = Event::with_id(
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            EventType::TaskCompleted,
            EventData::Task {
                task_id: "task-001".to_string(),
                name: Some("Test Task".to_string()),
                duration_ms: Some(1500),
                result: Some("success".to_string()),
            },
        );
        
        let json = original.to_json().expect("Failed to serialize");
        let deserialized = Event::from_json(&json).expect("Failed to deserialize");
        
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.timestamp, deserialized.timestamp);
        assert_eq!(original.event_type, deserialized.event_type);
        
        // Check payload fields
        if let (EventData::Task { task_id, name, duration_ms, result }, 
                EventData::Task { task_id: t2, name: n2, duration_ms: d2, result: r2 }) = 
                (&original.payload, &deserialized.payload) {
            assert_eq!(task_id, t2);
            assert_eq!(name, n2);
            assert_eq!(duration_ms, d2);
            assert_eq!(result, r2);
        } else {
            panic!("Payload types don't match");
        }
    }

    #[test]
    fn event_serialization_should_handle_all_event_types() {
        let event_types = vec![
            EventType::AgentStarted,
            EventType::AgentStopped,
            EventType::AgentError,
            EventType::WorkflowStarted,
            EventType::WorkflowCompleted,
            EventType::WorkflowFailed,
            EventType::TaskStarted,
            EventType::TaskCompleted,
            EventType::TaskFailed,
            EventType::SystemEvent,
            EventType::Custom("custom_event".to_string()),
        ];
        
        for event_type in event_types {
            let payload = EventData::system("test message");
            let event = Event::new(event_type.clone(), payload);
            let json = event.to_json().expect("Failed to serialize");
            let deserialized = Event::from_json(&json).expect("Failed to deserialize");
            assert_eq!(event_type, deserialized.event_type);
        }
    }

    #[test]
    fn event_default_should_create_valid_event() {
        let event = Event::default();
        
        assert_ne!(event.id, Uuid::nil());
        assert_eq!(event.event_type, EventType::SystemEvent);
        assert!(event.validate().is_ok());
    }

    // ===== Container Lifecycle Event Tests =====

    #[test]
    fn event_type_container_started_should_format_correctly() {
        assert_eq!(EventType::ContainerStarted.to_string(), "container_started");
    }

    #[test]
    fn event_type_container_exited_should_format_correctly() {
        assert_eq!(EventType::ContainerExited.to_string(), "container_exited");
    }

    #[test]
    fn event_type_container_skipped_should_format_correctly() {
        assert_eq!(EventType::ContainerSkipped.to_string(), "container_skipped");
    }

    #[test]
    fn event_type_container_queued_should_format_correctly() {
        assert_eq!(EventType::ContainerQueued.to_string(), "container_queued");
    }

    #[test]
    fn event_type_git_diff_should_format_correctly() {
        assert_eq!(EventType::GitDiff.to_string(), "git_diff");
    }

    #[test]
    fn event_data_container_started_should_create_valid_payload() {
        let payload = EventData::container_started(
            "kilosynth/prompter:latest",
            "cron",
            Some("*/5 * * * *".to_string()),
            "abc123",
        );
        
        if let EventData::ContainerStarted { image, trigger, schedule, container_id } = payload {
            assert_eq!(image, "kilosynth/prompter:latest");
            assert_eq!(trigger, "cron");
            assert_eq!(schedule, Some("*/5 * * * *".to_string()));
            assert_eq!(container_id, "abc123");
        } else {
            panic!("Expected ContainerStarted variant");
        }
    }

    #[test]
    fn event_data_container_started_should_validate() {
        let payload = EventData::container_started(
            "kilosynth/prompter:latest",
            "cron",
            None,
            "abc123",
        );
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn event_data_container_started_should_fail_for_empty_image() {
        let payload = EventData::container_started(
            "",
            "cron",
            None,
            "abc123",
        );
        assert!(payload.validate().is_err());
    }

    #[test]
    fn event_data_container_exited_should_create_valid_payload() {
        let payload = EventData::container_exited(0, 847, false);
        
        if let EventData::ContainerExited { exit_code, duration_seconds, timeout_hit } = payload {
            assert_eq!(exit_code, 0);
            assert_eq!(duration_seconds, 847);
            assert_eq!(timeout_hit, false);
        } else {
            panic!("Expected ContainerExited variant");
        }
    }

    #[test]
    fn event_data_container_exited_should_handle_timeout() {
        let payload = EventData::container_exited(137, 300, true);
        
        if let EventData::ContainerExited { exit_code, duration_seconds, timeout_hit } = payload {
            assert_eq!(exit_code, 137);
            assert_eq!(duration_seconds, 300);
            assert_eq!(timeout_hit, true);
        } else {
            panic!("Expected ContainerExited variant");
        }
    }

    #[test]
    fn event_data_container_skipped_should_create_valid_payload() {
        let payload = EventData::container_skipped("overlap_skip", Some("a1b2c3d4".to_string()));
        
        if let EventData::ContainerSkipped { reason, running_run_id } = payload {
            assert_eq!(reason, "overlap_skip");
            assert_eq!(running_run_id, Some("a1b2c3d4".to_string()));
        } else {
            panic!("Expected ContainerSkipped variant");
        }
    }

    #[test]
    fn event_data_container_skipped_should_validate() {
        let payload = EventData::container_skipped("overlap_skip", None);
        assert!(payload.validate().is_ok());
    }

    #[test]
    fn event_data_container_skipped_should_fail_for_empty_reason() {
        let payload = EventData::container_skipped("", Some("a1b2c3d4".to_string()));
        assert!(payload.validate().is_err());
    }

    #[test]
    fn event_data_container_queued_should_create_valid_payload() {
        let payload = EventData::container_queued(1, Some("a1b2c3d4".to_string()));
        
        if let EventData::ContainerQueued { queue_position, running_run_id } = payload {
            assert_eq!(queue_position, 1);
            assert_eq!(running_run_id, Some("a1b2c3d4".to_string()));
        } else {
            panic!("Expected ContainerQueued variant");
        }
    }

    #[test]
    fn event_data_container_queued_should_handle_higher_queue_position() {
        let payload = EventData::container_queued(5, Some("a1b2c3d4".to_string()));
        
        if let EventData::ContainerQueued { queue_position, running_run_id } = payload {
            assert_eq!(queue_position, 5);
        } else {
            panic!("Expected ContainerQueued variant");
        }
    }

    #[test]
    fn event_data_git_diff_should_create_valid_payload() {
        let commits = vec![
            CommitInfo {
                hash: "abc1234".to_string(),
                message: "feat: first commit".to_string(),
                files_changed: 4,
                insertions: 127,
                deletions: 12,
            },
            CommitInfo {
                hash: "def5678".to_string(),
                message: "test: second commit".to_string(),
                files_changed: 2,
                insertions: 89,
                deletions: 0,
            },
        ];
        
        let payload = EventData::git_diff(commits);
        
        if let EventData::GitDiff { commit_count, commits: result_commits, total_insertions, total_deletions, total_files_changed } = payload {
            assert_eq!(commit_count, 2);
            assert_eq!(result_commits.len(), 2);
            assert_eq!(total_insertions, 216);
            assert_eq!(total_deletions, 12);
            assert_eq!(total_files_changed, 6);
        } else {
            panic!("Expected GitDiff variant");
        }
    }

    #[test]
    fn event_data_git_diff_should_handle_empty_commits() {
        let commits = vec![];
        let payload = EventData::git_diff(commits);
        
        if let EventData::GitDiff { commit_count, total_insertions, total_deletions, total_files_changed, .. } = payload {
            assert_eq!(commit_count, 0);
            assert_eq!(total_insertions, 0);
            assert_eq!(total_deletions, 0);
            assert_eq!(total_files_changed, 0);
        } else {
            panic!("Expected GitDiff variant");
        }
    }

    #[test]
    fn container_events_should_serialize_and_deserialize() {
        // Test container.started
        let event = Event::new(
            EventType::ContainerStarted,
            EventData::container_started(
                "kilosynth/prompter:latest",
                "cron",
                Some("*/5 * * * *".to_string()),
                "container-123",
            ),
        );
        let json = event.to_json().expect("Failed to serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(event.event_type, deserialized.event_type);

        // Test container.exited
        let event = Event::new(
            EventType::ContainerExited,
            EventData::container_exited(0, 847, false),
        );
        let json = event.to_json().expect("Failed to serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(event.event_type, deserialized.event_type);

        // Test container.skipped
        let event = Event::new(
            EventType::ContainerSkipped,
            EventData::container_skipped("overlap_skip", Some("a1b2c3d4".to_string())),
        );
        let json = event.to_json().expect("Failed to serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(event.event_type, deserialized.event_type);

        // Test container.queued
        let event = Event::new(
            EventType::ContainerQueued,
            EventData::container_queued(1, Some("a1b2c3d4".to_string())),
        );
        let json = event.to_json().expect("Failed to serialize");
        let deserialized: Event = serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(event.event_type, deserialized.event_type);
    }
}
