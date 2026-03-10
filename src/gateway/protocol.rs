//! Gateway protocol types for gateway<->project communication

use serde::{Deserialize, Serialize};

/// GatewayMessage enum for gateway<->project communication
///
/// This enum defines the message types used for communication between
/// the gateway service and registered projects.
/// Uses externally-tagged format where the message type is specified
/// in a "type" field at the top level, e.g.:
/// {"type": "register", "project_name": "...", "channels": [...]}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayMessage {
    /// Register a new project with the gateway
    ///
    /// Sent by a project to register itself with the gateway service.
    /// The gateway responds with a RegisterAck message.
    /// Register a new project with the gateway
    Register {
        /// Name of the project to register
        project_name: String,
        /// List of channel names the project wants to subscribe to
        channels: Vec<String>,
    },

    /// Acknowledge project registration
    ///
    /// Sent by the gateway in response to a Register message.
    /// Contains the session ID for the registered project.
    /// Acknowledge project registration
    RegisterAck {
        /// Status of the registration (e.g., "ok")
        status: String,
        /// Unique session ID for this registration
        session_id: String,
    },

    /// Error during registration
    ///
    /// Sent by the gateway when registration fails.
    /// Error during registration
    RegisterError {
        /// Error message describing why registration failed
        error: String,
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
    Heartbeat {
        /// Unix timestamp of the heartbeat
        timestamp: u64,
    },

    /// Acknowledge heartbeat
    HeartbeatAck {
        /// Unix timestamp matching the acknowledged heartbeat
        timestamp: u64,
    },

    /// Subscribe to additional channels at runtime
    ///
    /// Sent by a project to subscribe to additional channels beyond
    /// those specified during registration.
    ChannelSubscribe {
        /// List of channel names to subscribe to
        channels: Vec<String>,
    },

    /// Acknowledge channel subscription
    ///
    /// Sent by the gateway in response to a ChannelSubscribe message.
    ChannelSubscribeAck {
        /// Status of the subscription (e.g., "ok")
        status: String,
    },

    /// Unsubscribe from channels at runtime
    ///
    /// Sent by a project to unsubscribe from specific channels.
    ChannelUnsubscribe {
        /// List of channel names to unsubscribe from
        channels: Vec<String>,
    },

    /// Acknowledge channel unsubscription
    ///
    /// Sent by the gateway in response to a ChannelUnsubscribe message.
    ChannelUnsubscribeAck {
        /// Status of the unsubscription (e.g., "ok")
        status: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test Register message serialization and deserialization roundtrip
    #[test]
    fn test_register_serialization_roundtrip() {
        let msg = GatewayMessage::Register {
            project_name: "test-project-123".to_string(),
            channels: vec!["channel1".to_string(), "channel2".to_string()],
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Register");
        assert!(json.contains("\"type\":\"register\""));
        assert!(json.contains("\"project_name\":\"test-project-123\""));
        assert!(json.contains("\"channels\""));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize Register");
        assert!(
            matches!(deserialized, GatewayMessage::Register { project_name, channels, .. } 
            if project_name == "test-project-123" 
            && channels.len() == 2
            && channels[0] == "channel1"
            && channels[1] == "channel2")
        );
    }

    /// Test RegisterAck message serialization and deserialization roundtrip
    #[test]
    fn test_register_ack_serialization_roundtrip() {
        let msg = GatewayMessage::RegisterAck {
            status: "ok".to_string(),
            session_id: "session-abc-123".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize RegisterAck");
        assert!(json.contains("\"type\":\"register_ack\""));
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"session_id\":\"session-abc-123\""));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize RegisterAck");
        assert!(matches!(
            deserialized,
            GatewayMessage::RegisterAck { status, session_id, .. }
            if status == "ok" && session_id == "session-abc-123"
        ));
    }

    /// Test RegisterError message serialization and deserialization roundtrip
    #[test]
    fn test_register_error_serialization_roundtrip() {
        let msg = GatewayMessage::RegisterError {
            error: "Project name already exists".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize RegisterError");
        assert!(json.contains("\"type\":\"register_error\""));
        assert!(json.contains("\"error\":\"Project name already exists\""));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize RegisterError");
        assert!(
            matches!(deserialized, GatewayMessage::RegisterError { error, .. } 
            if error == "Project name already exists")
        );
    }

    /// Test Message serialization and deserialization roundtrip
    #[test]
    fn test_message_serialization_roundtrip() {
        let msg = GatewayMessage::Message {
            payload: "Hello, world!".to_string(),
            channel_id: 12345,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Message");
        assert!(json.contains("\"type\":\"message\""));
        assert!(json.contains("Hello, world!"));
        assert!(json.contains("12345"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize Message");
        assert!(matches!(
            deserialized,
            GatewayMessage::Message { payload, channel_id, .. }
            if payload == "Hello, world!" && channel_id == 12345
        ));
    }

    /// Test Heartbeat serialization and deserialization roundtrip
    #[test]
    fn test_heartbeat_serialization_roundtrip() {
        let msg = GatewayMessage::Heartbeat {
            timestamp: 1699999999,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize Heartbeat");
        assert!(json.contains("\"type\":\"heartbeat\""));
        assert!(json.contains("1699999999"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize Heartbeat");
        assert!(
            matches!(deserialized, GatewayMessage::Heartbeat { timestamp, .. } if timestamp == 1699999999)
        );
    }

    /// Test HeartbeatAck serialization and deserialization roundtrip
    #[test]
    fn test_heartbeat_ack_serialization_roundtrip() {
        let msg = GatewayMessage::HeartbeatAck {
            timestamp: 1700000000,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize HeartbeatAck");
        assert!(json.contains("\"type\":\"heartbeat_ack\""));
        assert!(json.contains("1700000000"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize HeartbeatAck");
        assert!(
            matches!(deserialized, GatewayMessage::HeartbeatAck { timestamp, .. } if timestamp == 1700000000)
        );
    }

    /// Test that heartbeat message is properly deserialized from JSON string
    #[test]
    fn should_deserialize_heartbeat_from_json_string() {
        let json = r#"{"type":"heartbeat","timestamp":1699999999}"#;
        let deserialized: GatewayMessage =
            serde_json::from_str(json).expect("Failed to deserialize Heartbeat from JSON");
        assert!(
            matches!(deserialized, GatewayMessage::Heartbeat { timestamp, .. } if timestamp == 1699999999)
        );
    }

    /// Test that HeartbeatAck is correctly serialized to JSON
    #[test]
    fn should_serialize_heartbeat_ack_to_json() {
        let msg = GatewayMessage::HeartbeatAck {
            timestamp: 1700000000,
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize HeartbeatAck");
        assert!(json.contains("\"type\":\"heartbeat_ack\""));
        assert!(json.contains("1700000000"));
    }

    /// Test ChannelSubscribe serialization and deserialization roundtrip
    #[test]
    fn should_serialize_and_deserialize_channel_subscribe() {
        let msg = GatewayMessage::ChannelSubscribe {
            channels: vec!["channel1".to_string(), "channel2".to_string()],
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize ChannelSubscribe");
        assert!(json.contains("\"type\":\"channel_subscribe\""));
        assert!(json.contains("\"channels\":"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize ChannelSubscribe");
        assert!(
            matches!(deserialized, GatewayMessage::ChannelSubscribe { channels, .. } 
            if channels.len() == 2
            && channels[0] == "channel1"
            && channels[1] == "channel2")
        );
    }

    /// Test ChannelSubscribeAck serialization and deserialization roundtrip
    #[test]
    fn should_serialize_and_deserialize_channel_subscribe_ack() {
        let msg = GatewayMessage::ChannelSubscribeAck {
            status: "ok: subscribed to 2 channels".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize ChannelSubscribeAck");
        assert!(json.contains("\"type\":\"channel_subscribe_ack\""));
        assert!(json.contains("\"status\":\"ok: subscribed to 2 channels\""));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize ChannelSubscribeAck");
        assert!(
            matches!(deserialized, GatewayMessage::ChannelSubscribeAck { status, .. } 
            if status == "ok: subscribed to 2 channels")
        );
    }

    /// Test ChannelUnsubscribe serialization and deserialization roundtrip
    #[test]
    fn should_serialize_and_deserialize_channel_unsubscribe() {
        let msg = GatewayMessage::ChannelUnsubscribe {
            channels: vec!["channel1".to_string()],
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize ChannelUnsubscribe");
        assert!(json.contains("\"type\":\"channel_unsubscribe\""));
        assert!(json.contains("\"channels\":"));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize ChannelUnsubscribe");
        assert!(
            matches!(deserialized, GatewayMessage::ChannelUnsubscribe { channels, .. } 
            if channels.len() == 1
            && channels[0] == "channel1")
        );
    }

    /// Test ChannelUnsubscribeAck serialization and deserialization roundtrip
    #[test]
    fn should_serialize_and_deserialize_channel_unsubscribe_ack() {
        let msg = GatewayMessage::ChannelUnsubscribeAck {
            status: "ok: unsubscribed from 1 channels".to_string(),
        };
        let json = serde_json::to_string(&msg).expect("Failed to serialize ChannelUnsubscribeAck");
        assert!(json.contains("\"type\":\"channel_unsubscribe_ack\""));
        assert!(json.contains("\"status\":\"ok: unsubscribed from 1 channels\""));
        let deserialized: GatewayMessage =
            serde_json::from_str(&json).expect("Failed to deserialize ChannelUnsubscribeAck");
        assert!(
            matches!(deserialized, GatewayMessage::ChannelUnsubscribeAck { status, .. } 
            if status == "ok: unsubscribed from 1 channels")
        );
    }
}
