//! Unit tests for the observability module.
//!
//! These tests validate the Event struct, EventData enum, and EventEmitter
//! following the TDD methodology - tests are written first.

use chrono::{DateTime, Utc};
use serde_json::json;
use std::fs;
use tempfile::NamedTempFile;

use crate::observability::emitter::EventEmitter;
use crate::observability::event::{Event, EventData};

/// Test EventData serialization to JSON for Scheduler variant
#[test]
fn test_event_data_scheduler_serialization() {
    let event_data = EventData::Scheduler {
        task_id: "task-123".to_string(),
        status: "running".to_string(),
    };
    
    let json = serde_json::to_string(&event_data).expect("Failed to serialize");
    assert!(json.contains("task_id"));
    assert!(json.contains("task-123"));
    assert!(json.contains("status"));
    assert!(json.contains("running"));
}

/// Test EventData deserialization from JSON for Scheduler variant
#[test]
fn test_event_data_scheduler_deserialization() {
    let json = r#"{
        "type": "Scheduler",
        "payload": {
            "task_id": "task-456",
            "status": "completed"
        }
    }"#;
    
    let event_data: EventData = serde_json::from_str(json).expect("Failed to deserialize");
    match event_data {
        EventData::Scheduler { task_id, status } => {
            assert_eq!(task_id, "task-456");
            assert_eq!(status, "completed");
        }
        _ => panic!("Expected Scheduler variant"),
    }
}

/// Test EventData serialization to JSON for Container variant
#[test]
fn test_event_data_container_serialization() {
    let event_data = EventData::Container {
        container_id: "container-abc".to_string(),
        action: "start".to_string(),
    };
    
    let json = serde_json::to_string(&event_data).expect("Failed to serialize");
    assert!(json.contains("container_id"));
    assert!(json.contains("container-abc"));
    assert!(json.contains("action"));
    assert!(json.contains("start"));
}

/// Test EventData deserialization from JSON for Container variant
#[test]
fn test_event_data_container_deserialization() {
    let json = r#"{
        "type": "Container",
        "payload": {
            "container_id": "container-def",
            "action": "stop"
        }
    }"#;
    
    let event_data: EventData = serde_json::from_str(json).expect("Failed to deserialize");
    match event_data {
        EventData::Container { container_id, action } => {
            assert_eq!(container_id, "container-def");
            assert_eq!(action, "stop");
        }
        _ => panic!("Expected Container variant"),
    }
}

/// Test EventData serialization to JSON for Agent variant
#[test]
fn test_event_data_agent_serialization() {
    let event_data = EventData::Agent {
        agent_id: "agent-001".to_string(),
        message: "Task completed successfully".to_string(),
    };
    
    let json = serde_json::to_string(&event_data).expect("Failed to serialize");
    assert!(json.contains("agent_id"));
    assert!(json.contains("agent-001"));
    assert!(json.contains("message"));
    assert!(json.contains("Task completed successfully"));
}

/// Test EventData deserialization from JSON for Agent variant
#[test]
fn test_event_data_agent_deserialization() {
    let json = r#"{
        "type": "Agent",
        "payload": {
            "agent_id": "agent-002",
            "message": "Processing request"
        }
    }"#;
    
    let event_data: EventData = serde_json::from_str(json).expect("Failed to deserialize");
    match event_data {
        EventData::Agent { agent_id, message } => {
            assert_eq!(agent_id, "agent-002");
            assert_eq!(message, "Processing request");
        }
        _ => panic!("Expected Agent variant"),
    }
}

/// Test EventData serialization to JSON for Custom variant
#[test]
fn test_event_data_custom_serialization() {
    let custom_value = json!({"key": "value", "number": 42});
    let event_data = EventData::Custom(custom_value);
    
    let json = serde_json::to_string(&event_data).expect("Failed to serialize");
    assert!(json.contains("key"));
    assert!(json.contains("value"));
    assert!(json.contains("number"));
}

/// Test EventData deserialization from JSON for Custom variant
#[test]
fn test_event_data_custom_deserialization() {
    let json = r#"{"type":"Custom","payload":{"key":"value","number":42}}"#;
    
    let event_data: EventData = serde_json::from_str(json).expect("Failed to deserialize");
    match event_data {
        EventData::Custom(value) => {
            assert_eq!(value["key"], "value");
            assert_eq!(value["number"], 42);
        }
        _ => panic!("Expected Custom variant"),
    }
}

/// Test Event struct serialization
#[test]
fn test_event_serialization() {
    let timestamp = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    
    let event = Event {
        id: "event-uuid-123".to_string(),
        timestamp,
        event_type: "scheduler.task.started".to_string(),
        source: "scheduler".to_string(),
        data: EventData::Scheduler {
            task_id: "task-789".to_string(),
            status: "started".to_string(),
        },
    };
    
    let json = serde_json::to_string(&event).expect("Failed to serialize");
    assert!(json.contains("event-uuid-123"));
    assert!(json.contains("scheduler.task.started"));
    assert!(json.contains("scheduler"));
    assert!(json.contains("task-789"));
}

/// Test Event struct deserialization
#[test]
fn test_event_deserialization() {
    let json = r#"{
        "id": "event-uuid-456",
        "timestamp": "2024-01-15T10:30:00Z",
        "event_type": "container.started",
        "source": "docker",
        "data": {
            "type": "Container",
            "payload": {
                "container_id": "container-xyz",
                "action": "started"
            }
        }
    }"#;
    
    let event: Event = serde_json::from_str(json).expect("Failed to deserialize");
    assert_eq!(event.id, "event-uuid-456");
    assert_eq!(event.event_type, "container.started");
    assert_eq!(event.source, "docker");
    
    match event.data {
        EventData::Container { container_id, action } => {
            assert_eq!(container_id, "container-xyz");
            assert_eq!(action, "started");
        }
        _ => panic!("Expected Container variant"),
    }
}

/// Test EventEmitter can be created with a file path
#[test]
fn test_event_emitter_creation() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();
    
    let emitter = EventEmitter::new(path.clone());
    assert!(emitter.is_ok());
    
    let _ = emitter;
    // temp_file is automatically cleaned up
}

/// Test EventEmitter can emit an event to a file
#[test]
fn test_event_emitter_emit() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();
    
    let mut emitter = EventEmitter::new(path.clone()).expect("Failed to create emitter");
    let timestamp = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event = Event {
        id: "test-event-001".to_string(),
        timestamp,
        event_type: "test.event".to_string(),
        source: "test".to_string(),
        data: EventData::Agent {
            agent_id: "agent-1".to_string(),
            message: "Test".to_string(),
        },
    };
    let result = emitter.emit(event);
    
    // Verify the file contains the JSON line
    let content = fs::read_to_string(&path).expect("Failed to read file");
    assert!(!content.is_empty());
    assert!(content.contains("test-event-001"));
}

/// Test EventEmitter can emit multiple events to a file (JSON Lines format)
#[test]
fn test_event_emitter_multiple_events() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path().to_path_buf();
    
    let mut emitter = EventEmitter::new(path.clone()).expect("Failed to create emitter");
    let timestamp = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    let event1 = Event {
        id: "event-001".to_string(),
        timestamp,
        event_type: "event.first".to_string(),
        source: "test".to_string(),
        data: EventData::Scheduler {
            task_id: "task-001".to_string(),
            status: "running".to_string(),
        },
    };
    
    // Emit second event
    let event2 = Event {
        id: "event-002".to_string(),
        timestamp,
        event_type: "event.second".to_string(),
        source: "test".to_string(),
        data: EventData::Container {
            container_id: "container-001".to_string(),
            action: "started".to_string(),
        },
    };
    
    emitter.emit(event1).expect("Failed to emit first event");
    emitter.emit(event2).expect("Failed to emit second event");
    
    // Verify the file contains both events as JSON lines
    let content = fs::read_to_string(&path).expect("Failed to read file");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("event-001"));
    assert!(lines[1].contains("event-002"));
}

/// Test EventEmitter returns error for invalid path
#[test]
fn test_event_emitter_invalid_path() {
    let invalid_path = std::path::PathBuf::from("/nonexistent/directory/events.jsonl");
    
    let emitter = EventEmitter::new(invalid_path);
    assert!(emitter.is_err());
}

/// Test event schema validation - ensure all required fields are present
#[test]
fn test_event_schema_validation() {
    // Test with valid event
    let timestamp = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
        .unwrap()
        .with_timezone(&Utc);
    
    let event = Event {
        id: "schema-test-001".to_string(),
        timestamp,
        event_type: "validation.test".to_string(),
        source: "test".to_string(),
        data: EventData::Custom(json!({"valid": true})),
    };
    
    // Serialize and deserialize to verify schema is preserved
    let json = serde_json::to_string(&event).expect("Failed to serialize");
    let deserialized: Event = serde_json::from_str(&json).expect("Failed to deserialize");
    
    assert_eq!(event.id, deserialized.id);
    assert_eq!(event.event_type, deserialized.event_type);
    assert_eq!(event.source, deserialized.source);
}

/// Test that EventData enum correctly serializes with its tag
#[test]
fn test_event_data_enum_tag() {
    // Test each variant serializes with its tag
    let scheduler = EventData::Scheduler {
        task_id: "t1".to_string(),
        status: "s1".to_string(),
    };
    let scheduler_json = serde_json::to_string(&scheduler).unwrap();
    assert!(scheduler_json.contains("\"type\":\"Scheduler\""));
    
    let container = EventData::Container {
        container_id: "c1".to_string(),
        action: "a1".to_string(),
    };
    let container_json = serde_json::to_string(&container).unwrap();
    assert!(container_json.contains("\"type\":\"Container\""));
    
    let agent = EventData::Agent {
        agent_id: "ag1".to_string(),
        message: "m1".to_string(),
    };
    let agent_json = serde_json::to_string(&agent).unwrap();
    assert!(agent_json.contains("\"type\":\"Agent\""));
    
    let custom = EventData::Custom(json!({"key": "value"}));
    let custom_json = serde_json::to_string(&custom).unwrap();
    assert!(custom_json.contains("\"type\":\"Custom\""));
}
