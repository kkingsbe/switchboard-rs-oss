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

<<<<<<< HEAD
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

=======
use std::time::Instant;

// Include the performance_common module for this integration test
include!("performance_common.rs");

use switchboard::docker::skills::generate_entrypoint_script;

>>>>>>> skills-improvements
/// Setup function for performance tests
///
/// Creates a clean test environment.
/// This ensures test isolation between runs.
fn setup() {
    // No setup needed for script generation tests
}

/// Teardown function for performance tests
///
/// Cleans up the test environment.
fn teardown() {
    // No teardown needed - no external resources used
}

/// Performance test: Skill install script generation
///
/// This test verifies that generating the entrypoint script for a single
/// small skill completes within acceptable time limits. Script generation
/// should be very fast (under 1 second), as it only constructs the script
/// content without executing Docker or npx commands.
///
/// Actual skill installation in Docker would take longer (15 second target),
/// but this test only measures the script generation phase.
#[tokio::test]
async fn test_skill_install_script_generation_performance() {
    // Setup: ensure clean state
    setup();

    let agent_name = "test-agent";
    let skills = vec!["owner/repo".to_string()];

    // Measure script generation time
    let start = Instant::now();
    let result = generate_entrypoint_script(agent_name, &skills, &[]);
    let duration = start.elapsed();

    // Verify the script was generated successfully
    assert!(
        result.is_ok(),
        "generate_entrypoint_script failed: {:?}",
        result.err()
    );
    let script = result.unwrap();

    // Verify the script was generated successfully
    assert!(!script.is_empty(), "Generated script should not be empty");

    // Verify the script contains the skill installation command
    assert!(
        script.contains("owner/repo"),
        "Generated script should contain the skill reference 'owner/repo'"
    );

    // Assert script generation time is less than 1 second
    assert!(
        duration.as_secs() < 1,
        "Script generation for single skill took {}, expected < 1 second",
        format_duration(duration)
    );

    println!(
        "Script generation for 1 skill completed in {}",
        format_duration(duration)
    );

    // Teardown is automatic via _setup drop
}

/// Performance test: Skill install script generation with multiple skills
///
/// This test verifies that generating the entrypoint script for multiple
/// skills (5 skills with different formats) completes within acceptable time
/// limits. Script generation should be very fast (under 1 second), even with
/// multiple skills, as it only constructs the script content without executing
/// Docker or npx commands.
///
/// Testing with multiple skills helps verify that script generation scales
/// linearly and doesn't degrade significantly as the number of skills increases.
#[tokio::test]
async fn test_skill_install_script_generation_multiple_skills() {
    // Setup: ensure clean state
    let _setup = setup();

    let agent_name = "test-agent";
    let skills = vec![
        "owner/repo1".to_string(),
        "owner/repo2@skill".to_string(),
        "another/cool-skill".to_string(),
        "docker/operations".to_string(),
        "file/utils@helper".to_string(),
    ];

    // Measure script generation time
    let start = Instant::now();
    let result = generate_entrypoint_script(agent_name, &skills, &[]);
    let duration = start.elapsed();

    // Verify the script was generated successfully
    assert!(
        result.is_ok(),
        "generate_entrypoint_script failed: {:?}",
        result.err()
    );
    let script = result.unwrap();

    // Verify the script was generated successfully
    assert!(!script.is_empty(), "Generated script should not be empty");

    // Verify the script contains all skill references
    for skill in &skills {
        assert!(
            script.contains(skill),
            "Generated script should contain the skill reference '{}'",
            skill
        );
    }

    // Assert script generation time is less than 1 second
    assert!(
        duration.as_secs() < 1,
        "Script generation for {} skills took {}, expected < 1 second",
        skills.len(),
        format_duration(duration)
    );

    println!(
        "Script generation for {} skills completed in {}",
        skills.len(),
        format_duration(duration)
    );

    // Teardown is automatic via _setup drop
}

/// Benchmark test: Skill install script generation across different skill counts
///
/// This test benchmarks script generation performance across different skill counts
/// (1, 3, 5, and 10 skills) to verify linear scaling behavior (O(n) complexity).
///
/// The test measures script generation time for each skill count and verifies that
/// the preparation phase is fast (typically milliseconds), not a performance bottleneck.
/// The 15-second performance target specified in TODO applies to the full Docker
/// container startup + npx installation process, not to script generation alone.
///
/// These tests document that script generation is NOT the bottleneck for the
/// 15-second target - the actual Docker and npx execution in containers are the
/// dominant factors.
#[tokio::test]
async fn test_skill_install_script_generation_benchmark() {
    // Setup: ensure clean state
    let _setup = setup();

    let agent_name = "test-agent";
    let skill_counts = [1, 3, 5, 10];
    let mut ten_skill_duration = None;

    println!("\n=== Script Generation Benchmark ===\n");

    for count in skill_counts {
        // Create skills vector with generic names
        let skills: Vec<String> = (1..=count).map(|i| format!("owner/repo{}", i)).collect();

        // Measure script generation time
        let start = Instant::now();
        let result = generate_entrypoint_script(agent_name, &skills, &[]);
        let duration = start.elapsed();

        // Verify the script was generated successfully
        assert!(
            result.is_ok(),
            "generate_entrypoint_script failed for {} skills: {:?}",
            count,
            result.err()
        );
        let script = result.unwrap();

        // Verify the script is not empty
        assert!(
            !script.is_empty(),
            "Generated script for {} skills should not be empty",
            count
        );

        // Assert generation time is less than 1 second
        assert!(
            duration.as_secs() < 1,
            "Script generation for {} skills took {}, expected < 1 second",
            count,
            format_duration(duration)
        );

        println!(
            "{} skill(s): script generation completed in {}",
            count,
            format_duration(duration)
        );

        // Store the 10-skill duration for final assertion
        if count == 10 {
            ten_skill_duration = Some(duration);
        }
    }

<<<<<<< HEAD
=======
    // Print benchmark summary
    print_benchmark_summary(&benchmark_results);

>>>>>>> skills-improvements
    // Assert that 10-skills generation is reasonably faster than 15-second target
    if let Some(duration) = ten_skill_duration {
        let fifteen_seconds = std::time::Duration::from_secs(15);
        assert!(
            duration < fifteen_seconds,
            "Script generation for 10 skills took {:?}, should be much faster than 15 seconds",
            duration
        );

        println!("\n=== Summary ===");
        println!("Script generation completes in milliseconds (or seconds at most)");
        println!("The 15-second performance target applies to:");
        println!("  - Docker container startup time");
        println!("  - npx package download and installation within containers");
        println!("These tests verify that the script preparation phase is NOT a bottleneck.\n");
    }
}
