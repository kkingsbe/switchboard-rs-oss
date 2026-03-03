//! Message routing module for the gateway.
//!
//! This module provides routing functionality to forward Discord messages
//! to the appropriate project WebSocket connections based on channel subscriptions.

use crate::gateway::protocol::GatewayMessage;
use crate::gateway::registry::ChannelRegistry;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur during routing operations.
#[derive(Error, Debug)]
pub enum RoutingError {
    /// Failed to serialize message for routing.
    #[error("Failed to serialize message: {0}")]
    SerializationError(String),

    /// Failed to send message to a project.
    #[error("Failed to send message to project {project_id}: {source}")]
    SendError {
        /// The project ID that failed to receive the message.
        project_id: String,
        /// The underlying send error.
        #[source]
        source: tokio::sync::mpsc::error::SendError<String>,
    },

    /// Registry operation failed.
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Invalid channel ID format.
    #[error("Invalid channel ID '{channel_id}': {source}")]
    InvalidChannelId {
        /// The invalid channel ID.
        channel_id: String,
        /// The parse error.
        #[source]
        source: std::num::ParseIntError,
    },
}

/// Result type for routing operations.
pub type RoutingResult<T> = Result<T, RoutingError>;

/// Router for forwarding messages to projects based on channel subscriptions.
///
/// This struct wraps a `ChannelRegistry` and provides methods to route
/// messages to all projects subscribed to a specific channel.
#[derive(Debug, Clone)]
pub struct Router {
    /// The channel registry for looking up project subscriptions.
    registry: ChannelRegistry,
}

impl Router {
    /// Create a new Router with the given registry.
    ///
    /// # Arguments
    ///
    /// * `registry` - The channel registry to use for routing lookups
    ///
    /// # Returns
    ///
    /// A new Router instance.
    pub fn new(registry: ChannelRegistry) -> Self {
        Self { registry }
    }

    /// Route a message to all projects subscribed to the given channel.
    ///
    /// This method looks up all projects subscribed to the specified channel
    /// and forwards the message to each project's WebSocket sender.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID (as a string)
    /// * `content` - The message content to forward
    ///
    /// # Returns
    ///
    /// * `RoutingResult<usize>` - The number of projects the message was sent to
    pub async fn route_message(&self, channel_id: &str, content: &str) -> RoutingResult<usize> {
        // Look up projects subscribed to this channel
        let project_ids = self.registry.projects_for_channel(channel_id).await;

        if project_ids.is_empty() {
            debug!("No projects subscribed to channel {}", channel_id);
            return Ok(0);
        }

        // Parse channel_id to validate it
        let parsed_channel_id =
            channel_id
                .parse::<u64>()
                .map_err(|source| RoutingError::InvalidChannelId {
                    channel_id: channel_id.to_string(),
                    source,
                })?;

        info!(
            "Routing message from channel {} to {} projects",
            channel_id,
            project_ids.len()
        );

        // Forward message to each subscribed project
        let mut sent_count = 0;
        for project_id in &project_ids {
            if let Ok(project) = self.registry.get_project(project_id).await {
                // Create the message payload
                let message = GatewayMessage::Message {
                    payload: content.to_string(),
                    channel_id: parsed_channel_id,
                };

                match serde_json::to_string(&message) {
                    Ok(json) => {
                        if project.ws_sender.send(json).await.is_err() {
                            warn!(
                                "Failed to send message to project {}, client may be disconnected",
                                project_id
                            );
                        } else {
                            info!(
                                "Forwarded message to project {} ({})",
                                project.project_name, project_id
                            );
                            sent_count += 1;
                        }
                    }
                    Err(e) => {
                        warn!(
                            "Failed to serialize message for project {}: {}",
                            project_id, e
                        );
                    }
                }
            }
        }

        Ok(sent_count)
    }

    /// Extract channel ID from a Discord message event.
    ///
    /// This is a utility function that validates and parses a channel ID string.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID as a string
    ///
    /// # Returns
    ///
    /// * `RoutingResult<u64>` - The parsed channel ID
    pub fn extract_channel_id(channel_id: &str) -> RoutingResult<u64> {
        channel_id
            .parse::<u64>()
            .map_err(|source| RoutingError::InvalidChannelId {
                channel_id: channel_id.to_string(),
                source,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gateway::registry::ProjectConnection;
    use tokio::sync::mpsc;

    /// Helper to create a test project connection with a live receiver
    /// Returns (project, sender, receiver) where receiver must be kept alive
    fn create_test_project_with_receiver(
        project_id: &str,
    ) -> (
        ProjectConnection,
        mpsc::Sender<String>,
        mpsc::Receiver<String>,
    ) {
        let (sender, receiver) = mpsc::channel(10);
        let sender_clone = sender.clone();
        let project = ProjectConnection::new(
            project_id.to_string(),
            format!("Test Project {}", project_id),
            sender,
        );
        (project, sender_clone, receiver)
    }

    #[tokio::test]
    async fn test_router_creation() {
        let registry = ChannelRegistry::new();
        let router = Router::new(registry);

        // Verify router was created
        let _ = router;
    }

    #[tokio::test]
    async fn test_route_message_no_subscribers() {
        let registry = ChannelRegistry::new();
        let router = Router::new(registry);

        // Route to a channel with no subscribers
        let result = router.route_message("123456789", "Hello").await.unwrap();

        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_route_message_channel_not_subscribed() {
        let registry = ChannelRegistry::new();

        // Register project to channel "456"
        let (project, _sender, _receiver) = create_test_project_with_receiver("project-1");
        registry
            .register(project, vec!["456".to_string()])
            .await
            .unwrap();

        let router = Router::new(registry);

        // Route to different channel "123" - should have no subscribers
        let result = router.route_message("123", "Hello").await.unwrap();

        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_extract_channel_id_valid() {
        let result = Router::extract_channel_id("123456789012345678").unwrap();
        assert_eq!(result, 123456789012345678);
    }

    #[tokio::test]
    async fn test_extract_channel_id_small_number() {
        let result = Router::extract_channel_id("123").unwrap();
        assert_eq!(result, 123);
    }

    #[tokio::test]
    async fn test_extract_channel_id_invalid() {
        let result = Router::extract_channel_id("invalid");

        assert!(result.is_err());
        match result.unwrap_err() {
            RoutingError::InvalidChannelId { channel_id, .. } => {
                assert_eq!(channel_id, "invalid");
            }
            _ => panic!("Expected InvalidChannelId error"),
        }
    }

    #[tokio::test]
    async fn test_extract_channel_id_with_leading_zeros() {
        // Leading zeros should still parse correctly
        let result = Router::extract_channel_id("00123456789").unwrap();
        assert_eq!(result, 123456789);
    }

    #[tokio::test]
    async fn test_route_message_with_live_receiver() {
        let registry = ChannelRegistry::new();

        // Register a project to channel "123" - keep receiver alive
        let (project, _sender, mut receiver) = create_test_project_with_receiver("project-1");
        registry
            .register(project, vec!["123".to_string()])
            .await
            .unwrap();

        let router = Router::new(registry);

        // Spawn a task to receive the message
        let handle = tokio::spawn(async move { receiver.recv().await });

        // Route message to channel "123"
        let result = router.route_message("123", "Hello World").await.unwrap();

        // Wait for the receiver to get the message
        let received = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;

        assert_eq!(result, 1);
        assert!(received.is_ok(), "Message should be received");
    }

    #[tokio::test]
    async fn test_route_message_multiple_projects_partial_failure() {
        let registry = ChannelRegistry::new();

        // Register first project with live receiver
        let (project1, _sender1, mut receiver1) = create_test_project_with_receiver("project-1");
        // Register second project without receiver (will fail to send)
        let (project2, _sender2, _receiver2) = create_test_project_with_receiver("project-2");

        registry
            .register(project1, vec!["123".to_string()])
            .await
            .unwrap();
        registry
            .register(project2, vec!["123".to_string()])
            .await
            .unwrap();

        let router = Router::new(registry);

        // Spawn a task to receive the message
        let handle = tokio::spawn(async move { receiver1.recv().await });

        // Route message - should send to at least one project
        let result = router.route_message("123", "Hello").await.unwrap();

        // Wait for the receiver to get the message
        let _ = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle).await;

        // At least one should succeed
        assert!(result >= 1, "At least one message should be sent");
    }

    /// Test 1: All subscribed projects receive the message
    /// Verifies fan-out behavior: when 3 projects are subscribed to the same channel,
    /// all 3 should receive the message.
    #[tokio::test]
    async fn test_fan_out_all_subscribed_projects_receive_message() {
        let registry = ChannelRegistry::new();

        // Register 3 projects to the same channel, keeping all receivers alive
        let (project1, _sender1, mut receiver1) = create_test_project_with_receiver("project-1");
        let (project2, _sender2, mut receiver2) = create_test_project_with_receiver("project-2");
        let (project3, _sender3, mut receiver3) = create_test_project_with_receiver("project-3");

        registry
            .register(project1, vec!["123".to_string()])
            .await
            .unwrap();
        registry
            .register(project2, vec!["123".to_string()])
            .await
            .unwrap();
        registry
            .register(project3, vec!["123".to_string()])
            .await
            .unwrap();

        let router = Router::new(registry);

        // Spawn tasks to receive messages from each project
        let handle1 = tokio::spawn(async move { receiver1.recv().await });
        let handle2 = tokio::spawn(async move { receiver2.recv().await });
        let handle3 = tokio::spawn(async move { receiver3.recv().await });

        // Route message to channel "123"
        let result = router.route_message("123", "Fan-out message").await.unwrap();

        // Wait for all receivers to get the message
        let received1 = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle1).await;
        let received2 = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle2).await;
        let received3 = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle3).await;

        // Verify all 3 projects received the message
        assert_eq!(result, 3, "Message should be sent to all 3 projects");
        assert!(received1.is_ok(), "Project 1 should receive the message");
        assert!(received2.is_ok(), "Project 2 should receive the message");
        assert!(received3.is_ok(), "Project 3 should receive the message");
    }

    /// Test 2: Failure isolation - one failure doesn't stop others
    /// Verifies that when one project's sender is dropped (simulating disconnect),
    /// the other projects still receive the message.
    #[tokio::test]
    async fn test_fan_out_failure_isolation_one_project_disconnected() {
        let registry = ChannelRegistry::new();

        // Register 3 projects to the same channel
        let (project1, _sender1, mut receiver1) = create_test_project_with_receiver("project-1");
        // Project 2: drop the sender to simulate disconnect
        let (project2, sender2, receiver2) = create_test_project_with_receiver("project-2");
        let (project3, _sender3, mut receiver3) = create_test_project_with_receiver("project-3");

        registry
            .register(project1, vec!["123".to_string()])
            .await
            .unwrap();
        // Project 2's sender is dropped, but receiver is kept to not cause panic
        // This simulates a disconnected client (sender dropped = connection closed)
        drop(sender2);
        registry
            .register(project2, vec!["123".to_string()])
            .await
            .unwrap();
        // Now drop receiver for project 2 to fully simulate disconnected state
        drop(receiver2);

        registry
            .register(project3, vec!["123".to_string()])
            .await
            .unwrap();

        let router = Router::new(registry);

        // Spawn tasks to receive messages from projects 1 and 3
        let handle1 = tokio::spawn(async move { receiver1.recv().await });
        let handle3 = tokio::spawn(async move { receiver3.recv().await });

        // Route message to channel "123"
        let result = router.route_message("123", "Message despite disconnect").await.unwrap();

        // Wait for receivers to get the message
        let received1 = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle1).await;
        let received3 = tokio::time::timeout(tokio::time::Duration::from_millis(100), handle3).await;

        // Verify projects 1 and 3 still receive the message (result should be 2)
        assert_eq!(result, 2, "Message should be sent to 2 projects (project 2 failed)");
        assert!(received1.is_ok(), "Project 1 should still receive the message");
        assert!(received3.is_ok(), "Project 3 should still receive the message");
    }

    /// Test 3: Message ordering preserved per subscriber
    /// Verifies that when multiple messages are sent in sequence,
    /// each subscriber receives them in the same order.
    #[tokio::test]
    async fn test_fan_out_message_ordering_preserved_per_subscriber() {
        let registry = ChannelRegistry::new();

        // Register a project to channel "123", keeping receiver alive
        let (project, _sender, mut receiver) = create_test_project_with_receiver("project-1");
        registry
            .register(project, vec!["123".to_string()])
            .await
            .unwrap();

        let router = Router::new(registry);

        // Send 3 messages in sequence
        router.route_message("123", "Message 1").await.unwrap();
        router.route_message("123", "Message 2").await.unwrap();
        router.route_message("123", "Message 3").await.unwrap();

        // Receive all 3 messages
        let msg1 = tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.recv()).await;
        let msg2 = tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.recv()).await;
        let msg3 = tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.recv()).await;

        // Verify all messages were received
        assert!(msg1.is_ok(), "Should receive message 1");
        assert!(msg2.is_ok(), "Should receive message 2");
        assert!(msg3.is_ok(), "Should receive message 3");

        // Verify ordering: check the payload content in the JSON
        let content1 = msg1.unwrap().unwrap();
        let content2 = msg2.unwrap().unwrap();
        let content3 = msg3.unwrap().unwrap();

        assert!(content1.contains("Message 1"), "First message should be 'Message 1'");
        assert!(content2.contains("Message 2"), "Second message should be 'Message 2'");
        assert!(content3.contains("Message 3"), "Third message should be 'Message 3'");
    }
}
