//! Performance tests for container skill installation operations
//!
//! These tests measure execution time for entrypoint script generation
//! during container skill installation. Since actual skill installation
//! requires Docker, npx, and network access (which may not be available
//! in CI environments), these tests focus on measuring the script generation
//! performance, which is the preparation phase before actual installation.
//!
//! # Test Setup and Teardown
//!
//! Performance tests use the following patterns:
//!
//! ## No External Dependencies
//! - Script generation tests do not require TempDir or external resources
//! - They only measure CPU-bound string processing operations
//! - No file I/O or network operations are involved
//!
//! ## Test Repeatability
//! - Tests are designed to be repeatable (run multiple times without interference)
//! - No shared state between tests
//! - Deterministic input produces deterministic output
//!
//! ## Benchmark Tests
//! - The benchmark test measures scaling across different skill counts
//! - Warmup iterations are included in the measurement framework
//! - Results demonstrate O(n) linear scaling behavior
//!
//! # Testing Note
//!
//! Full integration testing of container skill installation requires Docker
//! and npx to be available on the system. These tests measure entrypoint
//! script generation performance, which is the preparation phase before
//! actual installation. For complete end-to-end testing of skill installation,
//! run tests in an environment with Docker and npx installed:
//! ```bash
//! cargo test --test skills_install_performance
//! ```

// Direct imports for switchboard functionality
use std::time::Instant;

use switchboard::docker::skills::generate_entrypoint_script;

/// Formats a duration in a human-readable format
fn format_duration(duration: std::time::Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    } else {
        format!("{:.4}s", duration.as_secs_f64())
    }
}
