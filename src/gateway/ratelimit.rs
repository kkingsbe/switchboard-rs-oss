//! Rate limiting module for gateway per-channel rate limiting.
//!
//! This module provides rate limiting functionality to prevent exceeding
//! Discord's API rate limits on a per-channel basis.

#![allow(private_interfaces)]

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time;
use tracing::{debug, warn};

/// Maximum number of requests allowed per rate limit window.
const MAX_REQUESTS_PER_WINDOW: u32 = 5;

/// Duration of the rate limit window in seconds.
const RATE_LIMIT_WINDOW_SECS: u64 = 5;

/// Maximum backoff time in seconds.
const MAX_BACKOFF_SECS: u64 = 60;

/// Initial backoff time in seconds.
const INITIAL_BACKOFF_SECS: u64 = 1;

/// Error types for rate limiting operations.
#[derive(Debug, Error)]
pub enum RateLimitError {
    /// Rate limit exceeded, must wait before retrying.
    #[error("Rate limit exceeded for channel {channel_id}, retry after {retry_after_secs}s")]
    RateLimitExceeded {
        /// The channel ID that hit the rate limit.
        channel_id: u64,
        /// Seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// Invalid channel ID.
    #[error("Invalid channel ID: {0}")]
    InvalidChannelId(String),

    /// Internal rate limiter error.
    #[error("Rate limiter error: {0}")]
    InternalError(String),
}

/// Per-channel rate limit state.
#[derive(Debug, Clone)]
struct ChannelState {
    /// Number of requests in current window.
    request_count: u32,
    /// When the current window started.
    window_start: Instant,
    /// Current backoff time in seconds (for exponential backoff).
    backoff_secs: u64,
    /// Number of consecutive 429 responses.
    consecutive_429s: u32,
    /// Override wait time from Discord's Retry-After header.
    /// This takes precedence over backoff_secs when set.
    retry_after_secs: Option<u64>,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self {
            request_count: 0,
            window_start: Instant::now(),
            backoff_secs: INITIAL_BACKOFF_SECS,
            consecutive_429s: 0,
            retry_after_secs: None,
        }
    }
}

/// Rate limiter for tracking per-channel request rates.
///
/// This rate limiter tracks requests per Discord channel and handles
/// 429 responses with exponential backoff.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Per-channel rate limit state.
    channels: Arc<RwLock<HashMap<u64, ChannelState>>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    /// Create a new RateLimiter.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let limiter = RateLimiter::new();
    /// ```
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a request to the given channel is allowed.
    ///
    /// If the rate limit is exceeded, this method returns an error with
    /// the number of seconds to wait before retrying.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID to check
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Request is allowed
    /// * `Err(RateLimitError::RateLimitExceeded)` - Rate limit exceeded, must wait
    pub async fn check_rate_limit(&self, channel_id: u64) -> Result<(), RateLimitError> {
        if channel_id == 0 {
            return Err(RateLimitError::InvalidChannelId(
                "Channel ID cannot be zero".to_string(),
            ));
        }

        let mut channels = self.channels.write().await;
        let state = channels.entry(channel_id).or_default();

        // Check if we're in a backoff period from previous 429s
        if state.consecutive_429s > 0 {
            // Use retry_after_secs if set (from Discord's Retry-After header), otherwise use backoff
            let wait_time = state.retry_after_secs.unwrap_or(state.backoff_secs);
            debug!(
                "Channel {} in backoff period, waiting {}s ({} consecutive 429s)",
                channel_id, wait_time, state.consecutive_429s
            );
            return Err(RateLimitError::RateLimitExceeded {
                channel_id,
                retry_after_secs: wait_time,
            });
        }

        // Check if current window has expired
        let elapsed = state.window_start.elapsed();
        if elapsed >= Duration::from_secs(RATE_LIMIT_WINDOW_SECS) {
            // Reset the window
            state.request_count = 0;
            state.window_start = Instant::now();
            debug!("Channel {} rate limit window reset", channel_id);
        }

        // Check if we've exceeded the rate limit
        if state.request_count >= MAX_REQUESTS_PER_WINDOW {
            let retry_after = RATE_LIMIT_WINDOW_SECS - elapsed.as_secs();
            warn!(
                "Rate limit exceeded for channel {}: {}/{} requests in window",
                channel_id, state.request_count, MAX_REQUESTS_PER_WINDOW
            );
            return Err(RateLimitError::RateLimitExceeded {
                channel_id,
                retry_after_secs: retry_after.max(1),
            });
        }

        Ok(())
    }

    /// Record a successful request for the given channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID
    pub async fn record_request(&self, channel_id: u64) {
        if channel_id == 0 {
            return;
        }

        let mut channels = self.channels.write().await;
        let state = channels.entry(channel_id).or_default();

        // Check if window has expired
        let elapsed = state.window_start.elapsed();
        if elapsed >= Duration::from_secs(RATE_LIMIT_WINDOW_SECS) {
            state.request_count = 0;
            state.window_start = Instant::now();
        }

        state.request_count += 1;
        debug!(
            "Recorded request for channel {}, count: {}/{}",
            channel_id, state.request_count, MAX_REQUESTS_PER_WINDOW
        );

        // Reset backoff on successful request
        if state.consecutive_429s > 0 {
            state.consecutive_429s = 0;
            state.backoff_secs = INITIAL_BACKOFF_SECS;
            state.retry_after_secs = None;
            debug!(
                "Channel {} backoff reset after successful request",
                channel_id
            );
        }
    }

    /// Handle a 429 (Too Many Requests) response for the given channel.
    ///
    /// This method implements exponential backoff by increasing the wait time
    /// with each consecutive 429 response.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID
    /// * `retry_after` - Optional retry-after value from response header (in seconds)
    ///
    /// # Returns
    ///
    /// * The number of seconds to wait before retrying
    pub async fn handle_429(&self, channel_id: u64, retry_after: Option<u64>) -> u64 {
        if channel_id == 0 {
            return 0;
        }

        let mut channels = self.channels.write().await;
        let state = channels.entry(channel_id).or_default();

        state.consecutive_429s += 1;

        // Use the retry-after header if provided, otherwise use current backoff
        let wait_time = retry_after.unwrap_or(state.backoff_secs);
        
        // Store the retry-after value if provided, so check_rate_limit can use it
        // If retry_after is not provided, clear the stored value so we use backoff
        if retry_after.is_some() {
            state.retry_after_secs = retry_after;
        } else {
            state.retry_after_secs = None;
        }

        warn!(
            "Received 429 for channel {}, consecutive: {}, retry_after: {:?}, using wait: {}s",
            channel_id, state.consecutive_429s, retry_after, wait_time
        );

        // Increase backoff for next time (exponential backoff)
        // Only double backoff when NOT using a retry-after value from server
        if retry_after.is_none() {
            state.backoff_secs = (state.backoff_secs * 2).min(MAX_BACKOFF_SECS);
        }

        debug!(
            "Backoff set to {}s for channel {}",
            state.backoff_secs, channel_id
        );

        wait_time
    }

    /// Get the current backoff time for a channel.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID
    ///
    /// # Returns
    ///
    /// * The current backoff time in seconds
    pub async fn get_backoff(&self, channel_id: u64) -> u64 {
        let channels = self.channels.read().await;
        channels
            .get(&channel_id)
            .map(|s| s.backoff_secs)
            .unwrap_or(INITIAL_BACKOFF_SECS)
    }

    /// Wait for rate limit to allow a request.
    ///
    /// If rate limited, this method will sleep for the appropriate duration.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Request is now allowed
    /// * `Err(RateLimitError)` - Error (rate limit exceeded or invalid channel)
    pub async fn wait_for_rate_limit(&self, channel_id: u64) -> Result<(), RateLimitError> {
        // Check and wait in a loop until we can proceed
        loop {
            match self.check_rate_limit(channel_id).await {
                Ok(()) => return Ok(()),
                Err(RateLimitError::RateLimitExceeded {
                    channel_id: _,
                    retry_after_secs,
                }) => {
                    debug!("Rate limited, waiting {}s before retry", retry_after_secs);
                    time::sleep(Duration::from_secs(retry_after_secs)).await;
                    // After sleeping, check again (window may have reset)
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Get the current state for debugging purposes.
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID
    ///
    /// # Returns
    ///
    /// * Some(state) if channel exists, None otherwise
    pub async fn get_state(&self, channel_id: u64) -> Option<ChannelState> {
        let channels = self.channels.read().await;
        channels.get(&channel_id).cloned()
    }

    /// Reset rate limit state for a channel (for testing).
    ///
    /// # Arguments
    ///
    /// * `channel_id` - The Discord channel ID to reset
    pub async fn reset_channel(&self, channel_id: u64) {
        let mut channels = self.channels.write().await;
        channels.remove(&channel_id);
        debug!("Rate limit state reset for channel {}", channel_id);
    }

    /// Reset all rate limit state (for testing).
    pub async fn reset_all(&self) {
        let mut channels = self.channels.write().await;
        channels.clear();
        debug!("All rate limit state reset");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that rate limit counter increments correctly.
    #[tokio::test]
    async fn test_rate_limit_counter_increments() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // First request should succeed
        assert!(limiter.check_rate_limit(channel_id).await.is_ok());
        limiter.record_request(channel_id).await;

        // Subsequent requests should succeed up to the limit
        for _ in 1..MAX_REQUESTS_PER_WINDOW {
            assert!(limiter.check_rate_limit(channel_id).await.is_ok());
            limiter.record_request(channel_id).await;
        }

        // Next request should fail (rate limit exceeded)
        let result = limiter.check_rate_limit(channel_id).await;
        assert!(result.is_err());
        if let Err(RateLimitError::RateLimitExceeded {
            channel_id: _,
            retry_after_secs,
        }) = result
        {
            assert!(retry_after_secs > 0);
        } else {
            panic!("Expected RateLimitExceeded error");
        }
    }

    /// Test that rate limit counter resets after window expires.
    #[tokio::test]
    async fn test_rate_limit_resets_after_window() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Exhaust the rate limit
        for _ in 0..MAX_REQUESTS_PER_WINDOW {
            assert!(limiter.check_rate_limit(channel_id).await.is_ok());
            limiter.record_request(channel_id).await;
        }

        // Should be rate limited
        assert!(limiter.check_rate_limit(channel_id).await.is_err());

        // Reset the channel state to simulate window expiration
        limiter.reset_channel(channel_id).await;

        // Should be allowed again
        assert!(limiter.check_rate_limit(channel_id).await.is_ok());
    }

    /// Test 429 response handling with retry-after header.
    #[tokio::test]
    async fn test_handle_429_with_retry_after() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Handle a 429 with retry-after of 5 seconds
        let wait_time = limiter.handle_429(channel_id, Some(5)).await;
        assert_eq!(wait_time, 5);

        // Should now be in backoff mode
        let result = limiter.check_rate_limit(channel_id).await;
        assert!(result.is_err());
        if let Err(RateLimitError::RateLimitExceeded {
            channel_id: _,
            retry_after_secs,
        }) = result
        {
            assert_eq!(retry_after_secs, 5); // Should use the retry-after value
        } else {
            panic!("Expected RateLimitExceeded error");
        }
    }

    /// Test exponential backoff increases on continued 429s.
    #[tokio::test]
    async fn test_exponential_backoff_increases() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Handle first 429 (no retry-after, use backoff)
        let wait1 = limiter.handle_429(channel_id, None).await;
        let backoff1 = limiter.get_backoff(channel_id).await;
        assert_eq!(wait1, INITIAL_BACKOFF_SECS); // Initial backoff is 1s
        assert_eq!(backoff1, INITIAL_BACKOFF_SECS * 2); // Backoff doubled for next time

        // Handle second 429 - backoff should double again
        let wait2 = limiter.handle_429(channel_id, None).await;
        let backoff2 = limiter.get_backoff(channel_id).await;
        assert_eq!(wait2, INITIAL_BACKOFF_SECS * 2); // 2s
        assert_eq!(backoff2, INITIAL_BACKOFF_SECS * 4); // Backoff doubled again

        // Handle third 429 - backoff should double again
        let wait3 = limiter.handle_429(channel_id, None).await;
        let backoff3 = limiter.get_backoff(channel_id).await;
        assert_eq!(wait3, INITIAL_BACKOFF_SECS * 4); // 4s
        assert_eq!(backoff3, INITIAL_BACKOFF_SECS * 8); // Backoff doubled again

        // Handle fourth 429 - backoff should double again
        let wait4 = limiter.handle_429(channel_id, None).await;
        let backoff4 = limiter.get_backoff(channel_id).await;
        assert_eq!(wait4, INITIAL_BACKOFF_SECS * 8); // 8s
        assert_eq!(backoff4, INITIAL_BACKOFF_SECS * 16); // Backoff doubled again
    }

    /// Test that backoff is capped at max value.
    #[tokio::test]
    async fn test_backoff_capped_at_max() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Handle many 429s to exceed max backoff
        for _ in 0..10 {
            limiter.handle_429(channel_id, None).await;
        }

        let backoff = limiter.get_backoff(channel_id).await;
        assert_eq!(backoff, MAX_BACKOFF_SECS);
    }

    /// Test that successful request resets backoff.
    #[tokio::test]
    async fn test_successful_request_resets_backoff() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Cause some 429s to build up backoff
        limiter.handle_429(channel_id, None).await;
        limiter.handle_429(channel_id, None).await;

        // Record a successful request
        limiter.record_request(channel_id).await;

        // Backoff should be reset
        let backoff = limiter.get_backoff(channel_id).await;
        assert_eq!(backoff, INITIAL_BACKOFF_SECS);
    }

    /// Test invalid channel ID handling.
    #[tokio::test]
    async fn test_invalid_channel_id() {
        let limiter = RateLimiter::new();

        // Channel ID 0 should be invalid
        let result = limiter.check_rate_limit(0).await;
        assert!(result.is_err());
        if let Err(RateLimitError::InvalidChannelId(msg)) = result {
            assert!(msg.contains("zero"));
        } else {
            panic!("Expected InvalidChannelId error");
        }
    }

    /// Test wait_for_rate_limit succeeds when not rate limited.
    #[tokio::test]
    async fn test_wait_for_rate_limit_allows_immediately() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Should succeed immediately
        let result = limiter.wait_for_rate_limit(channel_id).await;
        assert!(result.is_ok());
    }

    /// Test get_state returns correct state.
    #[tokio::test]
    async fn test_get_state() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Initially no state
        let state = limiter.get_state(channel_id).await;
        assert!(state.is_none());

        // Record a request
        limiter.record_request(channel_id).await;

        // Now should have state
        let state = limiter.get_state(channel_id).await;
        assert!(state.is_some());
        assert_eq!(state.unwrap().request_count, 1);
    }

    /// Test reset_all clears all state.
    #[tokio::test]
    async fn test_reset_all() {
        let limiter = RateLimiter::new();
        let channel_id = 12345u64;

        // Record some requests
        limiter.record_request(channel_id).await;
        limiter.handle_429(channel_id, None).await;

        // Reset all
        limiter.reset_all().await;

        // State should be cleared
        let state = limiter.get_state(channel_id).await;
        assert!(state.is_none());
    }
}
