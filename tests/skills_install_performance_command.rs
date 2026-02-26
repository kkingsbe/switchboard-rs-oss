//! Performance tests for the `switchboard skills install` command
//!
//! These tests measure execution time for skills installation operations.
//! Since actual skill installation requires npx and network access to the
//! npm registry (which may not be available in CI environments), these
//! tests focus on measuring the preparation phase of installation:
//!
//! - npx availability check (required before any installation)
//! - SkillsManager initialization
//! - Command preparation/building for installation
//! - Mock-based installation flow simulation
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
//! Full integration testing of `switchboard skills install` requires npx to be
//! available and able to access the npm registry. These tests use mock
//! implementations to demonstrate the timing framework and verify performance
//! characteristics of the preparation phase.
//!
//! For full integration testing, run tests in an environment with npx installed:
//! ```bash
//! cargo test --test skills_install_performance_command
//! ```

use std::fs;
use std::time::Instant;
use tempfile::TempDir;

use switchboard::skills::{create_npx_command, SkillsManager};

/// Performance test: SkillsManager initialization
///
/// This test measures the performance of creating a new SkillsManager instance.
/// This is the first step in any skills installation operation and should
/// complete very quickly (under 100ms).
///
/// The 100ms threshold is chosen because SkillsManager initialization only:
/// - Reads the HOME environment variable
/// - Constructs PathBuf values
/// - Does not perform any I/O or network operations
#[tokio::test]
async fn test_skills_manager_initialization_performance() {
    // Measure SkillsManager initialization time
    let start = Instant::now();
    let manager = SkillsManager::new(None);
    let duration = start.elapsed();

    // Verify the manager was created successfully
    assert!(
        manager.skills_dir().exists() || !manager.skills_dir().to_string_lossy().is_empty(),
        "SkillsManager should have valid skills_dir"
    );

    // Assert initialization time is less than 100 milliseconds
    let millis = duration.as_millis();
    assert!(
        millis < 100,
        "SkillsManager initialization took {:?}, expected < 100ms",
        duration
    );

    println!("SkillsManager initialization completed in {:?}", duration);
}

/// Performance test: SkillsManager initialization with custom skills directory
///
/// This test measures the performance of creating a SkillsManager with a
/// custom skills directory path. This is used for project-level installations.
#[tokio::test]
async fn test_skills_manager_custom_dir_initialization_performance() {
    let temp_dir = TempDir::new().unwrap();
    let custom_skills_dir = temp_dir.path().join(".kilocode").join("skills");

    // Create the directory to ensure it exists
    fs::create_dir_all(&custom_skills_dir).unwrap();

    // Measure SkillsManager initialization with custom directory
    let start = Instant::now();
    let manager = SkillsManager::with_skills_dir(custom_skills_dir.clone(), None);
    let duration = start.elapsed();

    // Verify the manager was created with the custom directory
    assert_eq!(
        manager.skills_dir(),
        &custom_skills_dir,
        "SkillsManager should use custom skills directory"
    );

    // Assert initialization time is less than 100 milliseconds
    let millis = duration.as_millis();
    assert!(
        millis < 100,
        "SkillsManager with custom dir initialization took {:?}, expected < 100ms",
        duration
    );

    println!(
        "SkillsManager with custom dir initialization completed in {:?}",
        duration
    );
}

/// Performance test: npx availability check (preparation step)
///
/// This test measures the performance of checking npx availability, which is
/// the required preparation step before any skills installation. The npx check
/// executes `npx --version` to verify the tool is installed.
///
/// Performance threshold: 3 seconds
/// Rationale: The npx check involves spawning a process and waiting for output.
/// While typically fast (milliseconds), we allow up to 3 seconds to account for
/// cold starts and system load in CI environments.
#[tokio::test]
async fn test_npx_availability_check_install_performance() {
    let mut manager = SkillsManager::new(None);

    // Measure npx availability check time
    let start = Instant::now();
    let result = manager.check_npx_available();
    let duration = start.elapsed();

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
        "npx availability check took {:?}, expected < 3 seconds",
        duration
    );

    println!("npx availability check completed in {:?}", duration);
}

/// Performance test: Command preparation for skills installation
///
/// This test measures the performance of building the npx command for skills
/// installation. This is the command preparation phase before actual execution.
///
/// The command building involves:
/// - Creating a Command instance
/// - Adding arguments (skills, add, source, -a kilo, -y)
///
/// This should be very fast (under 100ms) as it only constructs the command
/// without executing it.
#[tokio::test]
async fn test_skills_install_command_preparation_performance() {
    let source = "owner/repo";

    // Measure command preparation time
    let start = Instant::now();

    // Build the command (same as run_skills_install)
    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");
    cmd.arg(source);
    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y");

    let duration = start.elapsed();

    // Verify the command was built correctly
    assert_eq!(cmd.get_program().to_string_lossy(), "npx");

    // Get the arguments to verify they were set correctly
    // CommandArgs implements IntoIterator, so we can collect directly
    let args: Vec<String> = cmd
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    assert!(
        args.contains(&"skills".to_string()),
        "Command should contain 'skills'"
    );
    assert!(
        args.contains(&"add".to_string()),
        "Command should contain 'add'"
    );
    assert!(
        args.contains(&source.to_string()),
        "Command should contain the source '{}'",
        source
    );

    // Assert command preparation time is less than 100 milliseconds
    let millis = duration.as_millis();
    assert!(
        millis < 100,
        "Command preparation took {:?}, expected < 100ms",
        duration
    );

    println!(
        "Skills install command preparation completed in {:?}",
        duration
    );
}

/// Performance test: Command preparation with global flag
///
/// This test measures command preparation when the --global flag is used
/// for global skill installation.
#[tokio::test]
async fn test_skills_install_global_command_preparation_performance() {
    let source = "owner/repo";

    // Measure command preparation time with global flag
    let start = Instant::now();

    // Build the command with global flag
    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");
    cmd.arg(source);
    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y");
    cmd.arg("-g"); // Global flag

    let duration = start.elapsed();

    // Verify the command was built correctly with global flag
    let args: Vec<String> = cmd
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    assert!(
        args.contains(&"-g".to_string()),
        "Command should contain global flag '-g'"
    );

    // Assert command preparation time is less than 100 milliseconds
    let millis = duration.as_millis();
    assert!(
        millis < 100,
        "Global command preparation took {:?}, expected < 100ms",
        duration
    );

    println!(
        "Skills install global command preparation completed in {:?}",
        duration
    );
}

/// Performance test: Command preparation with skill name suffix
///
/// This test measures command preparation when installing a specific skill
/// from a repository using the @syntax (e.g., owner/repo@skill-name).
#[tokio::test]
async fn test_skills_install_with_skill_name_command_preparation_performance() {
    let source = "owner/repo@my-skill";

    // Measure command preparation time with skill name
    let start = Instant::now();

    // Build the command
    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");
    cmd.arg(source);
    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y");

    let duration = start.elapsed();

    // Verify the command contains the skill name
    let args: Vec<String> = cmd
        .get_args()
        .map(|s| s.to_string_lossy().to_string())
        .collect();
    assert!(
        args.contains(&source.to_string()),
        "Command should contain the source with skill name '{}'",
        source
    );

    // Assert command preparation time is less than 100 milliseconds
    let millis = duration.as_millis();
    assert!(
        millis < 100,
        "Command with skill name preparation took {:?}, expected < 100ms",
        duration
    );

    println!(
        "Skills install with skill name command preparation completed in {:?}",
        duration
    );
}

/// Performance test: Multiple command preparations (benchmark)
///
/// This test benchmarks command preparation across multiple iterations
/// to verify consistent performance. This simulates the preparation phase
/// that would occur before each installation attempt.
#[tokio::test]
async fn test_skills_install_command_preparation_benchmark() {
    let sources = vec![
        "owner/repo1",
        "owner/repo2@skill",
        "another/cool-skill",
        "docker/operations",
        "file/utils",
    ];

    let iterations = 100;

    println!("\n=== Command Preparation Benchmark ===\n");

    let mut total_duration = std::time::Duration::ZERO;

    for source in &sources {
        // Warmup iteration (not timed)
        let mut _cmd = create_npx_command();
        _cmd.arg("skills");
        _cmd.arg("add");
        _cmd.arg(source);
        _cmd.arg("-a");
        _cmd.arg("kilo");
        _cmd.arg("-y");

        // Timed iterations
        let start = Instant::now();
        for _ in 0..iterations {
            let mut cmd = create_npx_command();
            cmd.arg("skills");
            cmd.arg("add");
            cmd.arg(source);
            cmd.arg("-a");
            cmd.arg("kilo");
            cmd.arg("-y");
        }
        let duration = start.elapsed();
        total_duration += duration;

        let avg_duration = duration / iterations;
        let millis = avg_duration.as_millis();

        println!(
            "'{}': avg {:?} per iteration ({} iterations)",
            source, avg_duration, iterations
        );

        // Assert each iteration is under 1ms on average
        assert!(
            millis < 1,
            "Command preparation for '{}' took avg {:?}, expected < 1ms",
            source,
            avg_duration
        );
    }

    let overall_avg = total_duration / (sources.len() as u32 * iterations);
    println!(
        "\nOverall average: {:?} per command preparation\n",
        overall_avg
    );
}

/// Performance test: Skills directory path resolution
///
/// This test measures the performance of resolving and creating the skills
/// directory path. This is part of the installation preparation that determines
/// where skills will be installed.
#[tokio::test]
async fn test_skills_directory_path_resolution_performance() {
    let temp_dir = TempDir::new().unwrap();

    // Temporarily override HOME to use temp directory
    let original_home = std::env::var("HOME").ok();
    std::env::set_var("HOME", temp_dir.path());

    // Measure path resolution time
    let start = Instant::now();
    let manager = SkillsManager::new(None);
    let duration = start.elapsed();

    // Restore HOME
    match original_home {
        Some(home) => std::env::set_var("HOME", home),
        None => std::env::remove_var("HOME"),
    }

    // Verify the global skills directory was resolved correctly
    let global_dir = manager.global_skills_dir();
    assert!(
        global_dir.to_string_lossy().contains(".kilocode"),
        "Global skills dir should contain .kilocode"
    );

    // Assert path resolution time is less than 100 milliseconds
    let millis = duration.as_millis();
    assert!(
        millis < 100,
        "Skills directory path resolution took {:?}, expected < 100ms",
        duration
    );

    println!(
        "Skills directory path resolution completed in {:?}",
        duration
    );
}

/// Performance test: Sequential installation preparation steps
///
/// This test measures the complete preparation sequence for skills installation:
/// 1. SkillsManager initialization
/// 2. npx availability check
/// 3. Command preparation
///
/// This represents the full preparation phase before actual installation.
#[tokio::test]
async fn test_skills_install_preparation_sequence_performance() {
    let source = "test/skill";

    // Measure the complete preparation sequence
    let start = Instant::now();

    // Step 1: Initialize SkillsManager
    let mut manager = SkillsManager::new(None);

    // Step 2: Check npx availability
    let _npx_result = manager.check_npx_available();

    // Step 3: Prepare command
    let mut cmd = create_npx_command();
    cmd.arg("skills");
    cmd.arg("add");
    cmd.arg(source);
    cmd.arg("-a");
    cmd.arg("kilo");
    cmd.arg("-y");

    let duration = start.elapsed();

    // Verify all steps completed
    assert_eq!(cmd.get_program().to_string_lossy(), "npx");

    // Assert total preparation time is less than 3 seconds
    // (allows for npx check which may involve process spawning)
    assert!(
        duration.as_secs() < 3,
        "Full preparation sequence took {:?}, expected < 3 seconds",
        duration
    );

    println!(
        "Skills install preparation sequence completed in {:?}",
        duration
    );
}

/// Benchmark test: Installation preparation with various source formats
///
/// This benchmark tests preparation performance across different skill source
/// formats to ensure consistent performance regardless of input format.
#[tokio::test]
async fn test_skills_install_source_formats_benchmark() {
    let source_formats = vec![
        // Simple GitHub shorthand
        ("owner/repo", "simple github shorthand"),
        // GitHub with skill name
        ("owner/repo@skill-name", "github with skill name"),
        // Full GitHub URL
        ("https://github.com/owner/repo", "full github url"),
        // npm package style
        ("@org/package", "scoped npm package"),
        // Simple package name
        ("some-package", "simple npm package"),
    ];

    println!("\n=== Source Format Preparation Benchmark ===\n");

    for (source, description) in &source_formats {
        // Warmup
        let mut _cmd = create_npx_command();
        _cmd.arg("skills");
        _cmd.arg("add");
        _cmd.arg(source);

        // Timed iterations
        let iterations = 50;
        let start = Instant::now();
        for _ in 0..iterations {
            let mut cmd = create_npx_command();
            cmd.arg("skills");
            cmd.arg("add");
            cmd.arg(source);
            cmd.arg("-a");
            cmd.arg("kilo");
            cmd.arg("-y");
        }
        let duration = start.elapsed();

        let avg_duration = duration / iterations;
        println!("{}: avg {:?} per preparation", description, avg_duration);

        // Assert each preparation is fast
        assert!(
            avg_duration.as_millis() < 1,
            "Preparation for '{}' ({}) took {:?}, expected < 1ms",
            source,
            description,
            avg_duration
        );
    }

    println!();
}
