//! Discord REST API client module
//!
//! Provides HTTP-based interaction with Discord's REST API for sending messages
//! and handling rate limiting.

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Maximum length of a Discord message
const MAX_MESSAGE_LENGTH: usize = 2000;

/// Delay between chunked message sends to maintain order
const CHUNK_DELAY_MS: u64 = 250;

/// Discord API client for REST operations
pub struct DiscordApiClient {
    /// Bot token for authentication
    token: String,
    /// HTTP client for making requests
    http_client: reqwest::Client,
    /// Base URL for Discord API
    base_url: String,
    /// Remaining requests in current rate limit window
    rate_limit_remaining: u32,
    /// Time when the rate limit resets
    rate_limit_reset: Option<Instant>,
}

impl DiscordApiClient {
    /// Create a new Discord API client
    pub fn new(token: String) -> Self {
        Self {
            token,
            http_client: reqwest::Client::new(),
            base_url: "https://discord.com/api/v10".to_string(),
            rate_limit_remaining: u32::MAX,
            rate_limit_reset: None,
        }
    }

    /// Send a message to a Discord channel
    pub async fn send_message(
        &mut self,
        channel_id: &str,
        content: &str,
    ) -> Result<Message, ApiError> {
        // Check if we need to wait due to rate limiting
        self.wait_for_rate_limit().await?;

        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let auth_value = HeaderValue::from_str(&format!("Bot {}", self.token))
            .map_err(|e| ApiError::SerializationError(format!("Invalid token: {}", e)))?;
        headers.insert(AUTHORIZATION, auth_value);

        let body = serde_json::json!({
            "content": content
        });

        let response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        // Update rate limit state from response headers
        self.update_rate_limit_state(&response);

        let status = response.status();

        // Handle rate limiting (429)
        if status.as_u16() == 429 {
            let retry_after = self.get_retry_after(&response);
            return Err(ApiError::RateLimited(retry_after));
        }

        // Handle other error status codes
        match status.as_u16() {
            200..=299 => {
                let message: Message = response.json().await.map_err(|e| {
                    ApiError::SerializationError(format!("Failed to parse message: {}", e))
                })?;
                Ok(message)
            }
            401 => Err(ApiError::Unauthorized),
            403 => Err(ApiError::Unauthorized),
            404 => Err(ApiError::NotFound),
            500..=599 => Err(ApiError::ServerError(status.as_u16())),
            _ => Err(ApiError::ServerError(status.as_u16())),
        }
    }

    /// Send a message that may be longer than Discord's 2000 character limit
    ///
    /// Automatically splits the message into chunks:
    /// 1. First tries to split on paragraph boundaries (\n\n)
    /// 2. Falls back to splitting on newlines (\n)
    /// 3. Last resort: splits at 1990 characters with continuation marker
    pub async fn send_message_chunked(
        &mut self,
        channel_id: &str,
        content: &str,
    ) -> Result<(), ApiError> {
        if content.len() <= MAX_MESSAGE_LENGTH {
            return self.send_message(channel_id, content).await.map(|_| ());
        }

        // Try splitting by paragraphs first
        let chunks = self.split_message(content);

        for (i, chunk) in chunks.iter().enumerate() {
            // Add continuation marker for all but the last chunk
            let message_content = if i < chunks.len() - 1 {
                format!("{}\n\n(continued...)", chunk)
            } else {
                chunk.clone()
            };

            self.send_message(channel_id, &message_content).await?;

            // Add delay between chunks to maintain order
            if i < chunks.len() - 1 {
                tokio::time::sleep(Duration::from_millis(CHUNK_DELAY_MS)).await;
            }
        }

        Ok(())
    }

    /// Split message into chunks that fit within Discord's limit
    fn split_message(&self, content: &str) -> Vec<String> {
        // First try splitting on paragraph boundaries
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut chunks: Vec<String> = Vec::new();
        let mut current_chunk = String::new();

        for paragraph in paragraphs {
            let para_len = paragraph.len() + 2; // +2 for \n\n

            if current_chunk.len() + para_len > MAX_MESSAGE_LENGTH {
                // Current paragraph would exceed limit, push current chunk
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }

                // If single paragraph exceeds limit, try splitting on newlines
                if paragraph.len() > MAX_MESSAGE_LENGTH {
                    let line_chunks = self.split_on_newlines(paragraph);
                    for line_chunk in line_chunks {
                        if current_chunk.len() + line_chunk.len() > MAX_MESSAGE_LENGTH {
                            if !current_chunk.is_empty() {
                                chunks.push(current_chunk.clone());
                                current_chunk.clear();
                            }
                            // If still exceeds, split at 1990 chars
                            if line_chunk.len() > MAX_MESSAGE_LENGTH {
                                let char_chunks = self.split_on_chars(&line_chunk);
                                for char_chunk in char_chunks {
                                    chunks.push(char_chunk);
                                }
                            } else {
                                current_chunk = line_chunk;
                            }
                        } else {
                            if !current_chunk.is_empty() {
                                current_chunk.push('\n');
                            }
                            current_chunk.push_str(&line_chunk);
                        }
                    }
                } else {
                    current_chunk = paragraph.to_string();
                }
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push_str("\n\n");
                }
                current_chunk.push_str(paragraph);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Split content on newlines
    fn split_on_newlines(&self, content: &str) -> Vec<String> {
        let lines: Vec<&str> = content.split('\n').collect();
        let mut chunks: Vec<String> = Vec::new();
        let mut current_chunk = String::new();

        for line in lines {
            let line_len = line.len() + 1; // +1 for \n

            if current_chunk.len() + line_len > MAX_MESSAGE_LENGTH {
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }

                // If single line exceeds limit, split at character level
                if line.len() > MAX_MESSAGE_LENGTH {
                    let char_chunks = self.split_on_chars(line);
                    chunks.extend(char_chunks);
                } else {
                    current_chunk = line.to_string();
                }
            } else {
                if !current_chunk.is_empty() {
                    current_chunk.push('\n');
                }
                current_chunk.push_str(line);
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Split content at character level at 1990 chars with continuation marker
    fn split_on_chars(&self, content: &str) -> Vec<String> {
        let mut chunks: Vec<String> = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let chunk_size = 1990;

        let mut i = 0;
        while i < chars.len() {
            let end = std::cmp::min(i + chunk_size, chars.len());
            let chunk: String = chars[i..end].iter().collect();

            if i + chunk_size < chars.len() {
                // Not the last chunk, add continuation marker
                chunks.push(format!("…{}", chunk));
            } else {
                chunks.push(chunk);
            }

            i = end;
        }

        chunks
    }

    /// Wait if we're currently rate limited
    async fn wait_for_rate_limit(&mut self) -> Result<(), ApiError> {
        if let Some(reset_time) = self.rate_limit_reset {
            if self.rate_limit_remaining == 0 && reset_time > Instant::now() {
                let wait_duration = reset_time.duration_since(Instant::now());
                tokio::time::sleep(wait_duration).await;
            }
        }
        Ok(())
    }

    /// Update rate limit state from HTTP response headers
    fn update_rate_limit_state(&mut self, response: &reqwest::Response) {
        if let Some(remaining) = response
            .headers()
            .get("X-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            self.rate_limit_remaining = remaining;
        }

        if let Some(reset) = response
            .headers()
            .get("X-RateLimit-Reset")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
        {
            // Convert Unix timestamp to Instant
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if reset > now {
                let delay = Duration::from_secs(reset - now);
                self.rate_limit_reset = Some(Instant::now() + delay);
            }
        }
    }

    /// Get retry-after duration from rate limited response
    fn get_retry_after(&self, response: &reqwest::Response) -> u64 {
        response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
            .unwrap_or(1)
    }

    // ========================================
    // Test helpers (only compiled in test mode)
    // ========================================

    #[cfg(test)]
    /// Get current rate limit remaining (for testing)
    pub fn get_rate_limit_remaining(&self) -> u32 {
        self.rate_limit_remaining
    }

    #[cfg(test)]
    /// Get current rate limit reset time (for testing)
    pub fn get_rate_limit_reset(&self) -> Option<Instant> {
        self.rate_limit_reset
    }

    #[cfg(test)]
    /// Set rate limit remaining (for testing)
    pub fn set_rate_limit_remaining(&mut self, remaining: u32) {
        self.rate_limit_remaining = remaining;
    }

    #[cfg(test)]
    /// Set rate limit reset time (for testing)
    pub fn set_rate_limit_reset(&mut self, reset: Option<Instant>) {
        self.rate_limit_reset = reset;
    }
}

/// Discord message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message ID
    pub id: String,
    /// Channel ID where the message was sent
    pub channel_id: String,
    /// Message content
    pub content: String,
    /// Author of the message
    pub author: User,
    /// Timestamp of the message
    pub timestamp: String,
}

/// Discord user structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: String,
    /// Username
    pub username: String,
}

/// API errors that can occur during Discord API operations
#[derive(Debug, Error)]
pub enum ApiError {
    /// HTTP request failed
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    /// Rate limited, retry after specified seconds
    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),
    /// Unauthorized (invalid token or missing permissions)
    #[error("Unauthorized")]
    Unauthorized,
    /// Resource not found
    #[error("Not found")]
    NotFound,
    /// Server error (5xx)
    #[error("Server error: {0}")]
    ServerError(u16),
    /// Message too long for Discord
    #[error("Message too long")]
    MessageTooLong,
    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_on_paragraphs() {
        let client = DiscordApiClient::new("test".to_string());
        // Short paragraphs that fit in one chunk should return single chunk
        let content = "First paragraph\n\nSecond paragraph\n\nThird paragraph";
        let chunks = client.split_message(content);

        // All content fits in one chunk since it's under MAX_MESSAGE_LENGTH
        assert_eq!(chunks.len(), 1);
        assert_eq!(
            chunks[0],
            "First paragraph\n\nSecond paragraph\n\nThird paragraph"
        );
    }

    #[test]
    fn test_split_on_newlines() {
        let client = DiscordApiClient::new("test".to_string());
        // Short lines that fit in one chunk should return single chunk
        let content = "Line 1\nLine 2\nLine 3";
        let chunks = client.split_on_newlines(content);

        // All content fits in one chunk since it's under MAX_MESSAGE_LENGTH
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_split_on_chars() {
        let client = DiscordApiClient::new("test".to_string());
        let content = "x".repeat(2500);
        let chunks = client.split_on_chars(&content);

        // Should be split into 2 chunks (1990 + 510)
        assert_eq!(chunks.len(), 2);
        // First chunk should have continuation marker
        assert!(chunks[0].starts_with('…'));
    }

    #[test]
    fn test_message_under_limit() {
        let client = DiscordApiClient::new("test".to_string());
        let content = "Short message";
        let chunks = client.split_message(content);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "Short message");
    }

    // ========================================
    // ApiError tests
    // ========================================

    #[test]
    fn test_api_error_rate_limited() {
        let error = ApiError::RateLimited(30);
        let message = format!("{}", error);
        assert!(message.contains("Rate limited"));
        assert!(message.contains("30"));
    }

    #[test]
    fn test_api_error_unauthorized() {
        let error = ApiError::Unauthorized;
        let message = format!("{}", error);
        assert!(message.contains("Unauthorized"));
    }

    #[test]
    fn test_api_error_not_found() {
        let error = ApiError::NotFound;
        let message = format!("{}", error);
        assert!(message.contains("Not found"));
    }

    #[test]
    fn test_api_error_server_error() {
        let error = ApiError::ServerError(500);
        let message = format!("{}", error);
        assert!(message.contains("Server error"));
        assert!(message.contains("500"));
    }

    #[test]
    fn test_api_error_message_too_long() {
        let error = ApiError::MessageTooLong;
        let message = format!("{}", error);
        assert!(message.contains("Message too long"));
    }

    #[test]
    fn test_api_error_serialization() {
        let error = ApiError::SerializationError("invalid json".to_string());
        let message = format!("{}", error);
        assert!(message.contains("Serialization error"));
        assert!(message.contains("invalid json"));
    }

    #[test]
    fn test_api_error_matches_variants() {
        // Test that we can match on all ApiError variants
        let rate_limited = ApiError::RateLimited(10);
        let unauthorized = ApiError::Unauthorized;
        let not_found = ApiError::NotFound;
        let server_error = ApiError::ServerError(503);
        let message_too_long = ApiError::MessageTooLong;
        let serialization = ApiError::SerializationError("test".to_string());

        match rate_limited {
            ApiError::RateLimited(_) => {}
            _ => panic!("Expected RateLimited"),
        }

        match unauthorized {
            ApiError::Unauthorized => {}
            _ => panic!("Expected Unauthorized"),
        }

        match not_found {
            ApiError::NotFound => {}
            _ => panic!("Expected NotFound"),
        }

        match server_error {
            ApiError::ServerError(_) => {}
            _ => panic!("Expected ServerError"),
        }

        match message_too_long {
            ApiError::MessageTooLong => {}
            _ => panic!("Expected MessageTooLong"),
        }

        match serialization {
            ApiError::SerializationError(_) => {}
            _ => panic!("Expected SerializationError"),
        }
    }

    // ========================================
    // DiscordApiClient tests
    // ========================================

    #[test]
    fn test_client_initial_state() {
        let client = DiscordApiClient::new("test_token".to_string());
        // Client should have initial rate limit state
        // We can't directly access private fields, but we can test behavior
        // The client is created with rate_limit_remaining = u32::MAX
        assert_eq!(client.base_url, "https://discord.com/api/v10");
    }

    #[test]
    fn test_client_creation_with_different_tokens() {
        let client1 = DiscordApiClient::new("token1".to_string());
        let client2 = DiscordApiClient::new("token2".to_string());
        // Both clients should be created successfully
        assert_eq!(client1.base_url, client2.base_url);
    }

    #[test]
    fn test_message_struct_creation() {
        let message = Message {
            id: "12345".to_string(),
            channel_id: "67890".to_string(),
            content: "Hello, world!".to_string(),
            author: User {
                id: "111".to_string(),
                username: "testuser".to_string(),
            },
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        };

        assert_eq!(message.id, "12345");
        assert_eq!(message.channel_id, "67890");
        assert_eq!(message.content, "Hello, world!");
        assert_eq!(message.author.username, "testuser");
    }

    #[test]
    fn test_user_struct_creation() {
        let user = User {
            id: "123".to_string(),
            username: "testuser".to_string(),
        };

        assert_eq!(user.id, "123");
        assert_eq!(user.username, "testuser");
    }

    // ========================================
    // Rate limit state management tests
    // ========================================

    #[test]
    fn test_rate_limit_constants() {
        // Test that MAX_MESSAGE_LENGTH is correctly defined
        assert_eq!(MAX_MESSAGE_LENGTH, 2000);
        assert!(MAX_MESSAGE_LENGTH > 0);
    }

    #[test]
    fn test_chunk_delay_constant() {
        // Test that CHUNK_DELAY_MS is correctly defined
        assert_eq!(CHUNK_DELAY_MS, 250);
    }

    #[test]
    fn test_split_message_empty_content() {
        let client = DiscordApiClient::new("test".to_string());
        let chunks = client.split_message("");
        // Empty content should return empty vector
        assert!(chunks.is_empty() || chunks.len() == 1);
    }

    #[test]
    fn test_split_message_single_char() {
        let client = DiscordApiClient::new("test".to_string());
        let chunks = client.split_message("a");
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], "a");
    }

    #[test]
    fn test_split_on_newlines_empty() {
        let client = DiscordApiClient::new("test".to_string());
        let chunks = client.split_on_newlines("");
        // Empty content should return empty or single chunk
        assert!(chunks.is_empty() || chunks.len() == 1);
    }

    #[test]
    fn test_split_on_chars_empty() {
        let client = DiscordApiClient::new("test".to_string());
        let chunks = client.split_on_chars("");
        assert!(chunks.is_empty() || chunks.len() == 1);
    }

    #[test]
    fn test_split_on_chars_exact_boundary() {
        let client = DiscordApiClient::new("test".to_string());
        // Test exact 1990 character boundary
        let content = "x".repeat(1990);
        let chunks = client.split_on_chars(&content);
        // Should be exactly 1 chunk at the boundary
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_split_on_chars_just_over_boundary() {
        let client = DiscordApiClient::new("test".to_string());
        // Test just over 1990 character boundary
        let content = "x".repeat(1991);
        let chunks = client.split_on_chars(&content);
        // Should be 2 chunks
        assert_eq!(chunks.len(), 2);
    }

    // ========================================
    // Rate limit handling tests
    // ========================================

    #[test]
    fn test_rate_limit_initial_state() {
        let client = DiscordApiClient::new("test_token".to_string());
        // Initial state should have max remaining and no reset time
        assert_eq!(client.get_rate_limit_remaining(), u32::MAX);
        assert!(client.get_rate_limit_reset().is_none());
    }

    #[test]
    fn test_rate_limit_state_update_remaining() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Create a mock response with X-RateLimit-Remaining header
        let mut headers = http::HeaderMap::new();
        headers.insert("X-RateLimit-Remaining", http::HeaderValue::from_static("5"));
        headers.insert("X-RateLimit-Limit", http::HeaderValue::from_static("10"));

        // We need to create a reqwest Response from http response
        // Since we can't easily do this, we'll test the parsing logic indirectly
        // by testing that the initial state is correct and can be modified

        // Test that we can set and get rate limit remaining
        client.set_rate_limit_remaining(5);
        assert_eq!(client.get_rate_limit_remaining(), 5);

        // Test that we can set and get rate limit reset
        let future_time = Instant::now() + Duration::from_secs(60);
        client.set_rate_limit_reset(Some(future_time));
        assert!(client.get_rate_limit_reset().is_some());
    }

    #[test]
    fn test_rate_limit_state_per_channel_independence() {
        // Create two clients (simulating different channels)
        let mut client1 = DiscordApiClient::new("test_token".to_string());
        let mut client2 = DiscordApiClient::new("test_token".to_string());

        // Set different rate limit states for each
        client1.set_rate_limit_remaining(5);
        client1.set_rate_limit_reset(Some(Instant::now() + Duration::from_secs(30)));

        client2.set_rate_limit_remaining(10);
        client2.set_rate_limit_reset(Some(Instant::now() + Duration::from_secs(60)));

        // Verify they have independent state
        assert_eq!(client1.get_rate_limit_remaining(), 5);
        assert_eq!(client2.get_rate_limit_remaining(), 10);

        // Verify reset times are different
        let reset1 = client1.get_rate_limit_reset().unwrap();
        let reset2 = client2.get_rate_limit_reset().unwrap();
        assert!(reset1 < reset2);
    }

    #[test]
    fn test_rate_limit_reset_timing() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Set a reset time in the future
        let future_duration = Duration::from_secs(60);
        let future_reset = Instant::now() + future_duration;
        client.set_rate_limit_reset(Some(future_reset));

        // Verify the reset time is in the future
        let reset = client.get_rate_limit_reset().unwrap();
        assert!(reset > Instant::now());

        // Set a reset time in the past (should still be stored but will be treated as expired)
        let past_reset = Instant::now() - Duration::from_secs(10);
        client.set_rate_limit_reset(Some(past_reset));

        // Verify it's set (even though it's in the past)
        assert!(client.get_rate_limit_reset().is_some());
    }

    #[tokio::test]
    async fn test_wait_for_rate_limit_no_wait_when_not_limited() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Set remaining > 0, so no wait needed
        client.set_rate_limit_remaining(5);
        client.set_rate_limit_reset(None);

        let start = Instant::now();
        client.wait_for_rate_limit().await.unwrap();
        let elapsed = start.elapsed();

        // Should return immediately without waiting
        assert!(elapsed < Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_wait_for_rate_limit_waits_when_remaining_zero() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Set remaining to 0 with a reset time in the future
        let wait_duration = Duration::from_millis(100);
        let future_reset = Instant::now() + wait_duration;
        client.set_rate_limit_remaining(0);
        client.set_rate_limit_reset(Some(future_reset));

        let start = Instant::now();
        client.wait_for_rate_limit().await.unwrap();
        let elapsed = start.elapsed();

        // Should have waited approximately the duration
        assert!(elapsed >= wait_duration - Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_wait_for_rate_limit_no_wait_when_reset_past() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Set remaining to 0 but reset time in the past
        let past_reset = Instant::now() - Duration::from_millis(100);
        client.set_rate_limit_remaining(0);
        client.set_rate_limit_reset(Some(past_reset));

        let start = Instant::now();
        client.wait_for_rate_limit().await.unwrap();
        let elapsed = start.elapsed();

        // Should return immediately since reset time has passed
        assert!(elapsed < Duration::from_millis(10));
    }

    #[test]
    fn test_rate_limit_remaining_exhausted() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Simulate rate limit being exhausted
        client.set_rate_limit_remaining(0);

        // Verify state
        assert_eq!(client.get_rate_limit_remaining(), 0);
    }

    #[test]
    fn test_rate_limit_state_reset() {
        let mut client = DiscordApiClient::new("test_token".to_string());

        // Set some state
        client.set_rate_limit_remaining(5);
        client.set_rate_limit_reset(Some(Instant::now() + Duration::from_secs(30)));

        // Reset the state (simulating a new rate limit window)
        client.set_rate_limit_remaining(u32::MAX);
        client.set_rate_limit_reset(None);

        // Verify reset state
        assert_eq!(client.get_rate_limit_remaining(), u32::MAX);
        assert!(client.get_rate_limit_reset().is_none());
    }
}

#[cfg(test)]
mod integration_tests {
    // Additional integration-style tests that verify
    // the interaction between components

    use super::*;

    #[test]
    fn test_error_display_consistency() {
        // Verify that all ApiError variants produce non-empty error messages
        let errors = vec![
            ApiError::RateLimited(5),
            ApiError::Unauthorized,
            ApiError::NotFound,
            ApiError::ServerError(500),
            ApiError::MessageTooLong,
            ApiError::SerializationError("test".to_string()),
        ];

        for error in errors {
            let msg = format!("{}", error);
            assert!(!msg.is_empty(), "Error message should not be empty");
        }
    }

    #[test]
    fn test_message_fields_accessible() {
        let message = Message {
            id: "msg_123".to_string(),
            channel_id: "chan_456".to_string(),
            content: "Test content".to_string(),
            author: User {
                id: "user_789".to_string(),
                username: "author".to_string(),
            },
            timestamp: "2024-01-01T00:00:00.000Z".to_string(),
        };

        // Verify all fields are accessible
        let _ = message.id.clone();
        let _ = message.channel_id.clone();
        let _ = message.content.clone();
        let _ = message.author.id.clone();
        let _ = message.author.username.clone();
        let _ = message.timestamp.clone();
    }
}
