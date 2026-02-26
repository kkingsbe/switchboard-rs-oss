//! Clock trait and SystemClock implementation for injectable time operations
//!
//! This module provides:
//! - `Clock` trait for abstracting time operations
//! - `SystemClock` implementation using std::time::Instant
//!
//! The Clock trait enables dependency injection for time-based operations,
//! allowing deterministic testing with mock clocks.

use std::time::Instant;

/// Trait for clock operations
///
/// This trait abstracts time operations to enable dependency injection
/// and testing with controllable clocks.
pub trait Clock: Send + Sync {
    /// Returns the current instant
    fn now(&self) -> Instant;
}

/// System clock implementation using std::time::Instant
///
/// This is the default clock implementation used in production.
pub struct SystemClock;

/// Implementation of the [`Clock`] trait using the system's real-time clock.
///
/// This implementation provides access to the actual system time using
/// `std::time::Instant::now()`. It is designed for use in production code
/// where real wall-clock time is needed for scheduling and timing operations.
///
/// The `SystemClock` is a zero-sized type that implements the `Clock` trait,
/// allowing it to be used as a dependency in code that requires clock operations
/// without the overhead of storing any state.
///
/// # Behavior
///
/// - The `now()` method returns the current monotonic time from the system
/// - The returned `Instant` represents the time elapsed since an unspecified
///   epoch (typically system boot time)
/// - The clock is monotonic, meaning it always moves forward and cannot go
///   backwards, even if the system clock is adjusted
///
/// # Thread Safety
///
/// This implementation is `Send` and `Sync`, allowing safe concurrent access
/// across threads, as required by the `Clock` trait bounds.
///
/// # Examples
///
/// ```rust
/// use switchboard::scheduler::{Clock, SystemClock};
///
/// let clock = SystemClock;
///
/// // Get the current time
/// let now = clock.now();
///
/// // Measure elapsed time
/// let start = clock.now();
/// // ... perform some work ...
/// let elapsed = clock.now().duration_since(start);
/// ```
impl Clock for SystemClock {
    /// Returns the current system time using Instant::now()
    fn now(&self) -> Instant {
        Instant::now()
    }
}
