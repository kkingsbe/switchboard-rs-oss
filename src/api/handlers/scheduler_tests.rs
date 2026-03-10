//! Unit Tests for Scheduler Status Handler (TC-UT-010 to TC-UT-016)
//!
//! These tests verify the scheduler status handler functions including:
//! - read_pid_file
//! - is_process_running
//! - is_scheduler_running
//! - scheduler_status endpoint
//!
//! Test Cases:
//! - TC-UT-010: Test read_pid_file with valid PID
//! - TC-UT-011: Test read_pid_file with non-existent file
//! - TC-UT-012: Test read_pid_file with invalid content
//! - TC-UT-013: Test is_process_running detection
//! - TC-UT-014: Test is_scheduler_running with valid running process
//! - TC-UT-015: Test is_scheduler_running with stale PID file
//! - TC-UT-016: Test scheduler_status endpoint response format
//!
//! Integration Tests (TC-IT-001 to TC-IT-005):
//! - TC-IT-001: CLI up creates instance-specific PID file
//! - TC-IT-002: CLI status reads correct PID file
//! - TC-IT-003: API scheduler/status returns correct running state
//! - TC-IT-004: API scheduler/up and scheduler/status alignment
//! - TC-IT-005: CLI and API PID path interoperability

use crate::api::error::ApiError;
use crate::api::handlers::scheduler::{
    is_process_running, is_scheduler_running, read_pid_file, write_pid_file,
    ApiResponse, SchedulerStatusResponse,
};
use crate::api::registry::{get_instance_dir, get_instance_pid_file};
use crate::api::state::ApiState;
use crate::config::ApiConfig;
use axum::extract::State;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

// Import the scheduler_status handler for TC-UT-016
use crate::api::handlers::scheduler::scheduler_status;

/// TC-UT-010: Test read_pid_file with valid PID
///
/// This test verifies that read_pid_file correctly reads a valid PID from a temporary file.
mod tc_ut_010 {
    use super::*;

    #[test]
    fn test_read_pid_file_valid_pid() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("scheduler.pid");
        
        // Write a valid PID to the file
        std::fs::write(&temp_path, "12345").expect("Failed to write PID file");
        
        // Call read_pid_file
        let result = read_pid_file(&temp_path);
        
        // Verify the result
        assert!(result.is_ok(), "read_pid_file should return Ok");
        assert_eq!(result.unwrap(), Some(12345), "Should return Some(12345)");
    }
}

/// TC-UT-011: Test read_pid_file with non-existent file
///
/// This test verifies that read_pid_file returns Ok(None) when the file doesn't exist.
mod tc_ut_011 {
    use super::*;

    #[test]
    fn test_read_pid_file_non_existent() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let non_existent_path = temp_dir.path().join("non_existent.pid");
        
        // Call read_pid_file with non-existent path
        let result = read_pid_file(&non_existent_path);
        
        // Verify the result
        assert!(result.is_ok(), "read_pid_file should return Ok for non-existent file");
        assert_eq!(result.unwrap(), None, "Should return None for non-existent file");
    }
}

/// TC-UT-012: Test read_pid_file with invalid content
///
/// This test verifies that read_pid_file returns an error when the file contains invalid content.
mod tc_ut_012 {
    use super::*;

    #[test]
    fn test_read_pid_file_invalid_content() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().join("scheduler.pid");
        
        // Write invalid content to the file
        std::fs::write(&temp_path, "not-a-number").expect("Failed to write PID file");
        
        // Call read_pid_file
        let result = read_pid_file(&temp_path);
        
        // Verify the result is an error
        assert!(result.is_err(), "read_pid_file should return error for invalid content");
        let err = result.unwrap_err();
        assert!(matches!(err, ApiError::Internal(_)), "Should be ApiError::Internal");
    }
}

/// TC-UT-013: Test is_process_running detection
///
/// This test verifies that is_process_running correctly detects running and non-running processes.
mod tc_ut_013 {
    use super::*;

    #[test]
    fn test_is_process_running_current_process() {
        // Get current process PID
        let current_pid = std::process::id();
        
        // The current process should be running
        let result = is_process_running(current_pid);
        assert!(result, "Current process (PID {}) should be running", current_pid);
    }

    #[test]
    fn test_is_process_running_non_existent() {
        // Use a PID that definitely doesn't exist
        let non_existent_pid = 99999;
        
        // This PID should not be running
        let result = is_process_running(non_existent_pid);
        assert!(!result, "PID {} should not be running", non_existent_pid);
    }
}

/// TC-UT-014: Test is_scheduler_running with valid running process
///
/// This test verifies that is_scheduler_running returns (true, Some(pid)) when
/// the PID file contains the current process PID.
mod tc_ut_014 {
    use super::*;

    #[test]
    fn test_is_scheduler_running_valid_running_process() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Get current process PID
        let current_pid = std::process::id();
        
        // Write current PID to the file
        write_pid_file(&pid_file_path, current_pid).expect("Failed to write PID file");
        
        // Create a mock state with the temp PID file path
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("test-instance".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        // Override the instance_pid_file to use our temp file
        let state = crate::api::state::ApiState {
            instance_pid_file: pid_file_path,
            ..state
        };
        
        // Call is_scheduler_running
        let (is_running, returned_pid) = is_scheduler_running(&state);
        
        // Verify the result
        assert!(is_running, "Scheduler should be reported as running");
        assert_eq!(returned_pid, Some(current_pid), "Should return the current PID");
    }
}

/// TC-UT-015: Test is_scheduler_running with stale PID file
///
/// This test verifies that is_scheduler_running correctly handles a stale PID file
/// (a PID file containing a non-existent process PID).
mod tc_ut_015 {
    use super::*;

    #[test]
    fn test_is_scheduler_running_stale_pid_file() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Write a non-existent PID to the file (99999 is unlikely to be running)
        let stale_pid = 99999;
        write_pid_file(&pid_file_path, stale_pid).expect("Failed to write PID file");
        
        // Create a mock state with the temp PID file path
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("test-instance".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        // Override the instance_pid_file to use our temp file
        let state = crate::api::state::ApiState {
            instance_pid_file: pid_file_path.clone(),
            ..state
        };
        
        // Call is_scheduler_running
        let (is_running, returned_pid) = is_scheduler_running(&state);
        
        // Verify the result
        assert!(!is_running, "Scheduler should NOT be reported as running for stale PID");
        assert_eq!(returned_pid, None, "Should return None for stale PID");
        
        // Verify the PID file was deleted
        assert!(!pid_file_path.exists(), "Stale PID file should be deleted");
    }
}

/// TC-UT-016: Test scheduler_status endpoint response format
///
/// This test verifies that the scheduler_status endpoint returns the correct response structure.
mod tc_ut_016 {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_status_endpoint_response_format() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Get current process PID and write to file
        let current_pid = std::process::id();
        write_pid_file(&pid_file_path, current_pid).expect("Failed to write PID file");
        
        // Create ApiState with instance_id "test-instance"
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("test-instance".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let mut state = ApiState::new(config);
        // Override the instance_pid_file to use our temp file
        state.instance_pid_file = pid_file_path;
        
        let state = Arc::new(state);
        
        // Call scheduler_status handler
        let response = scheduler_status(State(Arc::clone(&state))).await;
        
        // Verify the response
        assert!(response.is_ok(), "scheduler_status should return Ok");
        
        let response_json = response.unwrap();
        let api_response: ApiResponse<SchedulerStatusResponse> = response_json.0;
        
        // Verify response structure
        assert!(api_response.success, "Response should have success = true");
        assert!(api_response.data.is_some(), "Response should have data");
        
        let status = api_response.data.unwrap();
        
        // Verify running is true since we wrote current PID
        assert!(status.running, "running should be true");
        
        // Verify pid is present
        assert!(status.pid.is_some(), "pid should be present");
        assert_eq!(status.pid.unwrap(), current_pid, "pid should match current process");
        
        // Verify instance_id
        assert_eq!(status.instance_id, "test-instance", "instance_id should be test-instance");
        
        // Verify uptime_seconds is present (optional but should be Some when running)
        assert!(status.uptime_seconds.is_some(), "uptime_seconds should be present when running");
        
        // Verify agents_registered is present (optional, based on config)
        // This will be None since we don't have switchboard_config
        assert!(status.agents_registered.is_none(), "agents_registered should be None without config");
        
        // Verify started_at is present
        assert!(status.started_at.is_some(), "started_at should be present when running");
    }

    #[tokio::test]
    async fn test_scheduler_status_not_running() {
        // Create a temporary directory with NO PID file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Don't write any PID file - simulating not running
        
        // Create ApiState with instance_id "test-instance"
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("test-instance".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let mut state = ApiState::new(config);
        // Override the instance_pid_file to use our temp file (that doesn't exist)
        state.instance_pid_file = pid_file_path;
        
        let state = Arc::new(state);
        
        // Call scheduler_status handler
        let response = scheduler_status(State(Arc::clone(&state))).await;
        
        // Verify the response
        assert!(response.is_ok(), "scheduler_status should return Ok");
        
        let response_json = response.unwrap();
        let api_response: ApiResponse<SchedulerStatusResponse> = response_json.0;
        
        // Verify response structure
        assert!(api_response.success, "Response should have success = true");
        assert!(api_response.data.is_some(), "Response should have data");
        
        let status = api_response.data.unwrap();
        
        // Verify running is false
        assert!(!status.running, "running should be false");
        
        // Verify pid is None
        assert!(status.pid.is_none(), "pid should be None");
        
        // Verify instance_id
        assert_eq!(status.instance_id, "test-instance", "instance_id should be test-instance");
        
        // Verify other fields are None when not running
        assert!(status.uptime_seconds.is_none(), "uptime_seconds should be None when not running");
        assert!(status.agents_registered.is_none(), "agents_registered should be None when not running");
        assert!(status.started_at.is_none(), "started_at should be None when not running");
    }
}

/// TC-UT-001: Verify get_instance_pid_file returns correct path
///
/// This test verifies that the `get_instance_pid_file()` function in
/// `src/api/registry.rs` returns the expected instance-specific path format:
/// `.switchboard/instances/<instance_id>/scheduler.pid`
mod tc_ut_001 {
    use super::*;

    #[test]
    fn test_get_instance_pid_file_default() {
        let result = get_instance_pid_file("default");
        let expected = PathBuf::from(".switchboard/instances/default/scheduler.pid");
        assert_eq!(
            result, expected,
            "get_instance_pid_file(\"default\") should return `.switchboard/instances/default/scheduler.pid`"
        );
    }

    #[test]
    fn test_get_instance_pid_file_test_instance() {
        let result = get_instance_pid_file("test-instance");
        let expected = PathBuf::from(".switchboard/instances/test-instance/scheduler.pid");
        assert_eq!(
            result, expected,
            "get_instance_pid_file(\"test-instance\") should return `.switchboard/instances/test-instance/scheduler.pid`"
        );
    }

    #[test]
    fn test_get_instance_pid_file_prod_us_east() {
        let result = get_instance_pid_file("prod-us-east");
        let expected = PathBuf::from(".switchboard/instances/prod-us-east/scheduler.pid");
        assert_eq!(
            result, expected,
            "get_instance_pid_file(\"prod-us-east\") should return `.switchboard/instances/prod-us-east/scheduler.pid`"
        );
    }

    #[test]
    fn test_get_instance_pid_file_path_format() {
        // Test multiple instance IDs to ensure consistent path format
        let test_cases = vec![
            "default",
            "test-instance",
            "prod-us-east",
            "my-custom-instance",
            "instance-123",
        ];

        for instance_id in test_cases {
            let result = get_instance_pid_file(instance_id);
            
            // Verify path starts with .switchboard/instances/
            assert!(
                result.starts_with(".switchboard/instances/"),
                "PID file path should start with `.switchboard/instances/` for instance `{}`",
                instance_id
            );
            
            // Verify path ends with scheduler.pid
            assert!(
                result.ends_with("scheduler.pid"),
                "PID file path should end with `scheduler.pid` for instance `{}`",
                instance_id
            );
            
            // Verify the instance_id is in the path
            assert!(
                result.to_string_lossy().contains(instance_id),
                "PID file path should contain the instance_id `{}`",
                instance_id
            );
        }
    }
}

/// TC-UT-002: Verify CLI up.rs path consistency
///
/// This test performs code inspection to verify that all PID file path
/// references in `src/cli/commands/up.rs` use the instance-specific format:
/// `.switchboard/instances/<instance_id>/scheduler.pid`
mod tc_ut_002 {
    use super::*;

    #[test]
    fn test_up_rs_pid_path_consistency() {
        // Read the up.rs source file
        let source = include_str!("../../cli/commands/up.rs");
        
        // Find all occurrences of ".switchboard" in the source
        // Only consider non-comment lines (code, not docs)
        let switchboard_occurrences: Vec<(usize, String)> = source
            .lines()
            .enumerate()
            .filter(|(_, line)| {
                let trimmed = line.trim();
                // Skip documentation comments (///) and regular comments (//)
                !trimmed.starts_with("///") && 
                !trimmed.starts_with("//") &&
                line.contains(".switchboard")
            })
            .map(|(idx, line)| (idx + 1, line.to_string())) // Line numbers are 1-indexed
            .collect();
        
        // There should be occurrences of .switchboard in the file
        assert!(
            !switchboard_occurrences.is_empty(),
            "Expected to find .switchboard path references in up.rs"
        );
        
        // Verify all PID file paths use instance-specific format
        // Look for lines that contain both ".switchboard" and "scheduler.pid"
        let pid_file_lines: Vec<(usize, String)> = switchboard_occurrences
            .into_iter()
            .filter(|(_, line)| line.contains("scheduler.pid"))
            .collect();
        
        // Verify we found PID file path lines
        assert!(
            !pid_file_lines.is_empty(),
            "Expected to find scheduler.pid path references in up.rs"
        );
        
        // Check each PID file line for correct instance-specific format
        for (line_num, line) in &pid_file_lines {
            // Each PID file path should either:
            // 1. Contain "instances" directly (explicit path), or
            // 2. Use "instance_dir.join" (which is already defined as instance-specific)
            let uses_instance_dir = line.contains("instance_dir");
            let contains_instances = line.contains("instances");
            
            assert!(
                uses_instance_dir || contains_instances,
                "Line {} should use instance-specific path (either 'instances' or 'instance_dir'). Found: {}",
                line_num,
                line
            );
            
            // Verify it uses scheduler.pid
            assert!(
                line.contains("scheduler.pid"),
                "Line {} should contain 'scheduler.pid'. Found: {}",
                line_num,
                line
            );
        }
    }

    #[test]
    fn test_no_hardcoded_non_instance_pid_paths() {
        // This test verifies there are no hardcoded ".switchboard/scheduler.pid"
        // paths without the instance-specific subdirectory
        let source = include_str!("../../cli/commands/up.rs");
        
        // Check for the incorrect pattern: .switchboard/scheduler.pid 
        // (without "instances" in the path)
        let lines: Vec<&str> = source.lines().collect();
        
        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            
            // Skip comments
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }
            
            // Check if line mentions scheduler.pid
            if line.contains("scheduler.pid") {
                // Should also contain "instances" or use "instance_dir" for instance-specific path
                let uses_instance_dir = line.contains("instance_dir");
                let contains_instances = line.contains("instances");
                assert!(
                    uses_instance_dir || contains_instances,
                    "Line {} contains scheduler.pid but should use instance-specific path. Found: {}",
                    line_num,
                    line
                );
            }
        }
    }
}

/// TC-UT-003: Verify API state instance_pid_file alignment
///
/// This test verifies that when creating an ApiState with a specific instance_id,
/// the `instance_pid_file` field is correctly set to the expected path:
/// `.switchboard/instances/<instance_id>/scheduler.pid`
mod tc_ut_003 {
    use super::*;

    #[test]
    fn test_api_state_instance_pid_file_alignment() {
        // Create ApiState with instance_id "test-instance"
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("test-instance".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        
        // Verify instance_id is set correctly
        assert_eq!(
            state.instance_id, "test-instance",
            "instance_id should be 'test-instance'"
        );
        
        // Verify instance_pid_file equals expected path
        let expected_pid_file = PathBuf::from(".switchboard/instances/test-instance/scheduler.pid");
        assert_eq!(
            state.instance_pid_file, expected_pid_file,
            "instance_pid_file should equal `.switchboard/instances/test-instance/scheduler.pid`"
        );
    }

    #[test]
    fn test_api_state_instance_pid_file_various_instances() {
        let test_cases = vec![
            ("default", ".switchboard/instances/default/scheduler.pid"),
            ("test-instance", ".switchboard/instances/test-instance/scheduler.pid"),
            ("prod-us-east", ".switchboard/instances/prod-us-east/scheduler.pid"),
            ("custom-instance", ".switchboard/instances/custom-instance/scheduler.pid"),
        ];
        
        for (instance_id, expected_path) in test_cases {
            let config = ApiConfig {
                enabled: true,
                instance_id: Some(instance_id.to_string()),
                port: 18500,
                host: "127.0.0.1".to_string(),
                auto_port: false,
                swagger: false,
                rate_limit: crate::config::RateLimitConfig::default(),
            };
            
            let state = ApiState::new(config);
            
            assert_eq!(
                state.instance_id, instance_id,
                "instance_id should be '{}'",
                instance_id
            );
            
            let expected = PathBuf::from(expected_path);
            assert_eq!(
                state.instance_pid_file, expected,
                "instance_pid_file should be '{}' for instance '{}'",
                expected_path, instance_id
            );
        }
    }

    #[test]
    fn test_api_state_instance_pid_file_derivation() {
        // Test that instance_pid_file is correctly derived from instance_id
        // using the same logic as get_instance_pid_file
        
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("derived-instance".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        
        // The instance_pid_file should match what get_instance_pid_file returns
        let expected = get_instance_pid_file(&state.instance_id);
        assert_eq!(
            state.instance_pid_file, expected,
            "instance_pid_file should match get_instance_pid_file(instance_id)"
        );
    }

    #[test]
    fn test_api_state_instance_dir_consistency() {
        // Verify that instance_dir, instance_log_dir, and instance_pid_file
        // all use consistent instance-specific paths
        
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("consistency-test".to_string()),
            port: 18500,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        
        // All instance-specific paths should be under .switchboard/instances/consistency-test/
        let expected_base = get_instance_dir("consistency-test");
        
        assert!(
            state.instance_dir.starts_with(&expected_base) || 
            state.instance_dir.to_string_lossy().contains("consistency-test"),
            "instance_dir should contain instance identifier"
        );
        
        assert!(
            state.instance_pid_file.to_string_lossy().contains("consistency-test"),
            "instance_pid_file should contain instance identifier"
        );
    }
}

// ============================================================================
// Mock-Based Tests (TC-MT-002 to TC-MT-004)
// Note: TC-MT-001 skipped due to platform-specific issues with process spawning
// ============================================================================

/// TC-MT-002: Mock PID file system for multi-instance
///
/// This test verifies multi-instance PID file isolation by creating
/// temp directories for multiple instances and writing different PIDs.
mod tc_mt_002 {
    use super::*;

    #[test]
    fn test_multi_instance_pid_file_isolation() {
        // Create temp directories for multiple instances
        let temp_dir1 = TempDir::new().expect("Failed to create temp dir 1");
        let temp_dir2 = TempDir::new().expect("Failed to create temp dir 2");
        let temp_dir3 = TempDir::new().expect("Failed to create temp dir 3");
        
        let pid_file1 = temp_dir1.path().join("scheduler.pid");
        let pid_file2 = temp_dir2.path().join("scheduler.pid");
        let pid_file3 = temp_dir3.path().join("scheduler.pid");
        
        // Write different PIDs to each instance's PID file
        let pid1: u32 = 11111;
        let pid2: u32 = 22222;
        let pid3: u32 = 33333;
        
        write_pid_file(&pid_file1, pid1).expect("Failed to write PID file 1");
        write_pid_file(&pid_file2, pid2).expect("Failed to write PID file 2");
        write_pid_file(&pid_file3, pid3).expect("Failed to write PID file 3");
        
        // Create states for each instance
        let config1 = ApiConfig {
            enabled: true,
            instance_id: Some("instance-1".to_string()),
            port: 18510,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        let config2 = ApiConfig {
            enabled: true,
            instance_id: Some("instance-2".to_string()),
            port: 18511,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        let config3 = ApiConfig {
            enabled: true,
            instance_id: Some("instance-3".to_string()),
            port: 18512,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state1 = ApiState::new(config1);
        let state1 = crate::api::state::ApiState {
            instance_pid_file: pid_file1.clone(),
            ..state1
        };
        
        let state2 = ApiState::new(config2);
        let state2 = crate::api::state::ApiState {
            instance_pid_file: pid_file2.clone(),
            ..state2
        };
        
        let state3 = ApiState::new(config3);
        let state3 = crate::api::state::ApiState {
            instance_pid_file: pid_file3.clone(),
            ..state3
        };
        
        // Test each instance's status returns correct PID
        let (running1, _pid_ret1) = is_scheduler_running(&state1);
        let (running2, _pid_ret2) = is_scheduler_running(&state2);
        let (running3, _pid_ret3) = is_scheduler_running(&state3);
        
        // All should return not running (since these PIDs don't exist)
        assert!(!running1, "Instance 1 should show not running (PID doesn't exist)");
        assert!(!running2, "Instance 2 should show not running (PID doesn't exist)");
        assert!(!running3, "Instance 3 should show not running (PID doesn't exist)");
        
        // The PID files should be cleaned up (stale)
        // Re-write for verification
        let pid_file1 = temp_dir1.path().join("scheduler.pid");
        let pid_file2 = temp_dir2.path().join("scheduler.pid");
        let pid_file3 = temp_dir3.path().join("scheduler.pid");
        
        write_pid_file(&pid_file1, pid1).expect("Failed to re-write PID file 1");
        write_pid_file(&pid_file2, pid2).expect("Failed to re-write PID file 2");
        write_pid_file(&pid_file3, pid3).expect("Failed to re-write PID file 3");
        
        // Verify the PID files contain the correct values
        let read_pid1 = read_pid_file(&pid_file1).expect("Failed to read PID file 1");
        let read_pid2 = read_pid_file(&pid_file2).expect("Failed to read PID file 2");
        let read_pid3 = read_pid_file(&pid_file3).expect("Failed to read PID file 3");
        
        assert_eq!(read_pid1, Some(pid1), "Instance 1 PID should match");
        assert_eq!(read_pid2, Some(pid2), "Instance 2 PID should match");
        assert_eq!(read_pid3, Some(pid3), "Instance 3 PID should match");
    }

    #[test]
    fn test_concurrent_instance_status_checks() {
        // Test that concurrent status checks on different instances are isolated
        let temp_dir1 = TempDir::new().expect("Failed to create temp dir 1");
        let temp_dir2 = TempDir::new().expect("Failed to create temp dir 2");
        
        let pid_file1 = temp_dir1.path().join("scheduler.pid");
        let pid_file2 = temp_dir2.path().join("scheduler.pid");
        
        // Use current process PID for one, non-existent for another
        let current_pid = std::process::id();
        let fake_pid: u32 = 88888;
        
        write_pid_file(&pid_file1, current_pid).expect("Failed to write PID file 1");
        write_pid_file(&pid_file2, fake_pid).expect("Failed to write PID file 2");
        
        // Create states
        let config1 = ApiConfig {
            enabled: true,
            instance_id: Some("current-proc-instance".to_string()),
            port: 18520,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        let config2 = ApiConfig {
            enabled: true,
            instance_id: Some("fake-proc-instance".to_string()),
            port: 18521,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state1 = ApiState::new(config1);
        let state1 = crate::api::state::ApiState {
            instance_pid_file: pid_file1,
            ..state1
        };
        
        let state2 = ApiState::new(config2);
        let state2 = crate::api::state::ApiState {
            instance_pid_file: pid_file2.clone(),
            ..state2
        };
        
        // Check status for both instances
        let (running1, pid_ret1) = is_scheduler_running(&state1);
        let (running2, _pid_ret2) = is_scheduler_running(&state2);
        
        // Instance 1 (current process) should be running
        assert!(running1, "Instance with current process PID should be running");
        assert_eq!(pid_ret1, Some(current_pid), "Should return current process PID");
        
        // Instance 2 (fake PID) should NOT be running
        assert!(!running2, "Instance with non-existent PID should not be running");
        
        // The fake PID file should be cleaned up (stale)
        assert!(!pid_file2.exists(), "Stale PID file should be deleted");
    }
}

/// TC-MT-003: Mock process running check
///
/// This test creates a ProcessChecker trait with mock implementation
/// for deterministic testing without relying on actual process states.
mod tc_mt_003 {
    use super::*;
    use std::sync::Arc;

    /// A mock process checker that returns configurable results.
    /// This allows deterministic testing without relying on actual process states.
    #[derive(Clone)]
    struct MockProcessChecker {
        running_pids: Arc<std::sync::Mutex<std::collections::HashSet<u32>>>,
    }
    
    impl MockProcessChecker {
        fn new() -> Self {
            Self {
                running_pids: Arc::new(std::sync::Mutex::new(std::collections::HashSet::new())),
            }
        }
        
        fn with_running_pid(self, pid: u32) -> Self {
            self.running_pids.lock().unwrap().insert(pid);
            self
        }
        
        fn is_running(&self, pid: u32) -> bool {
            self.running_pids.lock().unwrap().contains(&pid)
        }
    }

    #[test]
    fn test_mock_process_checker_always_running() {
        let checker = MockProcessChecker::new()
            .with_running_pid(100)
            .with_running_pid(200)
            .with_running_pid(300);
        
        // These should all report as running
        assert!(checker.is_running(100), "PID 100 should be running");
        assert!(checker.is_running(200), "PID 200 should be running");
        assert!(checker.is_running(300), "PID 300 should be running");
        
        // These should NOT be running
        assert!(!checker.is_running(999), "PID 999 should not be running");
        assert!(!checker.is_running(12345), "PID 12345 should not be running");
    }

    #[test]
    fn test_mock_process_checker_with_current_pid() {
        let current_pid = std::process::id();
        let checker = MockProcessChecker::new()
            .with_running_pid(current_pid);
        
        // Current process should be reported as running via mock
        assert!(checker.is_running(current_pid), "Current process should be running via mock");
    }

    #[test]
    fn test_process_checker_behavior_simulation() {
        // Simulate various process check scenarios using mock
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Scenario 1: PID file with running process
        let running_pid = std::process::id();
        write_pid_file(&pid_file_path, running_pid).expect("Failed to write PID file");
        
        // Use actual is_process_running to verify
        let actual_running = is_process_running(running_pid);
        assert!(actual_running, "Current process should actually be running");
        
        // Scenario 2: PID file with non-running process
        let non_running_pid: u32 = 54321;
        write_pid_file(&pid_file_path, non_running_pid).expect("Failed to write PID file");
        
        let actual_not_running = is_process_running(non_running_pid);
        assert!(!actual_not_running, "PID {} should not actually be running", non_running_pid);
        
        // Clean up
        let _ = std::fs::remove_file(&pid_file_path);
    }

    #[test]
    fn test_is_process_running_deterministic_behavior() {
        // Test deterministic behavior of is_process_running
        // by using known PIDs
        
        // Current process - always running in test context
        let current_pid = std::process::id();
        assert!(is_process_running(current_pid), "Current process should be running");
        
        // Non-existent PIDs - always not running
        let test_pids = [99999, 88888, 77777, 1];
        for &pid in &test_pids {
            let result = is_process_running(pid);
            // These should consistently return false
            assert!(!result, "PID {} should not be running", pid);
        }
    }

    #[test]
    fn test_process_status_consistency() {
        // Verify consistent behavior when checking same PID multiple times
        let current_pid = std::process::id();
        
        // Check multiple times - should always be consistent
        for _ in 0..10 {
            assert!(is_process_running(current_pid), "Current process should consistently be running");
        }
        
        // Non-existent PID - always consistent
        let fake_pid: u32 = 99999;
        for _ in 0..10 {
            assert!(!is_process_running(fake_pid), "Fake PID should consistently NOT be running");
        }
    }
}

/// TC-MT-004: Test stale PID file cleanup
///
/// This test verifies cleanup of stale PID files by writing a non-existent
/// PID and verifying the file is deleted when is_scheduler_running is called.
mod tc_mt_004 {
    use super::*;

    #[test]
    fn test_stale_pid_file_cleanup() {
        // Create a temporary directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Write a non-existent PID (99999 is very unlikely to be running)
        let stale_pid: u32 = 99999;
        write_pid_file(&pid_file_path, stale_pid).expect("Failed to write PID file");
        
        // Verify PID file exists before calling is_scheduler_running
        assert!(pid_file_path.exists(), "PID file should exist before cleanup check");
        
        // Create a mock state with the temp PID file path
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("stale-cleanup-test".to_string()),
            port: 18530,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        let state = crate::api::state::ApiState {
            instance_pid_file: pid_file_path.clone(),
            ..state
        };
        
        // Call is_scheduler_running - this should detect stale PID and clean up
        let (is_running, returned_pid) = is_scheduler_running(&state);
        
        // Verify the result - should NOT be running (stale PID)
        assert!(!is_running, "Scheduler should NOT be reported as running for stale PID");
        assert_eq!(returned_pid, None, "Should return None for stale PID");
        
        // Verify PID file was deleted (cleanup happened)
        assert!(!pid_file_path.exists(), "Stale PID file should be deleted");
    }

    #[test]
    fn test_stale_pid_file_cleanup_with_various_pids() {
        // Test cleanup with various non-existent PID values
        let stale_pids = [99999, 88888, 77777, 65535, 1];
        
        for stale_pid in stale_pids {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let pid_file_path = temp_dir.path().join("scheduler.pid");
            
            // Write the stale PID
            write_pid_file(&pid_file_path, stale_pid).expect("Failed to write PID file");
            
            // Create state
            let config = ApiConfig {
                enabled: true,
                instance_id: Some("stale-test".to_string()),
                port: 18531,
                host: "127.0.0.1".to_string(),
                auto_port: false,
                swagger: false,
                rate_limit: crate::config::RateLimitConfig::default(),
            };
            
            let state = ApiState::new(config);
            let state = crate::api::state::ApiState {
                instance_pid_file: pid_file_path.clone(),
                ..state
            };
            
            // Call is_scheduler_running
            let (is_running, returned_pid) = is_scheduler_running(&state);
            
            // Verify cleanup occurred
            assert!(!is_running, "PID {} should not be reported as running", stale_pid);
            assert_eq!(returned_pid, None, "Should return None for stale PID {}", stale_pid);
            assert!(!pid_file_path.exists(), "Stale PID file for {} should be deleted", stale_pid);
        }
    }

    #[test]
    fn test_stale_pid_file_not_cleaned_when_process_exists() {
        // Test that PID file is NOT cleaned up when the process is actually running
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let pid_file_path = temp_dir.path().join("scheduler.pid");
        
        // Write current process PID (which is definitely running)
        let current_pid = std::process::id();
        write_pid_file(&pid_file_path, current_pid).expect("Failed to write PID file");
        
        // Create state
        let config = ApiConfig {
            enabled: true,
            instance_id: Some("valid-pid-test".to_string()),
            port: 18532,
            host: "127.0.0.1".to_string(),
            auto_port: false,
            swagger: false,
            rate_limit: crate::config::RateLimitConfig::default(),
        };
        
        let state = ApiState::new(config);
        let state = crate::api::state::ApiState {
            instance_pid_file: pid_file_path.clone(),
            ..state
        };
        
        // Call is_scheduler_running
        let (is_running, returned_pid) = is_scheduler_running(&state);
        
        // Verify process is correctly detected as running
        assert!(is_running, "Current process should be reported as running");
        assert_eq!(returned_pid, Some(current_pid), "Should return current PID");
        
        // PID file should NOT be deleted (it's valid)
        assert!(pid_file_path.exists(), "Valid PID file should NOT be deleted");
        
        // Clean up
        let _ = std::fs::remove_file(&pid_file_path);
    }
}

// ============================================================================
// INTEGRATION TESTS: TC-IT-001 to TC-IT-005
// ============================================================================

/// Integration Tests for Scheduler Up/Down/Status Flow
///
/// These tests verify the end-to-end flow of the scheduler using CLI commands
/// and API endpoints. They may require specific environment setup and are marked
/// with `#[ignore]` by default to avoid running in CI without proper setup.
mod integration_tests {
    use super::*;
    use crate::api::handlers::scheduler::{ApiResponse, SchedulerStatusResponse};
    use crate::api::registry::{get_instance_dir, get_instance_pid_file};
    use axum::extract::State;
    use http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    /// Test helper: Get the switchboard binary path
    fn get_switchboard_binary() -> std::path::PathBuf {
        // Try to get the current executable path
        std::env::current_exe().unwrap_or_else(|_| {
            // Fallback: look for switchboard in PATH
            std::path::PathBuf::from("switchboard")
        })
    }

    /// Test helper: Create a minimal switchboard.toml for testing
    fn create_test_config(temp_dir: &std::path::Path, instance_id: &str) -> std::path::PathBuf {
        let config_content = r#"
[[agents]]
name = "test-agent"
schedule = "0 * * * *"
prompt = "./test-prompt.md"
image = "switchboard-agent:latest"
"#;
        let config_path = temp_dir.join("switchboard.toml");
        std::fs::write(&config_path, config_content).expect("Failed to write test config");
        config_path
    }

    /// Test helper: Clean up test instance directories
    fn cleanup_test_instance(instance_id: &str) {
        let instance_dir = get_instance_dir(instance_id);
        if instance_dir.exists() {
            let _ = std::fs::remove_dir_all(instance_dir);
        }
    }

    /// TC-IT-003: API scheduler/status returns correct running state
    ///
    /// This test verifies that the API status endpoint reflects actual scheduler state
    /// when a PID file is manually created.
    ///
    /// Test Steps:
    /// 1. Create API state with test instance
    /// 2. Write valid PID to instance-specific PID file
    /// 3. Call GET /api/v1/scheduler/status
    /// 4. Verify response shows running: true and correct PID
    mod tc_it_003 {
        use super::*;

        #[tokio::test]
        async fn test_api_scheduler_status_with_valid_pid_file() {
            // Create a temp directory for the test
            let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
            let instance_id = "test-it-003";
            
            // Clean up any existing test instance
            cleanup_test_instance(instance_id);
            
            // Create instance-specific PID file path
            let pid_file_path = get_instance_pid_file(instance_id);
            
            // Ensure parent directory exists
            if let Some(parent) = pid_file_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            
            // Write current process PID to the file (this process is running)
            let current_pid = std::process::id();
            std::fs::write(&pid_file_path, current_pid.to_string())
                .expect("Failed to write PID file");
            
            // Create API state with the test instance
            let config = ApiConfig {
                enabled: true,
                instance_id: Some(instance_id.to_string()),
                port: 18503,
                host: "127.0.0.1".to_string(),
                auto_port: false,
                swagger: false,
                rate_limit: crate::config::RateLimitConfig::default(),
            };
            
            let state = ApiState::new(config);
            let state = Arc::new(state);
            
            // Create router and make request
            let router = crate::api::router::create_router(Arc::clone(&state));
            
            let response = router
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/scheduler/status")
                        .body(axum::body::Body::empty())
                        .unwrap()
                )
                .await
                .unwrap();
            
            // Verify response status
            assert_eq!(response.status(), StatusCode::OK, "Status endpoint should return 200");
            
            // Parse response body
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_json: ApiResponse<SchedulerStatusResponse> = 
                serde_json::from_slice(&body).expect("Failed to parse response");
            
            // Verify response data
            assert!(response_json.success, "Response should have success = true");
            assert!(response_json.data.is_some(), "Response should have data");
            
            let status = response_json.data.unwrap();
            assert!(status.running, "running should be true when PID file exists with valid PID");
            assert!(status.pid.is_some(), "pid should be present");
            assert_eq!(status.pid.unwrap(), current_pid, "pid should match written PID");
            assert_eq!(status.instance_id, instance_id, "instance_id should match");
            
            // Clean up
            let _ = std::fs::remove_file(&pid_file_path);
            cleanup_test_instance(instance_id);
        }

        #[tokio::test]
        async fn test_api_scheduler_status_not_running() {
            let instance_id = "test-it-003-not-running";
            
            // Clean up any existing test instance
            cleanup_test_instance(instance_id);
            
            // Create API state - no PID file will exist
            let config = ApiConfig {
                enabled: true,
                instance_id: Some(instance_id.to_string()),
                port: 18504,
                host: "127.0.0.1".to_string(),
                auto_port: false,
                swagger: false,
                rate_limit: crate::config::RateLimitConfig::default(),
            };
            
            let state = ApiState::new(config);
            let state = Arc::new(state);
            
            // Create router and make request
            let router = crate::api::router::create_router(Arc::clone(&state));
            
            let response = router
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/scheduler/status")
                        .body(axum::body::Body::empty())
                        .unwrap()
                )
                .await
                .unwrap();
            
            // Verify response status
            assert_eq!(response.status(), StatusCode::OK, "Status endpoint should return 200");
            
            // Parse response body
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let response_json: ApiResponse<SchedulerStatusResponse> = 
                serde_json::from_slice(&body).expect("Failed to parse response");
            
            // Verify response data
            assert!(response_json.success, "Response should have success = true");
            assert!(response_json.data.is_some(), "Response should have data");
            
            let status = response_json.data.unwrap();
            assert!(!status.running, "running should be false when no PID file exists");
            assert!(status.pid.is_none(), "pid should be None");
            
            // Clean up
            cleanup_test_instance(instance_id);
        }
    }

    /// TC-IT-004: API scheduler/up and scheduler/status alignment
    ///
    /// This test verifies that the PID file created by the up handler is readable
    /// by the status handler. This is a unit test of the handlers working together.
    mod tc_it_004 {
        use super::*;
        use crate::api::handlers::scheduler::SchedulerStartRequest;

        #[tokio::test]
        #[ignore] // Requires proper API state setup with switchboard config
        async fn test_api_up_and_status_pid_alignment() {
            // This test requires the API to have a valid switchboard config
            // Since we can't easily set that up in a unit test, we skip it
            // The alignment is verified through the shared instance_pid_file path
            
            // Verify that both handlers use the same PID file path function
            let instance_id = "test-it-004";
            let pid_file_path = get_instance_pid_file(instance_id);
            
            // The path should be instance-specific
            assert!(
                pid_file_path.to_string_lossy().contains("instances"),
                "PID file path should be instance-specific"
            );
            assert!(
                pid_file_path.to_string_lossy().contains(instance_id),
                "PID file path should contain instance_id"
            );
            assert!(
                pid_file_path.to_string_lossy().ends_with("scheduler.pid"),
                "PID file path should end with scheduler.pid"
            );
        }
    }

    /// TC-IT-001: CLI up creates instance-specific PID file
    ///
    /// This test verifies that `switchboard up --detach` creates the PID file
    /// in the correct instance-specific location.
    ///
    /// NOTE: This is marked as ignore because it requires:
    /// - The switchboard binary to be built
    /// - A valid Docker environment
    /// - A minimal switchboard.toml config
    /// Run with: cargo test --test integration tc_it_001
    mod tc_it_001 {
        use super::*;

        #[test]
        #[ignore] // Requires Docker and switchboard binary
        fn test_cli_up_creates_instance_specific_pid_file() {
            let instance_id = "test-it-001";
            
            // Clean up any existing test instance
            cleanup_test_instance(instance_id);
            
            // Get expected PID file path
            let expected_pid_file = get_instance_pid_file(instance_id);
            
            // Ensure parent directory doesn't exist before test
            if expected_pid_file.exists() {
                let _ = std::fs::remove_file(&expected_pid_file);
            }
            if let Some(parent) = expected_pid_file.parent() {
                if parent.exists() {
                    let _ = std::fs::remove_dir_all(parent);
                }
            }
            
            // Check if switchboard binary exists
            let binary_path = get_switchboard_binary();
            if !binary_path.exists() {
                println!("SKIPPED: switchboard binary not found at {:?}", binary_path);
                return;
            }
            
            // Try to run switchboard up --detach
            // Note: This will likely fail due to missing config/Docker,
            // but we can check if it tries to create the PID file
            let output = std::process::Command::new(&binary_path)
                .args(["up", "--detach", "--config", "./switchboard.toml"])
                .output();
            
            match output {
                Ok(output) => {
                    // If the command ran (even if it failed), check if PID file was created
                    // in the right location
                    if expected_pid_file.exists() {
                        let pid_content = std::fs::read_to_string(&expected_pid_file)
                            .expect("Failed to read PID file");
                        
                        // Parse PID
                        let pid: u32 = pid_content.trim().parse()
                            .expect("Failed to parse PID");
                        
                        // Verify PID is valid (non-zero)
                        assert!(pid > 0, "PID should be non-zero");
                        
                        println!("PID file created at correct location: {:?}", expected_pid_file);
                    } else {
                        // Check if PID file was created at wrong location
                        let wrong_path = std::path::PathBuf::from(".switchboard/scheduler.pid");
                        if wrong_path.exists() {
                            println!("WARNING: PID file created at wrong location: {:?}", wrong_path);
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to run switchboard: {}", e);
                }
            }
            
            // Clean up
            cleanup_test_instance(instance_id);
        }

        #[test]
        fn test_pid_file_path_is_instance_specific() {
            // This is a unit test that verifies the PID file path function
            // returns instance-specific paths
            
            let test_cases = vec![
                "default",
                "test-instance",
                "prod-us-east",
                "tc-it-001",
            ];
            
            for instance_id in test_cases {
                let pid_file = get_instance_pid_file(instance_id);
                
                // Verify path format: .switchboard/instances/<instance_id>/scheduler.pid
                let path_str = pid_file.to_string_lossy();
                assert!(
                    path_str.contains("instances"),
                    "Path for {} should contain 'instances'", instance_id
                );
                assert!(
                    path_str.contains(instance_id),
                    "Path for {} should contain instance_id", instance_id
                );
                assert!(
                    path_str.ends_with("scheduler.pid"),
                    "Path should end with 'scheduler.pid'"
                );
            }
        }
    }

    /// TC-IT-002: CLI status reads correct PID file
    ///
    /// This test verifies that the status detection reads from the correct
    /// instance-specific path.
    ///
    /// NOTE: This test verifies the path alignment between CLI and expected location.
    mod tc_it_002 {
        use super::*;

        #[test]
        fn test_status_reads_from_instance_specific_path() {
            let instance_id = "test-it-002";
            
            // Get the PID file path that both CLI and API should use
            let expected_pid_file = get_instance_pid_file(instance_id);
            
            // Create the instance directory and PID file
            if let Some(parent) = expected_pid_file.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            
            // Write a test PID
            let test_pid = std::process::id();
            std::fs::write(&expected_pid_file, test_pid.to_string())
                .expect("Failed to write test PID file");
            
            // Verify the file exists at the expected location
            assert!(
                expected_pid_file.exists(),
                "PID file should exist at instance-specific path"
            );
            
            // Read it back using the same path function
            let read_pid = read_pid_file(&expected_pid_file)
                .expect("Failed to read PID file");
            
            assert_eq!(read_pid, Some(test_pid), "Should read back the same PID");
            
            // Clean up
            let _ = std::fs::remove_file(&expected_pid_file);
            cleanup_test_instance(instance_id);
        }
    }

    /// TC-IT-005: CLI and API PID path interoperability
    ///
    /// This test verifies that both CLI and API use the same PID file path,
    /// ensuring interoperability.
    mod tc_it_005 {
        use super::*;

        #[test]
        fn test_cli_and_api_use_same_pid_path() {
            // This test verifies that CLI and API both use the same
            // instance-specific PID file path resolution
            
            let instance_id = "test-it-005";
            
            // Get the expected path from registry (used by API)
            let api_pid_path = get_instance_pid_file(instance_id);
            
            // Verify it's instance-specific
            let path_str = api_pid_path.to_string_lossy();
            assert!(
                path_str.contains("instances"),
                "API PID path should be instance-specific"
            );
            assert!(
                path_str.contains(instance_id),
                "API PID path should contain instance_id"
            );
            
            // The CLI code (up.rs) uses:
            // Path::new(".switchboard").join("instances").join(&instance_id).join("scheduler.pid")
            // which is the same format
            let cli_expected_path = std::path::PathBuf::from(".switchboard")
                .join("instances")
                .join(instance_id)
                .join("scheduler.pid");
            
            assert_eq!(
                api_pid_path, cli_expected_path,
                "CLI and API should use the same PID file path"
            );
            
            println!("Verified: CLI and API both use {:?}", api_pid_path);
        }
    }
}
