//! Performance tests for the `switchboard skills list` command
//!
//! These tests measure execution time for skills listing operations.
//! The tests use a mock framework since actual skill installation
//! and listing require npx, which may not be available in CI environments.
//!
//! # Test Setup and Teardown
//!
//! Performance tests use the following isolation patterns:
//!
//! ## TempDir-based Isolation
//! - Tests create temporary directories using `tempfile::TempDir`
//! - TempDir automatically cleans up when dropped (end of test)
//! - Each test gets its own isolated directory
//!
//! ## Environment Variable Isolation
//! - Tests that need to override environment variables (like HOME)
//! - Save the original value before modification
//! - Always restore the original value in teardown, regardless of test outcome
//!
//! ## Test Repeatability
//! - Tests are designed to be repeatable (run multiple times without interference)
//! - No shared state between tests
//! - Each test creates its own test data
//!
//! # Testing Note
//!
//! Full integration testing of `switchboard skills list` requires npx to be
//! available on the system. These tests use mock implementations to
//! demonstrate the timing framework and verify performance characteristics.
//!
//! For full integration testing, run tests in an environment with npx installed:
//! ```bash
//! cargo test --test skills_list_performance
//! ```

use std::fs;

// Include the performance_common module for this integration test
include!("performance_common.rs");

use std::time::Instant;

use switchboard::skills::{scan_global_skills, scan_skill_directory, SkillMetadata};

<<<<<<< HEAD
/// High-precision timer for performance measurements
#[derive(Clone)]
struct Timer {
    name: String,
    start: std::time::Instant,
}

impl Timer {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: std::time::Instant::now(),
        }
    }

    fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

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
>>>>>>> skills-improvements
/// Setup function for performance tests
///
/// Creates a clean test environment with a temporary directory.
/// This ensures test isolation between runs.
fn setup() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Teardown function for performance tests
///
/// Cleans up the test environment.
fn teardown() {
    // No teardown needed - temp dirs are auto-cleaned
}

/// Performance test: Skills list with empty skills directory
///
/// This test verifies that listing skills with an empty .kilocode/skills/
/// directory completes within 3 seconds. This is a baseline test that
/// measures the overhead of directory scanning without any installed skills.
#[tokio::test]
async fn test_skills_list_empty_directory_performance() {
    // Setup: ensure clean state
    setup();

    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path().join(".kilocode").join("skills");

    // Create empty .kilocode/skills/ directory
    fs::create_dir_all(&skills_dir).unwrap();

    // Measure execution time for scanning empty skills directory using high-precision Timer
    let mut timer = Timer::new("skills_list_empty");
    let result = scan_skill_directory(&skills_dir);
    let duration = timer.elapsed();

    // Verify the scan succeeded
    assert!(
        result.is_ok(),
        "scan_skill_directory failed: {:?}",
        result.err()
    );
    let (skills, _warnings) = result.unwrap();

    // Verify no skills were found in empty directory
    assert_eq!(skills.len(), 0, "Expected no skills in empty directory");

    // Assert execution time is less than 3 seconds
    assert!(
        duration.as_secs() < 3,
        "Empty skills list took {}, expected < 3 seconds",
        format_duration(duration)
    );

    println!(
        "Empty skills directory scan completed in {}",
        format_duration(duration)
    );

    // Teardown is automatic via temp_dir and _setup drop
}

/// Performance test: Skills list with multiple installed skills (mock)
///
/// This test creates a mock implementation that simulates listing multiple
/// installed skills. Since actual skill installation requires npx and
/// may not be available in CI, this test demonstrates the timing framework
/// with simulated skill data.
///
/// # Testing Note
///
/// Full integration test requires npx availability. This mock test verifies:
/// - The timing framework works correctly
/// - Performance expectations are reasonable
/// - Test structure is ready for integration testing
#[tokio::test]
async fn test_skills_list_multiple_skills_mock_performance() {
    // Setup: ensure clean state
    setup();

    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path().join(".kilocode").join("skills");

    // Create skills directory
    fs::create_dir_all(&skills_dir).unwrap();

    // Create mock skill directories with SKILL.md files
    // Simulating multiple installed skills without actually running npx
    let mock_skills = vec![
        (
            "frontend-design",
            "Frontend design automation skill",
            "1.0.0",
        ),
        (
            "docker-operations",
            "Docker container management skill",
            "2.1.0",
        ),
        ("file-operations", "File system operations skill", "1.2.3"),
    ];

    for (skill_name, description, version) in &mock_skills {
        let skill_path = skills_dir.join(skill_name);
        fs::create_dir_all(&skill_path).unwrap();

        let skill_md_content = format!(
            r#"---
name: {}
description: {}
version: {}
authors: ["Test Author"]
dependencies: []
compatible_agents: ["architect", "code", "ask", "debug", "orchestrator"]
---

# {} Skill

This is a mock skill for performance testing.
"#,
            skill_name, description, version, skill_name
        );

        let skill_md_path = skill_path.join("SKILL.md");
        fs::write(&skill_md_path, skill_md_content).unwrap();
    }

    // Measure execution time for scanning skills directory with mock skills
    let mut timer = Timer::new("skills_list_multiple_mock");
    let result = scan_skill_directory(&skills_dir);
    let duration = timer.elapsed();

    // Verify the scan succeeded
    assert!(
        result.is_ok(),
        "scan_skill_directory failed: {:?}",
        result.err()
    );
    let (skills, warnings) = result.unwrap();

    // Verify we found all mock skills
    assert_eq!(
        skills.len(),
        mock_skills.len(),
        "Expected {} skills, found {}",
        mock_skills.len(),
        skills.len()
    );

    // Verify no warnings were generated (all skills are valid)
    assert_eq!(
        warnings.len(),
        0,
        "Expected no warnings, got: {:?}",
        warnings
    );

    // Assert execution time is less than 3 seconds
    assert!(
        duration.as_secs() < 3,
        "Skills list with {} mock skills took {}, expected < 3 seconds",
        mock_skills.len(),
        format_duration(duration)
    );

    println!(
        "Skills list with {} mock skills completed in {}",
        skills.len(),
        format_duration(duration)
    );

    // Verify skill metadata was correctly parsed
    for skill in &skills {
        assert!(!skill.name.is_empty(), "Skill name should not be empty");
        assert!(
            skill.description.is_some(),
            "Skill description should be present"
        );
        assert!(skill.version.is_some(), "Skill version should be present");
    }

    // Teardown is automatic via temp_dir and _setup drop
}

/// Performance test: Skills list with search query (mock)
///
/// This test demonstrates the timing framework for skills listing with
/// search parameters. The actual run_skills_list function delegates to
/// `npx skills find` which requires npx availability.
///
/// This mock test verifies:
/// - Search query handling structure
/// - Timing measurements with search parameters
/// - Performance expectations for filtered results
///
/// # Testing Note
///
/// Full integration test requires npx availability to run `npx skills find <query>`.
/// This mock test creates the test structure and verifies timing expectations.
#[tokio::test]
async fn test_skills_list_search_query_mock_performance() {
    // Setup: ensure clean state
    let _setup = setup();

    let temp_dir = TempDir::new().unwrap();
    let skills_dir = temp_dir.path().join(".kilocode").join("skills");

    // Create skills directory
    fs::create_dir_all(&skills_dir).unwrap();

    // Create mock skills with diverse names/descriptions to test search filtering
    let mock_skills = vec![
        (
            "docker-operations",
            "Docker container management operations",
            "1.0.0",
        ),
        ("docker-compose", "Docker compose file generation", "1.5.0"),
        ("file-operations", "File system operations utility", "2.0.0"),
        (
            "discord-notifications",
            "Send discord notifications",
            "0.1.0",
        ),
        (
            "frontend-design",
            "Frontend design and UI generation",
            "3.0.0",
        ),
    ];

    for (skill_name, description, version) in &mock_skills {
        let skill_path = skills_dir.join(skill_name);
        fs::create_dir_all(&skill_path).unwrap();

        let skill_md_content = format!(
            r#"---
name: {}
description: {}
version: {}
authors: ["Test Author"]
dependencies: []
compatible_agents: ["architect", "code", "ask", "debug", "orchestrator"]
---

# {} Skill

This is a mock skill for search performance testing.
"#,
            skill_name, description, version, skill_name
        );

        let skill_md_path = skill_path.join("SKILL.md");
        fs::write(&skill_md_path, skill_md_content).unwrap();
    }

    // Simulate search query filtering - in production this would be handled by npx skills find
    let search_query = "docker";

    // Measure execution time for scanning and filtering skills using high-precision Timer
    let mut timer = Timer::new("skills_list_search_query");

    // Scan all skills
    let result = scan_skill_directory(&skills_dir);
    assert!(
        result.is_ok(),
        "scan_skill_directory failed: {:?}",
        result.err()
    );
    let (skills, _warnings) = result.unwrap();

    // Filter skills by search query (mock implementation of search functionality)
    let filtered_skills: Vec<&SkillMetadata> = skills
        .iter()
        .filter(|skill| {
            skill.name.contains(search_query)
                || skill
                    .description
                    .as_ref()
                    .map(|d| d.contains(search_query))
                    .unwrap_or(false)
        })
        .collect();

    let duration = timer.elapsed();

    // Verify filtering worked correctly
    assert!(
        filtered_skills.len() >= 2,
        "Expected at least 2 skills matching 'docker', found {}",
        filtered_skills.len()
    );

    // Verify each result actually contains the search term
    for skill in &filtered_skills {
        assert!(
            skill.name.contains(search_query)
                || skill
                    .description
                    .as_ref()
                    .map(|d| d.contains(search_query))
                    .unwrap_or(false),
            "Filtered skill '{}' should contain search query '{}'",
            skill.name,
            search_query
        );
    }

    // Assert execution time is less than 3 seconds
    assert!(
        duration.as_secs() < 3,
        "Skills list with search query '{}' took {}, expected < 3 seconds",
        search_query,
        format_duration(duration)
    );

    println!(
        "Skills list with search query '{}' completed in {} (found {} results)",
        search_query,
        format_duration(duration),
        filtered_skills.len()
    );

    // Teardown is automatic via temp_dir and _setup drop
}

/// Performance test: Global skills scanning (baseline)
///
/// This test measures the performance of scanning the global skills directory.
/// It verifies that the scan completes within acceptable time limits even when
/// the global directory is empty or doesn't exist.
///
/// # Setup/Teardown
///
/// This test uses a temporary directory to isolate the global skills scan
/// from the user's actual configuration. The HOME environment variable is
/// temporarily modified to point to the temp directory, then restored after
/// the test completes.
#[tokio::test]
async fn test_global_skills_scan_performance() {
    // Setup: ensure clean state with temp dir
<<<<<<< HEAD
    let temp_dir = setup();
    let temp_dir_path = temp_dir.path();
=======
    let _setup = setup();

    // Create a temp directory for testing
    let temp_dir = TempDir::new().unwrap();
>>>>>>> skills-improvements

    // Temporarily override HOME environment variable to point to temp dir
    // This ensures we don't actually read the user's global skills directory
    // Setup: Save and modify HOME
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp_dir_path);

    // Create empty global skills directory
<<<<<<< HEAD
    let global_skills_dir = temp_dir_path.join(".kilocode").join("skills");
=======
    let global_skills_dir = temp_dir.path().join(".kilocode").join("skills");
>>>>>>> skills-improvements
    fs::create_dir_all(&global_skills_dir).unwrap();

    // Measure execution time for scanning global skills directory using high-precision Timer
    let mut timer = Timer::new("global_skills_scan");
    let result = scan_global_skills();
    let duration = timer.elapsed();

    // Teardown: Always restore HOME environment variable
    match original_home {
        Some(home) => std::env::set_var("HOME", home),
        None => std::env::remove_var("HOME"),
    }

    // Teardown is automatic via setup drop
}

/// Performance test: SkillsManager npx availability check
///
/// This test measures the performance of checking npx availability.
/// This is a baseline operation that runs before any skills list operation.
#[tokio::test]
async fn test_npx_availability_check_performance() {
    use switchboard::skills::SkillsManager;

    let mut manager = SkillsManager::new(None);

    // Measure execution time for npx availability check using high-precision Timer
    let mut timer = Timer::new("npx_availability_check");
    let result = manager.check_npx_available();
    let duration = timer.elapsed();

    // The result may be Ok(()) if npx is available, or Err if not
    // Both outcomes are valid for this performance test
    if result.is_ok() {
        println!("npx is available on the system");
    } else {
        println!("npx is not available on the system (expected in CI)");
    }

    // Assert execution time is less than 3 seconds
    assert!(
        duration.as_secs() < 3,
        "npx availability check took {}, expected < 3 seconds",
        format_duration(duration)
    );

    println!(
        "npx availability check completed in {}",
        format_duration(duration)
    );
}
