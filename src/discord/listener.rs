//! Discord event listener and message handling.
//!
//! This module provides the core event handling infrastructure for the Discord
//! concierge bot. It processes incoming Discord events, filters messages based
//! on configuration, and dispatches them to appropriate handlers.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Configuration for the Discord event listener.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerConfig {
    /// The Discord channel ID to listen to.
    pub channel_id: u64,
    /// The bot's own user ID to filter out self-messages.
    pub bot_user_id: u64,
}

impl ListenerConfig {
    /// Creates a new listener configuration.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID to listen to
    /// * `bot_user_id` - The bot's Discord user ID
    pub fn new(channel_id: u64, bot_user_id: u64) -> Self {
        Self {
            channel_id,
            bot_user_id,
        }
    }
}

/// Represents a Discord message from the MessageCreate event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    /// The unique message ID.
    pub id: u64,
    /// The channel ID where the message was sent.
    pub channel_id: u64,
    /// The author of the message.
    pub author: DiscordUser,
    /// The content of the message.
    pub content: String,
    /// Timestamp when the message was sent (ISO 8601 format).
    pub timestamp: String,
}

/// Represents a Discord user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    /// The user's unique ID.
    pub id: u64,
    /// The user's username.
    pub username: String,
    /// Whether the user is a bot.
    pub bot: Option<bool>,
}

impl DiscordUser {
    /// Checks if this user is a bot.
    pub fn is_bot(&self) -> bool {
        self.bot.unwrap_or(false)
    }
}

/// Errors that can occur during message handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageHandlerError {
    /// Message was filtered out (e.g., from bot or wrong channel).
    Filtered(String),
    /// Failed to parse or process the message.
    ProcessingError(String),
    /// Failed to dispatch message to handler.
    DispatchError(String),
}

impl fmt::Display for MessageHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageHandlerError::Filtered(msg) => write!(f, "Message filtered: {}", msg),
            MessageHandlerError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
            MessageHandlerError::DispatchError(msg) => write!(f, "Dispatch error: {}", msg),
        }
    }
}

impl std::error::Error for MessageHandlerError {}

impl From<anyhow::Error> for MessageHandlerError {
    fn from(err: anyhow::Error) -> Self {
        MessageHandlerError::ProcessingError(err.to_string())
    }
}

/// Result type for message handler operations.
pub type MessageHandlerResult<T> = Result<T, MessageHandlerError>;

/// Handles incoming Discord messages.
#[derive(Debug, Clone)]
pub struct MessageHandler {
    config: ListenerConfig,
}

impl MessageHandler {
    /// Creates a new message handler with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The listener configuration
    pub fn new(config: ListenerConfig) -> Self {
        Self { config }
    }

    /// Creates a new message handler with the given channel and bot user IDs.
    pub fn with_ids(channel_id: u64, bot_user_id: u64) -> Self {
        Self::new(ListenerConfig::new(channel_id, bot_user_id))
    }

    /// Gets a reference to the handler configuration.
    pub fn config(&self) -> &ListenerConfig {
        &self.config
    }
}

/// Filters a message to determine if it should be processed.
///
/// This function checks:
/// 1. The message is not from the bot itself (author.id != bot_user_id)
/// 2. The message is from the configured channel (channel_id matches)
///
/// # Arguments
///
/// * `message` - The Discord message to filter
/// * `config` - The listener configuration
///
/// # Returns
///
/// * `Ok(true)` - If the message should be processed
/// * `Err(MessageHandlerError::Filtered)` - If the message should be ignored
pub fn handle_message_create(
    message: &DiscordMessage,
    config: &ListenerConfig,
) -> MessageHandlerResult<bool> {
    // Check if message is from the bot itself
    if message.author.id == config.bot_user_id {
        tracing::debug!(
            "Ignoring message {} from bot user {}",
            message.id,
            message.author.id
        );
        return Err(MessageHandlerError::Filtered(
            "Message from bot user".to_string(),
        ));
    }

    // Check if message is from the configured channel
    if message.channel_id != config.channel_id {
        tracing::debug!(
            "Ignoring message {} from wrong channel {} (expected {})",
            message.id,
            message.channel_id,
            config.channel_id
        );
        return Err(MessageHandlerError::Filtered(
            "Message from wrong channel".to_string(),
        ));
    }

    tracing::debug!(
        "Accepting message {} from user {} in channel {}",
        message.id,
        message.author.username,
        message.channel_id
    );

    Ok(true)
}

/// Processes a Discord message, filtering and dispatching to the conversation manager.
///
/// This is the main entry point for handling incoming Discord messages.
/// It applies filtering logic and then extracts user information for
/// further processing.
///
/// # Arguments
///
/// * `message` - The Discord message to process
/// * `config` - The listener configuration
///
/// # Returns
///
/// * `Ok(ProcessedMessage)` - If the message was successfully processed
/// * `Err(MessageHandlerError)` - If the message was filtered or processing failed
pub fn process_message(
    message: &DiscordMessage,
    config: &ListenerConfig,
) -> MessageHandlerResult<ProcessedMessage> {
    // Apply filtering logic using handle_message_create
    handle_message_create(message, config).context("Failed message filter check")?;

    // Extract user ID and message content
    let user_id = message.author.id;
    let content = message.content.clone();

    // Log the message for debugging
    tracing::info!(
        "Processing message from user {}: {}",
        user_id,
        content.chars().take(50).collect::<String>()
    );

    // Return the processed message data
    // Note: In a full implementation, this would dispatch to the conversation manager
    Ok(ProcessedMessage {
        user_id,
        content,
        message_id: message.id,
        channel_id: message.channel_id,
        timestamp: message.timestamp.clone(),
    })
}

/// Represents a processed message ready for further handling.
#[derive(Debug, Clone)]
pub struct ProcessedMessage {
    /// The user ID who sent the message.
    pub user_id: u64,
    /// The message content.
    pub content: String,
    /// The original message ID.
    pub message_id: u64,
    /// The channel ID where the message was sent.
    pub channel_id: u64,
    /// The timestamp of the message.
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> ListenerConfig {
        ListenerConfig::new(123456789, 987654321)
    }

    fn create_test_message(channel_id: u64, author_id: u64) -> DiscordMessage {
        DiscordMessage {
            id: 111111111,
            channel_id,
            author: DiscordUser {
                id: author_id,
                username: "testuser".to_string(),
                bot: Some(false),
            },
            content: "Hello, bot!".to_string(),
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        }
    }

    #[test]
    fn test_handle_message_create_accepts_valid_message() {
        let config = create_test_config();
        let message = create_test_message(config.channel_id, 111111111);

        let result = handle_message_create(&message, &config);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_handle_message_create_rejects_bot_message() {
        let config = create_test_config();
        // Create message from the bot user
        let message = create_test_message(config.channel_id, config.bot_user_id);

        let result = handle_message_create(&message, &config);
        assert!(result.is_err());
        matches::assert_matches!(result, Err(MessageHandlerError::Filtered(_)));
    }

    #[test]
    fn test_handle_message_create_rejects_wrong_channel() {
        let config = create_test_config();
        let message = create_test_message(999999999, 111111111);

        let result = handle_message_create(&message, &config);
        assert!(result.is_err());
        matches::assert_matches!(result, Err(MessageHandlerError::Filtered(_)));
    }

    #[test]
    fn test_process_message_extracts_user_info() {
        let config = create_test_config();
        let message = create_test_message(config.channel_id, 111111111);

        let result = process_message(&message, &config);
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.user_id, 111111111);
        assert_eq!(processed.content, "Hello, bot!");
        assert_eq!(processed.message_id, 111111111);
    }

    #[test]
    fn test_message_handler_creation() {
        let handler = MessageHandler::with_ids(123, 456);
        assert_eq!(handler.config().channel_id, 123);
        assert_eq!(handler.config().bot_user_id, 456);
    }

    #[test]
    fn test_discord_user_is_bot() {
        let bot_user = DiscordUser {
            id: 1,
            username: "Bot".to_string(),
            bot: Some(true),
        };
        assert!(bot_user.is_bot());

        let regular_user = DiscordUser {
            id: 2,
            username: "User".to_string(),
            bot: Some(false),
        };
        assert!(!regular_user.is_bot());
    }
}
