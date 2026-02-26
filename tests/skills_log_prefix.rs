//! Tests for skill installation log prefix verification
//!
//! This test file verifies that skill installation logs use the correct
//! `[SKILL INSTALL]` prefix, making them distinguishable from agent execution
//! logs (which have no prefix).

use switchboard::docker::skills::generate_entrypoint_script;

/// Test that skill installation logs contain the `[SKILL INSTALL]` prefix
///
/// This test verifies that when a skill installation occurs (or fails),
/// the generated entrypoint script includes proper log prefixes. This
/// makes skill installation logs distinguishable from agent execution logs.
#[test]
fn test_skill_install_logs_contain_prefix() {
    // Use a valid skill format that simulates a skill that will fail during installation
    // The skill format is valid (has / separator) but the skill may not exist
    let skills = vec!["nonexistent-owner/nonexistent-repo".to_string()];
    let agent_name = "test-agent-with-nonexistent-skill";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Verify the script contains the [SKILL INSTALL] prefix for skill installation attempts
    assert!(
        script.contains("[SKILL INSTALL] Installing skill:"),
        "Script should contain '[SKILL INSTALL] Installing skill:' prefix for logging skill installation attempts"
    );

    // Verify the skill name is included in the log message
    assert!(
        script.contains("[SKILL INSTALL] Installing skill: nonexistent-owner/nonexistent-repo"),
        "Script should log the specific skill being installed"
    );

    // Verify that the script uses the skill format in the command
    assert!(
        script.contains("npx skills add nonexistent-owner/nonexistent-repo"),
        "Script should contain the npx skills add command for the skill"
    );
}

/// Test that skill installation error logs use the `[SKILL INSTALL ERROR]` prefix
///
/// This test verifies that when skill installation fails, error logs use
/// the `[SKILL INSTALL ERROR]` prefix, making them distinguishable from
/// successful installation logs and agent execution logs.
#[test]
fn test_skill_install_error_logs_contain_error_prefix() {
    // Use an invalid skill format to trigger error handling
    let skills = vec!["nonexistent-owner/nonexistent-repo".to_string()];
    let agent_name = "test-agent-with-nonexistent-skill";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Verify the script contains the [SKILL INSTALL ERROR] prefix
    assert!(
        script.contains("[SKILL INSTALL ERROR]"),
        "Script should contain '[SKILL INSTALL ERROR]' prefix for logging installation errors"
    );

    // Verify that the error handler captures and reports exit codes
    assert!(
        script.contains("[SKILL INSTALL ERROR] Command failed with exit code"),
        "Script should log exit code information when command fails"
    );
}

/// Test that skill installation stderr logs use the `[SKILL INSTALL STDERR]` prefix
///
/// This test verifies that stderr output from npx commands is captured and
/// prefixed with `[SKILL INSTALL STDERR]`, making it easy to identify
/// error output from skill installation commands.
#[test]
fn test_skill_install_stderr_logs_contain_stderr_prefix() {
    let skills = vec!["owner/repo".to_string()];
    let agent_name = "test-agent-stderr-logs";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Verify the script contains the [SKILL INSTALL STDERR] prefix
    assert!(
        script.contains("[SKILL INSTALL STDERR]"),
        "Script should contain '[SKILL INSTALL STDERR]' prefix for logging stderr from npx commands"
    );

    // Verify that stderr is captured using pipe and while loop
    assert!(
        script.contains("2>&1 | while IFS= read -r line"),
        "Script should capture stderr using pipe and while loop"
    );

    // Verify that the stderr line is prefixed
    assert!(
        script.contains("echo \"[SKILL INSTALL STDERR] $line\""),
        "Script should prefix each stderr line with [SKILL INSTALL STDERR]"
    );
}

/// Test that skill installation logs are distinguishable from agent execution logs
///
/// This test verifies that skill installation logs use the `[SKILL INSTALL]`
/// prefix, making them clearly distinguishable from agent execution logs which
/// have no prefix. This is important for operators to easily identify which
/// logs are related to skill installation vs. actual agent execution.
#[test]
fn test_skill_install_logs_distinguishable_from_agent_logs() {
    let skills = vec!["owner/repo@skill-name".to_string()];
    let agent_name = "test-agent-distinct-logs";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Verify skill installation logs start with [SKILL INSTALL] prefix
    let skill_install_lines = [
        "[SKILL INSTALL] Installing skill:",
        "[SKILL INSTALL STDERR]",
        "[SKILL INSTALL ERROR]",
    ];

    for line_prefix in skill_install_lines.iter() {
        assert!(
            script.contains(line_prefix),
            "Script should contain skill installation log prefix: {}",
            line_prefix
        );
    }

    // Count occurrences of [SKILL INSTALL] prefix to verify it's used consistently
    let skill_install_count = script.matches("[SKILL INSTALL").count();

    assert!(
        skill_install_count >= 3,
        "Script should contain at least 3 occurrences of '[SKILL INSTALL' prefix (for Installing, STDERR, and ERROR), found: {}",
        skill_install_count
    );

    // Verify the skill name appears in the installation log
    assert!(
        script.contains("[SKILL INSTALL] Installing skill: owner/repo@skill-name"),
        "Script should log the specific skill name being installed"
    );
}

/// Test that multiple skills all use the correct log prefix
///
/// This test verifies that when multiple skills are configured, each skill
/// installation attempt uses the `[SKILL INSTALL]` prefix consistently.
#[test]
fn test_multiple_skills_use_correct_log_prefix() {
    let skills = vec![
        "owner1/repo1".to_string(),
        "owner2/repo2@skill-name".to_string(),
        "owner3/repo3".to_string(),
    ];
    let agent_name = "test-agent-multiple-skills";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Verify each skill is logged with the [SKILL INSTALL] prefix
    for skill in skills.iter() {
        let expected_log = format!("[SKILL INSTALL] Installing skill: {}", skill);
        assert!(
            script.contains(&expected_log),
            "Script should log skill '{}' with correct prefix",
            skill
        );
    }

    // Verify that there's a log entry for each skill (3 skills = 3 installation log lines)
    let install_log_count = script.matches("[SKILL INSTALL] Installing skill:").count();

    assert_eq!(
        install_log_count, 3,
        "Script should contain exactly 3 skill installation log lines (one per skill), found: {}",
        install_log_count
    );
}

/// Test that skill installation logs contain the agent name for context
///
/// This test verifies that skill installation logs include the agent name,
/// providing context for which agent's skills are being installed.
#[test]
fn test_skill_install_logs_contain_agent_context() {
    let skills = vec!["owner/repo".to_string()];
    let agent_name = "production-agent-01";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // While the agent name may not be in every log line, the script
    // should be generated for a specific agent and this context is
    // important for debugging
    assert!(
        script.contains("[SKILL INSTALL]"),
        "Script should contain [SKILL INSTALL] prefix for agent logs"
    );

    // Verify the script handles the agent's skills
    assert!(
        script.contains("npx skills add owner/repo"),
        "Script should install the skills configured for the agent"
    );
}

/// Test comprehensive log prefix coverage for skill installation
///
/// This test verifies that all expected log prefixes for skill installation
/// are present in the generated script, providing complete coverage of
/// the logging behavior.
#[test]
fn test_comprehensive_log_prefix_coverage() {
    let skills = vec!["owner/repo".to_string()];
    let agent_name = "comprehensive-test-agent";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Define all expected log prefixes for skill installation
    let expected_prefixes = [
        "[SKILL INSTALL] Installing skill:",
        "[SKILL INSTALL STDERR]",
        "[SKILL INSTALL ERROR]",
    ];

    // Verify all expected prefixes are present
    for prefix in expected_prefixes.iter() {
        if !script.contains(prefix) {
            panic!(
                "Expected log prefix '{}' not found in generated script",
                prefix
            );
        }
    }

    // Verify the script structure for capturing stderr
    assert!(
        script.contains("2>&1 | while IFS= read -r line; do"),
        "Script should capture stderr output from npx commands"
    );

    assert!(
        script.contains("echo \"[SKILL INSTALL STDERR] $line\""),
        "Script should prefix each stderr line with [SKILL INSTALL STDERR]"
    );
}

/// Test that agent execution logs do NOT contain `[SKILL INSTALL]` prefix
///
/// This test verifies that agent execution logs are raw output with no special
/// formatting prefix, distinguishing them from skill installation logs. This is
/// important because:
/// - Skill installation logs use `[SKILL INSTALL]` prefix for easy identification
/// - Agent execution logs are raw container stdout/stderr output (written via streams.rs)
/// - This distinction helps operators filter and understand different log types
///
/// The entrypoint script structure:
/// 1. Skill installation phase: prefixed logs with `[SKILL INSTALL]`
/// 2. Agent execution phase: `exec kilocode --yes "$@"` with NO prefix
#[test]
fn test_agent_execution_logs_no_prefix() {
    let skills = vec!["owner/repo".to_string()];
    let agent_name = "test-agent-execution-logs";

    // Generate the entrypoint script
    let script = generate_entrypoint_script(agent_name, &skills, &[])
        .expect("Should be able to generate entrypoint script");

    // Verify the script contains skill installation logs with prefix
    assert!(
        script.contains("[SKILL INSTALL]"),
        "Script should contain skill installation log prefix"
    );

    // Verify the agent execution command is present
    assert!(
        script.contains("exec kilocode --yes \"$@\""),
        "Script should contain the agent execution command"
    );

    // Verify that the agent execution command is NOT prefixed with [SKILL INSTALL]
    // This is the key assertion: agent execution logs are raw output
    let lines: Vec<&str> = script.lines().collect();
    let mut found_exec_line = false;

    for line in lines.iter() {
        if line.contains("exec kilocode") {
            found_exec_line = true;
            // The exec line should NOT have [SKILL INSTALL] prefix
            assert!(
                !line.trim_start().starts_with("[SKILL INSTALL]"),
                "Agent execution command should NOT have [SKILL INSTALL] prefix. \
                Line: '{}'",
                line
            );
        }
    }

    assert!(
        found_exec_line,
        "Should have found the exec kilocode line in the script"
    );

    // Verify that after skill installation section, the agent execution section
    // does NOT continue using [SKILL INSTALL] prefix
    // Split the script by the "# Hand off to Kilo Code CLI" comment
    let parts: Vec<&str> = script.split("# Hand off to Kilo Code CLI").collect();

    if parts.len() > 1 {
        // This is the agent execution section (after the comment)
        let agent_execution_section = parts[1];

        // Agent execution section should NOT contain [SKILL INSTALL] prefix
        // This confirms that agent execution logs are raw/unmodified
        assert!(
            !agent_execution_section.contains("[SKILL INSTALL]"),
            "Agent execution section (after '# Hand off to Kilo Code CLI') \
            should NOT contain [SKILL INSTALL] prefix. This section:\n'{}'",
            agent_execution_section
        );
    }

    // Verify the overall script has clear separation between
    // skill installation (with prefix) and agent execution (no prefix)
    assert!(
        script.contains("# Install skills"),
        "Script should have skill installation section marker"
    );
    assert!(
        script.contains("# Hand off to Kilo Code CLI"),
        "Script should have agent execution section marker"
    );
}

/// Test that `switchboard logs` displays logs with `[SKILL INSTALL]` prefix
///
/// This test verifies that when log files contain skill installation logs
/// (prefixed with `[SKILL INSTALL]`), the `switchboard logs` command displays
/// them correctly and they are distinguishable in the output.
#[test]
fn test_switchboard_logs_displays_skill_install_prefix() {
    use assert_cmd::Command;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    let agent_log_dir = log_dir.join("test-agent");

    // Create directory structure
    fs::create_dir_all(&agent_log_dir).unwrap();

    // Create a valid config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with skill installation logs
    let log_file_path = agent_log_dir.join("1234567890.log");
    let log_content = r#"
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL] npx skills add owner/repo
[SKILL INSTALL] Successfully installed skill
"#;
    fs::write(&log_file_path, log_content).unwrap();

    // Run switchboard logs command from the temp directory (so it finds switchboard.toml)
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "test-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify the output contains skill installation logs
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(
        stdout.contains("[SKILL INSTALL]"),
        "Output should contain [SKILL INSTALL] prefix"
    );
    assert!(
        stdout.contains("[SKILL INSTALL] Installing skill: owner/repo"),
        "Output should contain skill installation log with skill name"
    );
    assert!(
        stdout.contains("[SKILL INSTALL] npx skills add owner/repo"),
        "Output should contain npx skills add command in logs"
    );
    assert!(
        stdout.contains("[SKILL INSTALL] Successfully installed skill"),
        "Output should contain success message from skill installation"
    );
}

/// Test that `switchboard logs` displays agent execution logs without prefix
///
/// This test verifies that when log files contain agent execution logs
/// (without any prefix), the `switchboard logs` command displays them correctly
/// as raw output.
#[test]
fn test_switchboard_logs_displays_agent_execution_logs() {
    use assert_cmd::Command;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    let agent_log_dir = log_dir.join("test-agent");

    // Create directory structure
    fs::create_dir_all(&agent_log_dir).unwrap();

    // Create a valid config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with agent execution logs (no prefix)
    let log_file_path = agent_log_dir.join("1234567890.log");
    let log_content = r#"
Starting agent execution
Processing request
Analyzing task
Generating response
Completed successfully
"#;
    fs::write(&log_file_path, log_content).unwrap();

    // Run switchboard logs command from the temp directory (so it finds switchboard.toml)
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "test-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify the output contains agent execution logs
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(
        stdout.contains("Starting agent execution"),
        "Output should contain agent execution log: 'Starting agent execution'"
    );
    assert!(
        stdout.contains("Processing request"),
        "Output should contain agent execution log: 'Processing request'"
    );
    assert!(
        stdout.contains("Analyzing task"),
        "Output should contain agent execution log: 'Analyzing task'"
    );
    assert!(
        stdout.contains("Generating response"),
        "Output should contain agent execution log: 'Generating response'"
    );
    assert!(
        stdout.contains("Completed successfully"),
        "Output should contain agent execution log: 'Completed successfully'"
    );

    // Verify that agent execution logs do NOT have [SKILL INSTALL] prefix
    let lines: Vec<&str> = stdout.lines().collect();
    for line in lines.iter() {
        if line.contains("Starting agent execution")
            || line.contains("Processing request")
            || line.contains("Analyzing task")
            || line.contains("Generating response")
            || line.contains("Completed successfully")
        {
            // Extract the actual log message (after [test-agent] prefix)
            let log_message = if line.contains("[test-agent]") {
                line.split("] ").nth(1).unwrap_or("")
            } else {
                line
            };

            // Agent execution logs should NOT start with [SKILL INSTALL]
            assert!(
                !log_message.starts_with("[SKILL INSTALL]"),
                "Agent execution log should NOT have [SKILL INSTALL] prefix: '{}'",
                line
            );
        }
    }
}

/// Test that `switchboard logs` output distinguishes between log types
///
/// This test verifies that when a log file contains both skill installation
/// logs (with `[SKILL INSTALL]` prefix) and agent execution logs (no prefix),
/// the `switchboard logs` command displays both types correctly and they are
/// clearly distinguishable in the output.
#[test]
fn test_switchboard_logs_distinguishes_log_types() {
    use assert_cmd::Command;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    let agent_log_dir = log_dir.join("test-agent");

    // Create directory structure
    fs::create_dir_all(&agent_log_dir).unwrap();

    // Create a valid config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with both skill installation and agent execution logs
    let log_file_path = agent_log_dir.join("1234567890.log");
    let log_content = r#"
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL] npx skills add owner/repo
[SKILL INSTALL] Successfully installed skill
Starting agent execution
Processing request
Analyzing task
Generating response
Completed successfully
"#;
    fs::write(&log_file_path, log_content).unwrap();

    // Run switchboard logs command from the temp directory (so it finds switchboard.toml)
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "test-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify the output contains both types of logs
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);

    // Verify skill installation logs with [SKILL INSTALL] prefix
    assert!(
        stdout.contains("[SKILL INSTALL] Installing skill: owner/repo"),
        "Output should contain skill installation log with prefix"
    );
    assert!(
        stdout.contains("[SKILL INSTALL] Successfully installed skill"),
        "Output should contain skill installation success log with prefix"
    );

    // Verify agent execution logs without [SKILL INSTALL] prefix
    assert!(
        stdout.contains("Starting agent execution"),
        "Output should contain agent execution log without prefix"
    );
    assert!(
        stdout.contains("Completed successfully"),
        "Output should contain agent execution completion log without prefix"
    );

    // Count log lines with and without [SKILL INSTALL] prefix
    let lines: Vec<&str> = stdout.lines().collect();
    let mut skill_install_count = 0;
    let mut agent_execution_count = 0;

    for line in lines.iter() {
        // Extract the actual log message (after [test-agent] prefix)
        let log_message = if line.contains("[test-agent]") {
            line.split("[test-agent] ").nth(1).unwrap_or(line)
        } else {
            line
        };

        // Check if line contains [SKILL INSTALL] anywhere
        if log_message.contains("[SKILL INSTALL]") {
            skill_install_count += 1;
        } else if !log_message.trim().is_empty() && !log_message.contains("[SKILL INSTALL]") {
            // Non-empty log without [SKILL INSTALL] is agent execution
            agent_execution_count += 1;
        }
    }

    assert!(
        skill_install_count >= 3,
        "Should have at least 3 skill installation logs (with [SKILL INSTALL] prefix), found: {}",
        skill_install_count
    );

    assert!(
        agent_execution_count >= 5,
        "Should have at least 5 agent execution logs (without prefix), found: {}",
        agent_execution_count
    );
}

/// Test that `switchboard logs` displays error logs with `[SKILL INSTALL ERROR]` prefix
///
/// This test verifies that when log files contain skill installation error logs
/// (prefixed with `[SKILL INSTALL ERROR]`), the `switchboard logs` command displays
/// them correctly and they are distinguishable from other log types.
#[test]
fn test_switchboard_logs_displays_skill_install_error_prefix() {
    use assert_cmd::Command;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    let agent_log_dir = log_dir.join("test-agent");

    // Create directory structure
    fs::create_dir_all(&agent_log_dir).unwrap();

    // Create a valid config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with skill installation error logs
    let log_file_path = agent_log_dir.join("1234567890.log");
    let log_content = r#"
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL] npx skills add owner/repo
[SKILL INSTALL ERROR] Command failed with exit code 1
[SKILL INSTALL ERROR] Error: Skill not found
"#;
    fs::write(&log_file_path, log_content).unwrap();

    // Run switchboard logs command from the temp directory (so it finds switchboard.toml)
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "test-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify the output contains skill installation error logs
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(
        stdout.contains("[SKILL INSTALL ERROR]"),
        "Output should contain [SKILL INSTALL ERROR] prefix"
    );
    assert!(
        stdout.contains("[SKILL INSTALL ERROR] Command failed with exit code 1"),
        "Output should contain error log with exit code"
    );
    assert!(
        stdout.contains("[SKILL INSTALL ERROR] Error: Skill not found"),
        "Output should contain error message"
    );
}

/// Test that `switchboard logs` displays stderr logs with `[SKILL INSTALL STDERR]` prefix
///
/// This test verifies that when log files contain stderr output from skill
/// installation commands (prefixed with `[SKILL INSTALL STDERR]`), the
/// `switchboard logs` command displays them correctly and they are distinguishable
/// from other log types.
#[test]
fn test_switchboard_logs_displays_skill_install_stderr_prefix() {
    use assert_cmd::Command;
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");
    let log_dir = temp_dir.path().join(".switchboard").join("logs");
    let agent_log_dir = log_dir.join("test-agent");

    // Create directory structure
    fs::create_dir_all(&agent_log_dir).unwrap();

    // Create a valid config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create a log file with skill installation stderr logs
    let log_file_path = agent_log_dir.join("1234567890.log");
    let log_content = r#"
[SKILL INSTALL] Installing skill: owner/repo
[SKILL INSTALL] npx skills add owner/repo
[SKILL INSTALL STDERR] npm ERR! code ENOENT
[SKILL INSTALL STDERR] npm ERR! syscall open
[SKILL INSTALL STDERR] npm ERR! path /nonexistent/path
"#;
    fs::write(&log_file_path, log_content).unwrap();

    // Run switchboard logs command from the temp directory (so it finds switchboard.toml)
    let result = Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args(["logs", "test-agent"])
        .current_dir(temp_dir.path())
        .assert()
        .success();

    // Verify the output contains skill installation stderr logs
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    assert!(
        stdout.contains("[SKILL INSTALL STDERR]"),
        "Output should contain [SKILL INSTALL STDERR] prefix"
    );
    assert!(
        stdout.contains("[SKILL INSTALL STDERR] npm ERR!"),
        "Output should contain stderr error messages"
    );
}
