//! Tests for per-user conversation loading in Discord listener.
//!
//! These tests verify that when a message arrives from a user, a conversation
//! is loaded or created for that user. The conversation should be identifiable
//! by user_id and channel_id.

#![cfg(feature = "discord")]

use switchboard::discord::conversation::{ConversationConfig, ConversationManager};
use switchboard::discord::listener::{process_message, DiscordMessage, DiscordUser, ListenerConfig};

/// Creates a test listener configuration.
fn create_test_config() -> ListenerConfig {
    ListenerConfig::new(123456789, 987654321)
}

/// Creates a test Discord message.
fn create_test_message(channel_id: u64, author_id: u64, content: &str) -> DiscordMessage {
    DiscordMessage {
        id: 111111111,
        channel_id,
        author: DiscordUser {
            id: author_id,
            username: "testuser".to_string(),
            bot: Some(false),
        },
        content: content.to_string(),
        timestamp: "2024-01-01T00:00:00.000Z".to_string(),
    }
}

/// Test that processing a message creates a conversation for the user.
///
/// This test verifies that when a message arrives from a user:
/// 1. A conversation is loaded or created for that user
/// 2. The conversation is identifiable by user_id
#[test]
fn test_process_message_creates_conversation_for_user() {
    let config = create_test_config();
    let message = create_test_message(config.channel_id, 111111111, "Hello, bot!");

    // Create a conversation manager
    let mut conversation_manager =
        ConversationManager::new(ConversationConfig::default());

    // Process the message - this should create a conversation for the user
    // Note: Currently process_message doesn't interact with conversation_manager
    // This test will fail until the functionality is implemented
    let result = process_message(&message, &config);
    assert!(result.is_ok(), "Message processing should succeed");

    let processed = result.unwrap();

    // The processed message should have the correct user_id and channel_id
    assert_eq!(processed.user_id, 111111111);
    assert_eq!(processed.channel_id, config.channel_id);

    // After processing, a conversation should exist for this user
    // This is the key assertion that will fail
    let user_id_str = processed.user_id.to_string();
    assert!(
        conversation_manager.has_conversation(&user_id_str),
        "Conversation should be created for user {} after processing message",
        user_id_str
    );
}

/// Test that different users get different conversations.
///
/// This test verifies that conversations are per-user, not shared.
#[test]
fn test_different_users_have_separate_conversations() {
    let config = create_test_config();

    // Create a conversation manager
    let mut conversation_manager =
        ConversationManager::new(ConversationConfig::default());

    // Process message from user 1
    let message1 = create_test_message(config.channel_id, 111111111, "Hello from user 1");
    let result1 = process_message(&message1, &config);
    assert!(result1.is_ok(), "First message processing should succeed");

    // Process message from user 2
    let message2 = create_test_message(config.channel_id, 222222222, "Hello from user 2");
    let result2 = process_message(&message2, &config);
    assert!(result2.is_ok(), "Second message processing should succeed");

    // Both users should have separate conversations
    assert!(
        conversation_manager.has_conversation("111111111"),
        "Conversation should exist for user 111111111"
    );
    assert!(
        conversation_manager.has_conversation("222222222"),
        "Conversation should exist for user 222222222"
    );
}

/// Test that messages in the same channel but from different users
/// maintain separate conversations.
#[test]
fn test_separate_conversations_per_user_in_same_channel() {
    let config = create_test_config();
    let channel_id = config.channel_id;

    // Create a conversation manager
    let mut conversation_manager =
        ConversationManager::new(ConversationConfig::default());

    // User 1 sends first message
    let user1_id = 111111111u64;
    let message1 = create_test_message(channel_id, user1_id, "First message from user 1");
    let result1 = process_message(&message1, &config);
    assert!(result1.is_ok());

    // User 2 sends first message in the same channel
    let user2_id = 222222222u64;
    let message2 = create_test_message(channel_id, user2_id, "First message from user 2");
    let result2 = process_message(&message2, &config);
    assert!(result2.is_ok());

    // Verify both users have conversations
    assert!(
        conversation_manager.has_conversation(&user1_id.to_string()),
        "User 1 should have a conversation"
    );
    assert!(
        conversation_manager.has_conversation(&user2_id.to_string()),
        "User 2 should have a conversation"
    );

    // Verify the conversations are separate (different message counts)
    let conv1_len = {
        let conv1 = conversation_manager
            .get_or_create_conversation(&user1_id.to_string());
        conv1.len()
    };
    let conv2_len = {
        let conv2 = conversation_manager
            .get_or_create_conversation(&user2_id.to_string());
        conv2.len()
    };

    // Each should have 1 message
    assert_eq!(conv1_len, 1, "User 1 conversation should have 1 message");
    assert_eq!(conv2_len, 1, "User 2 conversation should have 1 message");
}

/// Test that the ProcessedMessage contains channel_id for conversation identification.
///
/// Conversations should be identifiable by both user_id AND channel_id.
/// This test verifies the processed message has the channel_id needed
/// for proper conversation identification.
#[test]
fn test_processed_message_has_channel_id_for_conversation() {
    let config = create_test_config();
    let channel_id = 999999999u64;

    // Create message in a specific channel
    let message = create_test_message(channel_id, 111111111, "Test message");
    let result = process_message(&message, &config);

    assert!(result.is_ok(), "Message processing should succeed");

    let processed = result.unwrap();

    // The processed message should contain the channel_id
    assert_eq!(
        processed.channel_id, channel_id,
        "Processed message should contain the channel_id"
    );

    // The user_id should also be present
    assert_eq!(
        processed.user_id, 111111111,
        "Processed message should contain the user_id"
    );

    // Both channel_id and user_id should be used to identify the conversation
    // This is important for distinguishing conversations in different channels
    // from the same user
}
