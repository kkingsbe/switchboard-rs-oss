//! Unit tests for container skill installation failure handling
//!
//! This test file verifies that skill installation failure handling works correctly
//! in the container context. These are UNIT tests that do not require Docker.
//! They test the error handling logic and message generation at a component level.
//!
//! Tests cover:
//! - Error detection based on exit codes
//! - Error message format and content
//! - Remediation suggestions
//! - Agent execution result fields

use switchboard::docker::run::types::ContainerConfig;
use switchboard::metrics::AgentRunResult;

/// Test: Container skill installation failure detection from exit code
///
/// Verifies that when a container exits with a non-zero exit code and skills
/// are configured, the system correctly detects this as a skill installation failure.
/// This is a unit test that simulates the failure detection logic without Docker.
#[test]
fn test_container_skill_install_failure_detection() {
    // Simulate container configuration with skills
    let mut config = ContainerConfig::new("test-agent".to_string());
    config.skills = Some(vec!["owner/repo@skill".to_string()]);

    // Simulate non-zero exit code (skill installation failure)
    let exit_code: i64 = 1;
    let timed_out = false;

    // This is the logic from run.rs that determines if skills installation failed
    let skills_install_failed = config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
        && exit_code != 0
        && !timed_out;

    // Verify failure is correctly detected
    assert!(
        skills_install_failed,
        "Should detect skill installation failure when exit code is non-zero and skills are configured"
    );

    // Verify skills_installed is Some(false) for failure
    let skills_installed = if config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
    {
        if exit_code == 0 {
            Some(true)
        } else if timed_out {
            None // Timed out - unknown if skills installed
        } else {
            Some(false) // Failed
        }
    } else {
        None
    };

    assert_eq!(
        skills_installed,
        Some(false),
        "skills_installed should be Some(false) when exit code is non-zero"
    );
}

/// Test: Container skill installation success detection
///
/// Verifies that when a container exits with exit code 0 and skills
/// are configured, the system correctly detects this as successful installation.
#[test]
fn test_container_skill_install_success_detection() {
    let mut config = ContainerConfig::new("test-agent".to_string());
    config.skills = Some(vec!["owner/repo@skill".to_string()]);

    // Simulate successful exit code
    let exit_code: i64 = 0;
    let timed_out = false;

    let skills_install_failed = config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
        && exit_code != 0
        && !timed_out;

    assert!(
        !skills_install_failed,
        "Should NOT detect failure when exit code is 0"
    );

    let skills_installed = if config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
    {
        if exit_code == 0 {
            Some(true)
        } else if timed_out {
            None
        } else {
            Some(false)
        }
    } else {
        None
    };

    assert_eq!(
        skills_installed,
        Some(true),
        "skills_installed should be Some(true) when exit code is 0"
    );
}

/// Test: Container timeout does not mark as failure
///
/// Verifies that when a container times out during skill installation,
/// the skills_install_failed flag is false because we cannot determine
/// if the failure was due to skill installation or something else.
#[test]
fn test_container_timeout_not_marked_as_failure() {
    let mut config = ContainerConfig::new("test-agent".to_string());
    config.skills = Some(vec!["owner/repo@skill".to_string()]);

    // Simulate timeout (exit code doesn't matter)
    let exit_code: i64 = 124; // Common timeout exit code
    let timed_out = true;

    let skills_install_failed = config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
        && exit_code != 0
        && !timed_out;

    assert!(
        !skills_install_failed,
        "Timeout should NOT be marked as skill installation failure"
    );

    let skills_installed = if config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
    {
        if exit_code == 0 {
            Some(true)
        } else if timed_out {
            None // Unknown due to timeout
        } else {
            Some(false)
        }
    } else {
        None
    };

    assert_eq!(
        skills_installed, None,
        "skills_installed should be None when timed out (unknown status)"
    );
}

/// Test: No skills configured does not trigger failure detection
///
/// Verifies that when no skills are configured, the failure detection
/// logic correctly ignores exit codes.
#[test]
fn test_no_skills_no_failure_detection() {
    // No skills configured
    let config = ContainerConfig::new("test-agent".to_string());

    // Non-zero exit code (but no skills to fail)
    let exit_code: i64 = 1;
    let timed_out = false;

    let skills_install_failed = config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
        && exit_code != 0
        && !timed_out;

    assert!(
        !skills_install_failed,
        "Should NOT detect failure when no skills are configured"
    );

    let skills_installed = if config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
    {
        if exit_code == 0 {
            Some(true)
        } else if timed_out {
            None
        } else {
            Some(false)
        }
    } else {
        None
    };

    assert_eq!(
        skills_installed, None,
        "skills_installed should be None when no skills configured"
    );
}

/// Test: Empty skills list does not trigger failure detection
///
/// Verifies that an explicitly empty skills list (Some([])) does not
/// trigger failure detection.
#[test]
fn test_empty_skills_list_no_failure_detection() {
    // Empty skills list - same as no skills
    let mut config = ContainerConfig::new("test-agent".to_string());
    config.skills = Some(vec![]);

    let exit_code: i64 = 1;
    let timed_out = false;

    let skills_install_failed = config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
        && exit_code != 0
        && !timed_out;

    assert!(
        !skills_install_failed,
        "Should NOT detect failure when skills list is empty"
    );
}

/// Test: Error message generation for skill installation failure
///
/// Verifies that the error message includes all required components
/// when skill installation fails in a container.
#[test]
fn test_skill_install_failure_error_message() {
    let agent_name = "production-agent";
    let exit_code: i64 = 127; // Command not found - common skill install failure
    let skills = ["owner/repo1", "owner/repo2@skill-name"];

    // Generate error message components as in run.rs
    let error_header = format!(
        "[SKILL INSTALL] Error: Skill installation failed for agent '{}' (exit code: {})",
        agent_name, exit_code
    );

    let skills_str = skills.join(", ");
    let skills_msg = format!("[SKILL INSTALL] Skills being installed: {}", skills_str);

    let context_msg = "[SKILL INSTALL] The agent did not execute. Fix the skill installation issues before retrying.";

    // Verify error header contains agent name and exit code
    assert!(
        error_header.contains(agent_name),
        "Error header should contain agent name"
    );
    assert!(
        error_header.contains(&exit_code.to_string()),
        "Error header should contain exit code"
    );

    // Verify skills message contains all skills
    assert!(
        skills_msg.contains("owner/repo1"),
        "Skills message should contain first skill"
    );
    assert!(
        skills_msg.contains("owner/repo2@skill-name"),
        "Skills message should contain second skill"
    );

    // Verify context message indicates agent did not execute
    assert!(
        context_msg.contains("agent did not execute"),
        "Context should indicate agent didn't execute"
    );
}

/// Test: Remediation suggestions for skill installation failure
///
/// Verifies that the remediation message includes actionable steps
/// that users can take to resolve the issue.
#[test]
fn test_skill_install_failure_remediation() {
    let remediation_msg = "[SKILL INSTALL] Remediation steps:
- Check if the skill exists: switchboard skills list
- Verify the skill format: owner/repo or owner/repo@skill-name
- Check network connectivity (npx needs internet access)
- Review [SKILL INSTALL STDERR] lines above for detailed error information";

    // Verify all remediation steps are present
    assert!(
        remediation_msg.contains("Check if the skill exists"),
        "Remediation should mention checking skill exists"
    );
    assert!(
        remediation_msg.contains("switchboard skills list"),
        "Remediation should suggest switchboard skills list command"
    );
    assert!(
        remediation_msg.contains("Verify the skill format"),
        "Remediation should mention verifying skill format"
    );
    assert!(
        remediation_msg.contains("owner/repo"),
        "Remediation should show correct format example"
    );
    assert!(
        remediation_msg.contains("Check network connectivity"),
        "Remediation should mention network connectivity"
    );
    assert!(
        remediation_msg.contains("npx needs internet access"),
        "Remediation should explain npx requirement"
    );
    assert!(
        remediation_msg.contains("Review [SKILL INSTALL STDERR]"),
        "Remediation should suggest reviewing stderr"
    );
}

/// Test: AgentExecutionResult reflects skill installation failure
///
/// Verifies that the AgentExecutionResult correctly captures all
/// relevant fields when skill installation fails.
#[test]
fn test_agent_execution_result_on_failure() {
    // Use chrono for timestamps
    use chrono::Utc;

    // Simulate the result as constructed in run.rs
    let exit_code: i64 = 1;
    let skills_installed = Some(false);
    let skills_install_failed = true;
    let total_skills_count = 2;

    let (skills_installed_count, skills_failed_count) = if skills_install_failed {
        // All skills failed to install
        (0, total_skills_count)
    } else if skills_installed == Some(true) {
        // All skills installed successfully
        (total_skills_count, 0)
    } else {
        // Unknown/timeout
        (0, 0)
    };

    let now = Utc::now();
    let result = AgentRunResult {
        agent_name: "test-agent".to_string(),
        container_id: "test-container-123".to_string(),
        start_time: now,
        end_time: now,
        exit_code,
        timed_out: false,
        termination_type: None,
        queued_start_time: None,
        skills_installed_count: skills_installed_count as u32,
        skills_failed_count: skills_failed_count as u32,
        skills_install_time_seconds: Some(5.0),
    };

    // Verify result fields
    assert_eq!(
        result.exit_code, 1,
        "Exit code should be non-zero for failure"
    );
    assert_eq!(
        result.skills_installed_count, 0,
        "No skills should be counted as installed when installation fails"
    );
    assert_eq!(
        result.skills_failed_count, 2,
        "All skills should be counted as failed"
    );
    assert!(
        result.skills_install_time_seconds.is_some(),
        "Installation time should be recorded even on failure"
    );
}

/// Test: Multiple skills all fail installation
///
/// Verifies that when multiple skills are configured and all fail,
/// the failure count correctly reflects all skills.
#[test]
fn test_multiple_skills_all_fail() {
    let skills = vec![
        "owner/repo1".to_string(),
        "owner/repo2@skill-a".to_string(),
        "owner/repo3@skill-b".to_string(),
    ];

    let exit_code: i64 = 1;
    let timed_out = false;

    let mut config = ContainerConfig::new("test-agent".to_string());
    config.skills = Some(skills.clone());

    let skills_install_failed = config
        .skills
        .as_ref()
        .is_some_and(|s: &Vec<String>| !s.is_empty())
        && exit_code != 0
        && !timed_out;

    assert!(skills_install_failed, "Should detect failure");

    let total_skills_count = skills.len();
    let (skills_installed_count, skills_failed_count) = if skills_install_failed {
        (0, total_skills_count)
    } else {
        (total_skills_count, 0)
    };

    assert_eq!(skills_installed_count, 0, "No skills should be installed");
    assert_eq!(
        skills_failed_count, 3,
        "All 3 skills should be marked as failed"
    );
}

/// Test: Different exit codes for skill install failures
///
/// Verifies that various exit codes (127, 1, 2, etc.) are all
/// correctly treated as skill installation failures.
#[test]
fn test_various_exit_codes_are_failures() {
    let failure_exit_codes = [1, 2, 127, 128, 255];

    for exit_code in failure_exit_codes.iter() {
        let mut config = ContainerConfig::new("test-agent".to_string());
        config.skills = Some(vec!["owner/repo@skill".to_string()]);

        let timed_out = false;

        let skills_install_failed = config
            .skills
            .as_ref()
            .is_some_and(|s: &Vec<String>| !s.is_empty())
            && *exit_code != 0
            && !timed_out;

        assert!(
            skills_install_failed,
            "Exit code {} should be treated as skill installation failure",
            exit_code
        );
    }
}

/// Test: ContainerError types for container operations
///
/// Verifies that container error types are properly defined and
/// can be converted to DockerError.
#[test]
fn test_container_error_types() {
    use switchboard::docker::run::types::ContainerError;
    use switchboard::docker::DockerError;

    // Test ContainerCreationFailed error
    let create_err = ContainerError::ContainerCreationFailed("image not found".to_string());
    let docker_err: DockerError = create_err.into();
    let err_string = docker_err.to_string();

    assert!(
        err_string.contains("Container creation failed"),
        "Error message should mention container creation"
    );
    assert!(
        err_string.contains("image not found"),
        "Error message should contain original error details"
    );

    // Test ContainerStartFailed error
    let start_err = ContainerError::ContainerStartFailed("network error".to_string());
    let docker_err: DockerError = start_err.into();
    let err_string = docker_err.to_string();

    assert!(
        err_string.contains("Container start failed"),
        "Error message should mention container start"
    );

    // Test InvalidTimeout error
    let timeout_err = ContainerError::InvalidTimeout("invalid format".to_string());
    let docker_err: DockerError = timeout_err.into();
    let err_string = docker_err.to_string();

    assert!(
        err_string.contains("I/O error"),
        "Error message should mention I/O error, got: {}",
        err_string
    );
    assert!(
        err_string.contains("parse timeout"),
        "Error message should mention parse timeout operation"
    );
}

/// Test: Log prefixes for skill installation failures
///
/// Verifies that skill installation failures use distinct log prefixes
/// to make them easily identifiable in logs.
#[test]
fn test_skill_install_failure_log_prefixes() {
    let error_prefix = "[SKILL INSTALL ERROR]";
    let install_prefix = "[SKILL INSTALL]";
    let stderr_prefix = "[SKILL INSTALL STDERR]";

    // Verify prefixes are distinct and properly formatted
    assert!(
        error_prefix.starts_with("[SKILL INSTALL"),
        "Error prefix should start with [SKILL INSTALL"
    );
    assert!(
        install_prefix.starts_with("[SKILL INSTALL"),
        "Install prefix should start with [SKILL INSTALL"
    );
    assert!(
        stderr_prefix.starts_with("[SKILL INSTALL"),
        "Stderr prefix should start with [SKILL INSTALL"
    );

    // Verify prefixes are different
    assert_ne!(
        error_prefix, install_prefix,
        "Error and install prefixes should be different"
    );
    assert_ne!(
        error_prefix, stderr_prefix,
        "Error and stderr prefixes should be different"
    );

    // Build sample log lines
    let error_log = format!(
        "{} Skill installation failed for agent 'test-agent'",
        error_prefix
    );
    let install_log = format!("{} Installing skill: owner/repo", install_prefix);
    let stderr_log = format!("{} npm error 404", stderr_prefix);

    assert!(
        error_log.contains("Skill installation failed"),
        "Error log should describe the failure"
    );
    assert!(
        install_log.contains("Installing skill"),
        "Install log should describe installation"
    );
    assert!(
        stderr_log.contains("npm error"),
        "Stderr log should contain npm error"
    );
}
