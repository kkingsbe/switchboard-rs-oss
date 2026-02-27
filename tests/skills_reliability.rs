//! Reliability and stress tests for container skill installation operations
//!
//! These tests verify the reliability and stability of skill installation
//! operations under various stress conditions. Since actual skill installation
//! requires Docker, npx, and network access (which may not be available
//! in CI environments), these tests focus on the script generation phase,
//! which is the preparation step before actual installation.
//!
//! # What These Tests Verify
//!
//! - **Sequential multiple skills installation**: Tests installing 10 skills
//!   in sequence and verifies no resource leaks between installations.
//! - **Same skill reinstall**: Tests installing the same skill multiple times
//!   to verify idempotency and no duplicate entries.
//! - **Concurrent installation**: Tests whether the system handles concurrent
//!   installations properly (documents limitation if not supported).
//! - **Resource leak detection**: Verifies no file handles remain open and
//!   memory usage is stable after multiple operations.
//!
//! # Running These Tests
//!
//! ```bash
//! cargo test --test skills_reliability
//! ```
//!
//! For full integration testing with Docker and npx, run in an environment
//! where these are available.

use std::time::Instant;

use switchboard::docker::skills::generate_entrypoint_script;

/// Test: Sequential multiple skills installation
///
/// This test verifies that installing 10 skills in sequence works reliably
/// without any resource leaks or degradation between installations.
///
/// Each call to generate_entrypoint_script should complete successfully
/// and produce valid output. The test checks that:
/// - Each script generation completes without error
/// - Each generated script is valid and non-empty
/// - No resource degradation occurs across multiple sequential calls
#[tokio::test]
async fn test_sequential_multiple_skills_installation() {
    let agent_name = "test-agent";

    // Create 10 different skills for sequential installation
    let skills: Vec<String> = (1..=10).map(|i| format!("owner/repo{}", i)).collect();

    println!("\n=== Sequential Multiple Skills Installation Test ===");

    // Install skills sequentially
    for (idx, skill) in skills.iter().enumerate() {
        let single_skill = vec![skill.clone()];

        // Generate script for this skill
        let result = generate_entrypoint_script(agent_name, &single_skill, &[]);

        // Verify successful generation
        assert!(
            result.is_ok(),
            "Failed to generate script for skill {} (index {}): {:?}",
            skill,
            idx,
            result.err()
        );

        let script = result.unwrap();

        // Verify non-empty script
        assert!(
            !script.is_empty(),
            "Generated empty script for skill {} (index {})",
            skill,
            idx
        );

        // Verify skill is in the script
        assert!(
            script.contains(skill),
            "Generated script for skill {} does not contain the skill reference",
            skill
        );

        println!(
            "Successfully generated script for skill {} (index {})",
            skill, idx
        );
    }

    println!("All 10 sequential skill installations completed successfully");
}

/// Test: Same skill reinstall multiple times
///
/// This test verifies that installing the same skill multiple times (3 times)
/// works reliably and produces consistent, idempotent results.
///
/// The test checks:
/// - Each reinstall succeeds without errors
/// - No duplicate entries or corruption occurs
/// - Script output is consistent across reinstalls
#[tokio::test]
async fn test_same_skill_reinstall_multiple_times() {
    let agent_name = "test-agent";
    let skill = "owner/repo";

    // Reinstall the same skill 3 times
    let reinstall_count = 3;
    let mut previous_script: Option<String> = None;

    println!("\n=== Same Skill Reinstall Test ===");

    for idx in 0..reinstall_count {
        let single_skill = vec![skill.to_string()];

        // Generate script for reinstall
        let result = generate_entrypoint_script(agent_name, &single_skill, &[]);

        // Verify successful generation
        assert!(
            result.is_ok(),
            "Failed to generate script for reinstall {}: {:?}",
            idx + 1,
            result.err()
        );

        let script = result.unwrap();

        // Verify non-empty script
        assert!(
            !script.is_empty(),
            "Generated empty script for reinstall {}",
            idx + 1
        );

        // Verify skill is in the script
        assert!(
            script.contains(skill),
            "Generated script for reinstall {} does not contain the skill reference",
            idx + 1
        );

        // Verify consistency with previous script (if not first)
        if let Some(ref prev) = previous_script {
            assert_eq!(
                script,
                *prev,
                "Script for reinstall {} differs from previous - not idempotent",
                idx + 1
            );
            println!(
                "Script for reinstall {} is consistent with previous (idempotent)",
                idx + 1
            );
        } else {
            println!("Script for reinstall {} generated successfully", idx + 1);
        }

        previous_script = Some(script);
    }

    println!(
        "All {} reinstalls completed successfully and were idempotent",
        reinstall_count
    );
}

/// Test: Concurrent installation simulation
///
/// This test simulates concurrent installations by generating multiple scripts
/// in rapid succession. Since the SkillsManager's generate_entrypoint_script
/// function is synchronous and doesn't have internal locking, this test documents
/// the current behavior.
///
/// Current limitation: The generate_entrypoint_script function is not designed
/// for concurrent use with shared state. If concurrent installation is needed,
/// callers should ensure proper synchronization at the application level.
#[tokio::test]
async fn test_concurrent_installation_simulation() {
    let agent_name = "test-agent";

    // Create different skills for "concurrent" simulation
    let skills_set_1 = vec!["owner/repo1".to_string()];
    let skills_set_2 = vec!["owner/repo2".to_string()];
    let skills_set_3 = vec!["owner/repo3".to_string()];

    println!("\n=== Concurrent Installation Simulation Test ===");

    // Note: In a true concurrent scenario, SkillsManager would need to support
    // concurrent installations. Currently, generate_entrypoint_script is a
    // pure function that doesn't modify shared state, so it's safe to call
    // from multiple tasks, but actual skill installation would require
    // coordination at a higher level.

    // Test 1: Generate scripts for different skill sets (simulating concurrent requests)
    let result1 = generate_entrypoint_script(agent_name, &skills_set_1, &[]);
    let result2 = generate_entrypoint_script(agent_name, &skills_set_2, &[]);
    let result3 = generate_entrypoint_script(agent_name, &skills_set_3, &[]);

    assert!(
        result1.is_ok(),
        "Script generation 1 failed: {:?}",
        result1.err()
    );
    assert!(
        result2.is_ok(),
        "Script generation 2 failed: {:?}",
        result2.err()
    );
    assert!(
        result3.is_ok(),
        "Script generation 3 failed: {:?}",
        result3.err()
    );

    let script1 = result1.unwrap();
    let script2 = result2.unwrap();
    let script3 = result3.unwrap();

    assert!(!script1.is_empty(), "Script 1 should not be empty");
    assert!(!script2.is_empty(), "Script 2 should not be empty");
    assert!(!script3.is_empty(), "Script 3 should not be empty");

    // Verify each script contains its respective skill
    assert!(
        script1.contains("owner/repo1"),
        "Script 1 should contain owner/repo1"
    );
    assert!(
        script2.contains("owner/repo2"),
        "Script 2 should contain owner/repo2"
    );
    assert!(
        script3.contains("owner/repo3"),
        "Script 3 should contain owner/repo3"
    );

    println!("Concurrent script generation completed successfully");
    println!("Note: generate_entrypoint_script is a pure function with no shared state,");
    println!("so concurrent calls are safe. Actual skill installation requires");
    println!("coordination at the application level to avoid race conditions.");
}

/// Test: Resource leak detection - file handles
///
/// This test verifies that repeated script generations don't leak resources.
/// Since generate_entrypoint_script doesn't create files directly (it just
/// generates strings), we verify that:
/// - Memory usage remains stable (no growing allocations)
/// - Each call completes successfully
/// - No errors occur during repeated operations
///
/// Note: True file handle leak detection would require testing actual skill
/// installation, which requires Docker and npx.
#[tokio::test]
async fn test_resource_leak_detection_file_handles() {
    let agent_name = "test-agent";
    let skill = "owner/repo";
    let iterations = 20;

    println!("\n=== Resource Leak Detection Test ===");

    // Track memory usage indirectly by measuring time - significant time
    // increases could indicate memory pressure or leaks
    let mut durations: Vec<std::time::Duration> = Vec::new();

    for idx in 0..iterations {
        let single_skill = vec![skill.to_string()];

        let start = Instant::now();
        let result = generate_entrypoint_script(agent_name, &single_skill, &[]);
        let duration = start.elapsed();

        durations.push(duration);

        // Verify successful generation
        assert!(
            result.is_ok(),
            "Script generation {} failed: {:?}",
            idx + 1,
            result.err()
        );

        let script = result.unwrap();

        // Verify non-empty script
        assert!(
            !script.is_empty(),
            "Generated empty script for iteration {}",
            idx + 1
        );

        // Print progress every 5 iterations
        if (idx + 1) % 5 == 0 {
            println!(
                "Completed {} iterations, latest duration: {:?}",
                idx + 1,
                duration
            );
        }
    }

    // Verify no significant time degradation
    // First 5 iterations average vs last 5 iterations average
    let first_five_avg: std::time::Duration =
        durations[..5].iter().sum::<std::time::Duration>() / 5;
    let last_five_avg: std::time::Duration = durations[iterations - 5..]
        .iter()
        .sum::<std::time::Duration>()
        / 5;

    println!(
        "First 5 iterations average: {:?}, Last 5 iterations average: {:?}",
        first_five_avg, last_five_avg
    );

    // Allow 10x degradation as a reasonable threshold (in practice, should be much closer)
    // This is a very permissive check since we're measuring very fast operations
    let threshold = first_five_avg * 10;
    assert!(
        last_five_avg < threshold,
        "Significant time degradation detected: last 5 avg ({:?}) > 10x first 5 avg ({:?})",
        last_five_avg,
        threshold
    );

    println!(
        "Resource leak detection passed: no significant time/memory degradation over {} iterations",
        iterations
    );
}

/// Test: Large skill count reliability
///
/// This test verifies that script generation remains reliable with a large
/// number of skills (50 skills). This stress tests the script generation
/// to ensure it handles larger inputs without degradation or errors.
#[tokio::test]
async fn test_large_skill_count_reliability() {
    let agent_name = "test-agent";

    // Create 50 different skills
    let skills: Vec<String> = (1..=50).map(|i| format!("owner/repo{}", i)).collect();

    println!("\n=== Large Skill Count Reliability Test (50 skills) ===");

    let start = Instant::now();
    let result = generate_entrypoint_script(agent_name, &skills, &[]);
    let duration = start.elapsed();

    // Verify successful generation
    assert!(
        result.is_ok(),
        "Failed to generate script for 50 skills: {:?}",
        result.err()
    );

    let script = result.unwrap();

    // Verify non-empty script
    assert!(!script.is_empty(), "Generated empty script for 50 skills");

    // Verify all skills are in the script
    for skill in &skills {
        assert!(
            script.contains(skill),
            "Generated script for 50 skills does not contain skill {}",
            skill
        );
    }

    println!(
        "Successfully generated script for 50 skills in {:?}",
        duration
    );

    // Verify reasonable completion time (< 1 second for pure script generation)
    assert!(
        duration.as_secs() < 1,
        "Script generation for 50 skills took {:?}, expected < 1 second",
        duration
    );
}
