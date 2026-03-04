//! Reconnection module for handling gateway connection reconnection with exponential backoff
//!
//! This module provides components for managing reconnection attempts when a gateway
//! connection is lost, including configurable backoff strategies and retry limits.

use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, info, warn};

/// Errors that can occur during reconnection operations
#[derive(Error, Debug)]
pub enum ReconnectionError {
    #[error("Maximum retry attempts ({max_retries}) exceeded for project: {project_id}")]
    MaxRetriesExceeded {
        /// The project ID that exceeded max retries
        project_id: String,
        /// The maximum number of retries configured
        max_retries: u32,
    },

    #[error("Reconnection cancelled for project: {project_id}")]
    ReconnectionCancelled {
        /// The project ID whose reconnection was cancelled
        project_id: String,
    },

    #[error("Reconnection aborted: {0}")]
    Aborted(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for reconnection operations
pub type ReconnectionResult<T> = Result<T, ReconnectionError>;

/// Configuration for reconnection behavior
///
/// Controls how the reconnection manager handles retry attempts when
/// connections are lost.
#[derive(Debug, Clone)]
pub struct ReconnectionConfig {
    /// Initial delay before the first reconnection attempt
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts
    pub max_delay: Duration,
    /// Maximum number of reconnection attempts before giving up
    pub max_retries: u32,
    /// Multiplier for exponential backoff calculation
    pub multiplier: f64,
}

impl Default for ReconnectionConfig {
    /// Create a default reconnection configuration
    ///
    /// Default values:
    /// - initial_delay: 1 second
    /// - max_delay: 60 seconds
    /// - max_retries: 10 attempts
    /// - multiplier: 2.0 (exponential)
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_retries: 10,
            multiplier: 2.0,
        }
    }
}

impl ReconnectionConfig {
    /// Create a new ReconnectionConfig with custom values
    ///
    /// # Arguments
    /// * `initial_delay` - Initial delay before first retry
    /// * `max_delay` - Maximum delay between retries
    /// * `max_retries` - Maximum number of retry attempts
    /// * `multiplier` - Exponential backoff multiplier
    ///
    /// # Errors
    /// Returns an error if any value is invalid (e.g., zero duration, zero multiplier)
    pub fn new(
        initial_delay: Duration,
        max_delay: Duration,
        max_retries: u32,
        multiplier: f64,
    ) -> ReconnectionResult<Self> {
        if initial_delay.is_zero() {
            return Err(ReconnectionError::InvalidConfig(
                "initial_delay must be non-zero".to_string(),
            ));
        }
        if max_delay.is_zero() {
            return Err(ReconnectionError::InvalidConfig(
                "max_delay must be non-zero".to_string(),
            ));
        }
        if initial_delay > max_delay {
            return Err(ReconnectionError::InvalidConfig(
                "initial_delay cannot be greater than max_delay".to_string(),
            ));
        }
        if multiplier <= 0.0 {
            return Err(ReconnectionError::InvalidConfig(
                "multiplier must be positive".to_string(),
            ));
        }

        Ok(Self {
            initial_delay,
            max_delay,
            max_retries,
            multiplier,
        })
    }
}

/// Exponential backoff calculator for determining retry delays
///
/// Calculates the delay for the next reconnection attempt using the formula:
/// delay = min(initial_delay * multiplier^attempt, max_delay)
#[derive(Debug, Clone)]
pub struct Backoff {
    /// Current attempt number (0-indexed)
    attempt: u32,
    /// Configuration for backoff calculation
    config: ReconnectionConfig,
}

impl Backoff {
    /// Create a new Backoff with the given configuration
    ///
    /// The backoff starts at attempt 0.
    pub fn new(config: ReconnectionConfig) -> Self {
        Self { attempt: 0, config }
    }

    /// Create a new Backoff with default configuration
    pub fn default_backoff() -> Self {
        Self::new(ReconnectionConfig::default())
    }

    /// Get the delay for the current attempt
    ///
    /// Uses the formula: min(initial_delay * multiplier^attempt, max_delay)
    pub fn current_delay(&self) -> Duration {
        let delay_secs = self.config.initial_delay.as_secs_f64()
            * self.config.multiplier.powi(self.attempt as i32);

        // Cap at max_delay
        let delay_secs = delay_secs.min(self.config.max_delay.as_secs_f64());

        // Ensure at least 1ms to avoid zero duration
        Duration::from_secs_f64(delay_secs.max(0.001))
    }

    /// Get the delay for the next attempt and advance the counter
    ///
    /// This method calculates the delay for the next attempt number (current + 1),
    /// then increments the attempt counter for subsequent calls.
    pub fn next_delay(&mut self) -> Duration {
        self.attempt += 1;
        self.current_delay()
    }

    /// Get the current attempt number
    pub fn attempt(&self) -> u32 {
        self.attempt
    }

    /// Reset the backoff to the initial state (attempt 0)
    pub fn reset(&mut self) {
        self.attempt = 0;
    }

    /// Check if maximum retries have been reached
    ///
    /// Returns true if the current attempt exceeds max_retries
    pub fn is_max_retries_exceeded(&self) -> bool {
        self.attempt >= self.config.max_retries
    }

    /// Get the configuration
    pub fn config(&self) -> &ReconnectionConfig {
        &self.config
    }
}

/// Callback type for reconnection attempts
///
/// The callback receives the current attempt number and should return
/// true if reconnection was successful, false otherwise.
pub type ReconnectCallback = Box<dyn Fn(u32) -> bool + Send + Sync>;

/// Callback type for async reconnection attempts
pub type AsyncReconnectCallback =
    Box<dyn Fn(u32) -> futures_util::future::BoxFuture<'static, bool> + Send + Sync>;

/// State of a reconnection attempt
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconnectionState {
    /// No reconnection in progress
    Idle,
    /// Actively attempting to reconnect
    Reconnecting,
    /// Successfully reconnected
    Reconnected,
    /// Failed to reconnect after all retries
    Failed,
    /// Reconnection was cancelled
    Cancelled,
}

impl Default for ReconnectionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Manager for handling reconnection attempts
///
/// Tracks the state of reconnection attempts and provides async methods
/// for attempting to reconnect with exponential backoff.
#[derive(Debug)]
pub struct ReconnectionManager {
    /// Configuration for reconnection behavior
    config: ReconnectionConfig,
    /// Current state of reconnection
    state: ReconnectionState,
    /// Current retry count
    retry_count: u32,
    /// Project ID being reconnected
    project_id: String,
}

impl ReconnectionManager {
    /// Create a new ReconnectionManager
    ///
    /// # Arguments
    /// * `project_id` - The project ID to reconnect
    /// * `config` - Configuration for reconnection behavior
    pub fn new(project_id: String, config: ReconnectionConfig) -> Self {
        Self {
            config,
            state: ReconnectionState::Idle,
            retry_count: 0,
            project_id,
        }
    }

    /// Create a new ReconnectionManager with default configuration
    pub fn with_default_config(project_id: String) -> Self {
        Self::new(project_id, ReconnectionConfig::default())
    }

    /// Get the current state
    pub fn state(&self) -> &ReconnectionState {
        &self.state
    }

    /// Get the current retry count
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Get the project ID
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Reset the manager to initial state
    pub fn reset(&mut self) {
        self.state = ReconnectionState::Idle;
        self.retry_count = 0;
    }

    /// Attempt to reconnect with exponential backoff
    ///
    /// This method will:
    /// 1. Set state to Reconnecting
    /// 2. Attempt reconnection up to max_retries times
    /// 3. Wait with exponential backoff between attempts
    /// 4. Return success if callback returns true
    ///
    /// # Arguments
    /// * `callback` - Callback function that attempts reconnection
    ///
    /// # Returns
    /// * `Ok(true)` - Reconnection successful
    /// * `Ok(false)` - Reconnection failed but within retry limit
    /// * `Err(ReconnectionError::MaxRetriesExceeded)` - All retries exhausted
    /// * `Err(ReconnectionError::ReconnectionCancelled)` - Reconnection was cancelled
    pub async fn attempt_reconnection<F, Fut>(&mut self, callback: F) -> ReconnectionResult<bool>
    where
        F: Fn(u32) -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        self.state = ReconnectionState::Reconnecting;
        self.retry_count = 0;

        let mut backoff = Backoff::new(self.config.clone());

        while backoff.attempt() < self.config.max_retries {
            let current_attempt = backoff.attempt();
            self.retry_count = current_attempt + 1;

            info!(
                target: "gateway::reconnection",
                project_id = %self.project_id,
                attempt = self.retry_count,
                max_retries = self.config.max_retries,
                "Attempting reconnection"
            );

            // Attempt reconnection
            let success = callback(current_attempt).await;

            if success {
                self.state = ReconnectionState::Reconnected;
                info!(
                    target: "gateway::reconnection",
                    project_id = %self.project_id,
                    attempt = self.retry_count,
                    "Reconnection successful"
                );
                return Ok(true);
            }

            debug!(
                target: "gateway::reconnection",
                project_id = %self.project_id,
                attempt = self.retry_count,
                "Reconnection attempt failed"
            );

            // Calculate delay for next attempt
            let delay = backoff.next_delay();

            // Check if we've exceeded max retries
            if backoff.attempt() >= self.config.max_retries {
                break;
            }

            info!(
                target: "gateway::reconnection",
                project_id = %self.project_id,
                attempt = self.retry_count,
                delay_secs = delay.as_secs_f64(),
                "Waiting before next reconnection attempt"
            );

            // Wait before next attempt
            sleep(delay).await;
        }

        // All retries exhausted
        self.state = ReconnectionState::Failed;
        warn!(
            target: "gateway::reconnection",
            project_id = %self.project_id,
            retries = self.retry_count,
            "Max reconnection attempts exceeded"
        );

        Err(ReconnectionError::MaxRetriesExceeded {
            project_id: self.project_id.clone(),
            max_retries: self.config.max_retries,
        })
    }

    /// Attempt reconnection with a synchronous callback
    ///
    /// This is a convenience method that wraps a sync callback in an async context.
    pub async fn attempt_reconnection_sync<C>(&mut self, callback: C) -> ReconnectionResult<bool>
    where
        C: Fn(u32) -> bool + Send + Sync + Clone,
    {
        let callback = callback.clone();
        self.attempt_reconnection(move |attempt| {
            let cb = callback.clone();
            async move { cb(attempt) }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test default ReconnectionConfig values
    #[test]
    fn test_default_config() {
        let config = ReconnectionConfig::default();

        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.max_retries, 10);
        assert_eq!(config.multiplier, 2.0);
    }

    /// Test ReconnectionConfig::new with valid values
    #[test]
    fn test_config_new_valid() {
        let config = ReconnectionConfig::new(
            Duration::from_secs(2),
            Duration::from_secs(30),
            5,
            1.5,
        )
        .expect("Failed to create config");

        assert_eq!(config.initial_delay, Duration::from_secs(2));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.multiplier, 1.5);
    }

    /// Test ReconnectionConfig::new rejects zero initial_delay
    #[test]
    fn test_config_new_rejects_zero_initial_delay() {
        let result = ReconnectionConfig::new(
            Duration::ZERO,
            Duration::from_secs(60),
            10,
            2.0,
        );

        assert!(result.is_err());
        assert!(
            matches!(result, Err(ReconnectionError::InvalidConfig(_))),
            "Expected InvalidConfig error"
        );
    }

    /// Test ReconnectionConfig::new rejects zero max_delay
    #[test]
    fn test_config_new_rejects_zero_max_delay() {
        let result = ReconnectionConfig::new(
            Duration::from_secs(1),
            Duration::ZERO,
            10,
            2.0,
        );

        assert!(result.is_err());
        assert!(
            matches!(result, Err(ReconnectionError::InvalidConfig(_))),
            "Expected InvalidConfig error"
        );
    }

    /// Test ReconnectionConfig::new rejects initial_delay > max_delay
    #[test]
    fn test_config_new_rejects_invalid_delay_order() {
        let result = ReconnectionConfig::new(
            Duration::from_secs(60),
            Duration::from_secs(30),
            10,
            2.0,
        );

        assert!(result.is_err());
        assert!(
            matches!(result, Err(ReconnectionError::InvalidConfig(_))),
            "Expected InvalidConfig error"
        );
    }

    /// Test ReconnectionConfig::new rejects non-positive multiplier
    #[test]
    fn test_config_new_rejects_invalid_multiplier() {
        let result = ReconnectionConfig::new(
            Duration::from_secs(1),
            Duration::from_secs(60),
            10,
            0.0,
        );

        assert!(result.is_err());
        assert!(
            matches!(result, Err(ReconnectionError::InvalidConfig(_))),
            "Expected InvalidConfig error"
        );
    }

    /// Test Backoff calculation: 1s, 2s, 4s progression
    #[test]
    fn test_backoff_progression() {
        let config = ReconnectionConfig::default();
        let mut backoff = Backoff::new(config);

        // First attempt: 1s * 2^0 = 1s
        let delay1 = backoff.current_delay();
        assert_eq!(delay1, Duration::from_secs(1));

        // Second attempt: 1s * 2^1 = 2s
        let delay2 = backoff.next_delay();
        assert_eq!(delay2, Duration::from_secs(2));

        // Third attempt: 1s * 2^2 = 4s
        let delay3 = backoff.next_delay();
        assert_eq!(delay3, Duration::from_secs(4));

        // Fourth attempt: 1s * 2^3 = 8s
        let delay4 = backoff.next_delay();
        assert_eq!(delay4, Duration::from_secs(8));
    }

    /// Test Backoff max delay cap at 60s
    #[test]
    fn test_backoff_max_delay_cap() {
        let config = ReconnectionConfig::default();
        let mut backoff = Backoff::new(config);

        // Progress through attempts until we hit max_delay
        // At attempt 6: 1s * 2^6 = 64s, should cap at 60s
        for _ in 0..10 {
            let delay = backoff.next_delay();
            assert!(
                delay <= Duration::from_secs(60),
                "Delay {} should not exceed max_delay (60s)",
                delay.as_secs()
            );
        }
    }

    /// Test Backoff max_retries limit
    #[test]
    fn test_backoff_max_retries_limit() {
        let config = ReconnectionConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            max_retries: 3,
            multiplier: 2.0,
        };
        let backoff = Backoff::new(config);

        assert!(!backoff.is_max_retries_exceeded()); // attempt 0 < 3
    }

    /// Test Backoff reset functionality
    #[test]
    fn test_backoff_reset() {
        let config = ReconnectionConfig::default();
        let mut backoff = Backoff::new(config);

        // Advance a few attempts
        backoff.next_delay();
        backoff.next_delay();
        assert_eq!(backoff.attempt(), 2);

        // Reset
        backoff.reset();
        assert_eq!(backoff.attempt(), 0);
    }

    /// Test ReconnectionManager basic functionality
    #[test]
    fn test_reconnection_manager_creation() {
        let manager =
            ReconnectionManager::with_default_config("project-123".to_string());

        assert_eq!(manager.project_id(), "project-123");
        assert_eq!(manager.retry_count(), 0);
        assert_eq!(manager.state(), &ReconnectionState::Idle);
    }

    /// Test ReconnectionManager reset
    #[test]
    fn test_reconnection_manager_reset() {
        let mut manager =
            ReconnectionManager::with_default_config("project-123".to_string());

        // Manually set some state
        manager.state = ReconnectionState::Reconnecting;

        // Reset
        manager.reset();

        assert_eq!(manager.state(), &ReconnectionState::Idle);
        assert_eq!(manager.retry_count(), 0);
    }

    /// Test successful reconnection on first attempt
    #[tokio::test]
    async fn test_successful_reconnection_first_attempt() {
        let mut manager =
            ReconnectionManager::with_default_config("project-123".to_string());

        let result = manager.attempt_reconnection(|_attempt| async { true }).await;

        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(manager.state(), &ReconnectionState::Reconnected);
    }

    /// Test reconnection succeeds after some failures
    #[tokio::test]
    async fn test_reconnection_succeeds_after_failures() {
        let mut manager =
            ReconnectionManager::with_default_config("project-123".to_string());

        let attempt_count = std::sync::atomic::AtomicU32::new(0);
        let result = manager
            .attempt_reconnection(|attempt| {
                let count = &attempt_count;
                async move {
                    count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // Succeed on 3rd attempt
                    attempt >= 2
                }
            })
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
        assert_eq!(manager.state(), &ReconnectionState::Reconnected);
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    /// Test reconnection fails after max retries
    #[tokio::test]
    async fn test_reconnection_fails_after_max_retries() {
        let config = ReconnectionConfig {
            initial_delay: Duration::from_millis(1), // Fast for testing
            max_delay: Duration::from_millis(10),
            max_retries: 3,
            multiplier: 2.0,
        };
        let mut manager = ReconnectionManager::new("project-123".to_string(), config);

        let result = manager
            .attempt_reconnection(|_attempt| async { false })
            .await;

        assert!(result.is_err());
        assert!(
            matches!(
                result,
                Err(ReconnectionError::MaxRetriesExceeded {
                    project_id,
                    max_retries: 3
                }) if project_id == "project-123"
            ),
            "Expected MaxRetriesExceeded error"
        );
        assert_eq!(manager.state(), &ReconnectionState::Failed);
    }

    /// Test custom backoff with different multiplier
    #[test]
    fn test_custom_multiplier() {
        let config = ReconnectionConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(100),
            max_retries: 10,
            multiplier: 3.0, // Triple each time
        };
        let mut backoff = Backoff::new(config);

        // First attempt: 1s * 3^0 = 1s
        assert_eq!(backoff.current_delay(), Duration::from_secs(1));

        // Second attempt: 1s * 3^1 = 3s
        assert_eq!(backoff.next_delay(), Duration::from_secs(3));

        // Third attempt: 1s * 3^2 = 9s
        assert_eq!(backoff.next_delay(), Duration::from_secs(9));
    }

    /// Test that initial_delay is respected
    #[test]
    fn test_custom_initial_delay() {
        let config = ReconnectionConfig {
            initial_delay: Duration::from_secs(5),
            max_delay: Duration::from_secs(60),
            max_retries: 10,
            multiplier: 2.0,
        };
        let mut backoff = Backoff::new(config);

        assert_eq!(backoff.current_delay(), Duration::from_secs(5));
        assert_eq!(backoff.next_delay(), Duration::from_secs(10)); // 5 * 2^1
    }
}
