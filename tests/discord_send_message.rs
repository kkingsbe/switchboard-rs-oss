//! Integration tests for send_message function
//!
//! This test file tests the DiscordApiClient send_message method which should:
//! - Accept channel_id and message content as parameters
//! - Send a POST request to /channels/{channel_id}/messages
//! - Return a Message on success or ApiError on failure

#[cfg(feature = "discord")]
use switchboard::discord::api::{ApiError, DiscordApiClient, Message};

#[cfg(feature = "discord")]
/// Test that send_message function has the correct signature
/// This test will fail to compile if send_message doesn't exist or has wrong signature
#[tokio::test]
async fn test_send_message_function_exists() {
    // This line will fail to compile if send_message doesn't exist as a method on DiscordApiClient
    // with the expected signature: async fn send_message(&mut self, &str, &str) -> Result<Message, ApiError>

    // We're just testing that the function can be called with the right signature
    // In a real scenario, this would make an actual API call
    // For now, we just verify the function signature compiles

    let mut client = DiscordApiClient::new("test_bot_token".to_string());
    let _channel_id = "123456789";
    let _message = "Test message";

    // This will fail with "cannot find method" if the method doesn't exist
    // or if it has a different signature
    let _result = client.send_message(_channel_id, _message).await;

    // Note: We don't assert on the result because we expect it to fail with
    // network/authentication errors in a test environment without a real Discord bot
}

#[cfg(feature = "discord")]
/// Test send_message with empty message
/// This verifies the function accepts empty strings as valid parameters
#[tokio::test]
async fn test_send_message_empty_message() {
    let mut client = DiscordApiClient::new("test_bot_token".to_string());
    let channel_id = "123456789";
    let message = "";

    // This should compile and return an error (empty message might be rejected by API)
    let result = client.send_message(channel_id, message).await;

    // Result should be Err since we're using a fake token
    assert!(result.is_err());
}

#[cfg(feature = "discord")]
/// Test send_message with empty channel_id
#[tokio::test]
async fn test_send_message_empty_channel_id() {
    let mut client = DiscordApiClient::new("test_bot_token".to_string());
    let channel_id = "";
    let message = "Test message";

    let result = client.send_message(channel_id, message).await;

    // Should return an error (invalid channel ID)
    assert!(result.is_err());
}

#[cfg(feature = "discord")]
/// Test send_message with very long message (over Discord's 2000 char limit)
/// This tests that the function accepts long messages - chunking should be handled separately
#[tokio::test]
async fn test_send_message_long_content() {
    let mut client = DiscordApiClient::new("test_bot_token".to_string());
    let channel_id = "123456789";
    let message = "a".repeat(3000); // 3000 character message

    // Function should accept the message (chunking is a separate concern)
    let result = client.send_message(channel_id, &message).await;

    // Should fail with network/auth error, not because of message length
    assert!(result.is_err());
}
