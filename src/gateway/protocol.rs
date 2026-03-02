//! Gateway protocol types for gateway<->project communication

use serde::{Deserialize, Serialize};

/// GatewayMessage enum for gateway<->project communication
///
/// This enum defines the message types used for communication between
/// the gateway service and registered projects.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GatewayMessage {
    /// Register a new project with the gateway
    ///
    /// Sent by a project to register itself with the gateway service.
    /// The gateway responds with a RegisterAck message.
    Register {
        /// Unique identifier for the project
        project_id: String,
    },

    /// Acknowledge project registration
    ///
    /// Sent by the gateway in response to a Register message.
    /// Contains the assigned channel for communication.
    RegisterAck {
        /// The registered project's ID
        project_id: String,
        /// The assigned channel ID for this project
        assigned_channel: u64,
    },

    /// Regular message from project or gateway
    ///
    /// Standard message payload sent between gateway and projects.
    Message {
        /// The message content/payload
        payload: String,
        /// The channel ID this message is destined for
        channel_id: u64,
    },

    /// Heartbeat to keep connection alive
    ///
    /// Periodic heartbeat message to maintain the connection
    /// and detect disconnections.
    Heartbeat {
        /// Unix timestamp of the heartbeat
        timestamp: u64,
    },

    /// Acknowledge heartbeat
    ///
    /// Response to a Heartbeat message, confirming the connection
    /// is still alive.
    HeartbeatAck {
        /// Unix timestamp matching the acknowledged heartbeat
        timestamp: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Register message serialization and deserialization roundtrip
    #[test]
    fn test_register_serialization_roundtrip() {
        let msg = GatewayMessage::Register {
            project_id: "test-project-123".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Register");
        assert!(json.contains("\"register\""));
        assert!(json.contains("test-project-123"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize Register");
        assert!(matches!(deserialized, GatewayMessage::Register { project_id } if project_id == "test-project-123"));
    }

    /// Test RegisterAck message serialization and deserialization roundtrip
    #[test]
    fn test_register_ack_serialization_roundtrip() {
        let msg = GatewayMessage::RegisterAck {
            project_id: "test-project-456".to_string(),
            assigned_channel: 789,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize RegisterAck");
        assert!(json.contains("\"registerack\""));
        assert!(json.contains("test-project-456"));
        assert!(json.contains("789"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize RegisterAck");
        assert!(matches!(
            deserialized,
            GatewayMessage::RegisterAck { project_id, assigned_channel }
            if project_id == "test-project-456" && assigned_channel == 789
        ));
    }

    /// Test Message serialization and deserialization roundtrip
    #[test]
    fn test_message_serialization_roundtrip() {
        let msg = GatewayMessage::Message {
            payload: "Hello, world!".to_string(),
            channel_id: 12345,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Message");
        assert!(json.contains("\"message\""));
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("12345"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize Message");
        assert!(matches!(
            deserialized,
            GatewayMessage::Message { payload, channel_id }
            if payload == "Hello, world!" && channel_id == 12345
        ));
    }

    /// Test Heartbeat serialization and deserialization roundtrip
    #[test]
    fn test_heartbeat_serialization_roundtrip() {
        let msg = GatewayMessage::Heartbeat { timestamp: 1699999999 };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Heartbeat");
        assert!(json.contains("\"heartbeat\""));
        assert!(json.contains("1699999999"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize Heartbeat");
        assert!(matches!(deserialized, GatewayMessage::Heartbeat { timestamp } if timestamp == 1699999999));
    }

    /// Test HeartbeatAck serialization and deserialization roundtrip
    #[test]
    fn test_heartbeat_ack_serialization_roundtrip() {
        let msg = GatewayMessage::HeartbeatAck { timestamp: 1700000000 };
        let json = serde_json::to_string(&msg).expect("Failed to serialize HeartbeatAck");
        assert!(json.contains("\"heartbeatack\""));
        assert!(json.contains("1700000000"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize HeartbeatAck");
        assert!(matches!(deserialized, GatewayMessage::HeartbeatAck { timestamp } if timestamp == 1700000000));
    }
}
