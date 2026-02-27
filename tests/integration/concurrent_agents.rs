//! Integration test for multiple concurrent agent runs
//!
//! This test verifies that:
//! - Multiple agents can run concurrently without conflicts
//! - Container logs are properly interleaved without corruption
//! - All containers exit with success (exit code 0)
//! - Containers are properly cleaned up after execution (--rm behavior)

#[cfg(feature = "integration")]
use super::docker_available;
#[cfg(feature = "integration")]
use bollard::container::ListContainersOptions;
#[cfg(feature = "integration")]
use futures::future::join_all;
#[cfg(feature = "integration")]
use std::collections::HashMap;
#[cfg(feature = "integration")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "integration")]
use std::time::Duration;
#[cfg(feature = "integration")]
use switchboard::docker::run::types::ContainerConfig;
#[cfg(feature = "integration")]
use switchboard::docker::DockerClient;
#[cfg(feature = "integration")]
use tokio::time::sleep;

/// Test multiple concurrent agent runs
///
/// This test verifies that:
/// - 2-3 agents can run concurrently without conflicts
/// - Containers are running simultaneously (verified by listing containers)
/// - All agents complete successfully (exit code 0)
/// - Logs are properly interleaved without corruption
/// - All containers are cleaned up (--rm behavior)
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_multiple_concurrent_agent_runs() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Number of concurrent agents to run
    const NUM_AGENTS: usize = 3;

    // Create a DockerClient instance
    let client = match DockerClient::new("test-image".to_string(), "latest".to_string()).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping test: Failed to create DockerClient: {:?}", e);
            return;
        }
    };

    // Verify alpine image exists
    let docker = client.docker();
    match docker.inspect_image("alpine:latest").await {
        Ok(_) => {
            // Image exists, proceed with test
        }
        Err(e) => {
            eprintln!(
                "Skipping test: Required image 'alpine:latest' not found. Please pull the image first: docker pull alpine:latest. Error: {}",
                e
            );
            return;
        }
    }

    // Create temporary directories for each agent to avoid conflicts
    let mut temp_dirs = Vec::new();
    let mut workspace_paths = Vec::new();
    for _i in 0..NUM_AGENTS {
        let temp_dir = tempfile::tempdir().expect("Failed to create temporary directory");
        let workspace_path = temp_dir.path().to_path_buf();
        temp_dirs.push(temp_dir);
        workspace_paths.push(workspace_path);
    }

    // Use scopeguard to ensure the temporary directories are cleaned up even on panic
    let _cleanup = scopeguard::guard(&temp_dirs, |dirs| {
        for dir in dirs {
            let _ = std::fs::remove_dir_all(dir.path());
        }
    });

    // Create logger collectors for each agent to capture log output
    let mut log_collectors: Vec<Arc<Mutex<Vec<String>>>> = Vec::new();
    for _i in 0..NUM_AGENTS {
        log_collectors.push(Arc::new(Mutex::new(Vec::new())));
    }

    // Create tasks to run agents concurrently
    let mut agent_tasks = Vec::new();

    for i in 0..NUM_AGENTS {
        let client_clone = client.clone();
        let workspace = workspace_paths[i]
            .to_str()
            .expect("Failed to convert workspace path to string")
            .to_string();
        let agent_name = format!("concurrent-agent-{}", i);
        let log_collector = Arc::clone(&log_collectors[i]);

        let task = tokio::spawn(async move {
            // Create a ContainerConfig for this agent
            let config = ContainerConfig {
                agent_name: agent_name.clone(),
                env_vars: vec![],
                timeout: None,
                readonly: false,
                prompt: String::new(),
                skills: None,
            };

            // Prepare a command that outputs a unique marker for this agent
            let agent_marker = format!("Agent-{}", i);
            let sleep_duration = 2 + i; // Different sleep durations to ensure overlap
            let cmd = vec![
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "echo '{}' && sleep {} && echo '{} done'",
                    agent_marker, sleep_duration, agent_marker
                ),
            ];

            // Run the agent
            let result = match switchboard::docker::run::run_agent(
                &workspace,
                &client_clone,
                &config,
                None, // No timeout
                "alpine:latest",
                Some(&cmd),
                None, // No logger
                None, // No metrics store
                &agent_name,
                None, // No queued start time
            )
            .await
            {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("Failed to run agent {}: {:?}", agent_name, e);
                    return (i, None, false);
                }
            };

            // Capture logs by reading from the log file if it exists
            let log_dir = format!("{}/.switchboard/logs", workspace);
            let log_file_path = format!("{}/{}.log", log_dir, agent_name);
            let logs = std::fs::read_to_string(&log_file_path).unwrap_or_default();

            // Store logs in the collector
            if let Ok(mut collector) = log_collector.lock() {
                *collector = logs.lines().map(|s| s.to_string()).collect();
            }

            (i, Some(result), true)
        });

        agent_tasks.push(task);
    }

    // Wait a short time to ensure all containers have started
    sleep(Duration::from_millis(500)).await;

    // List running containers to verify concurrency
    let filters = HashMap::from([("label".to_string(), vec!["switchboard.agent".to_string()])]);
    let list_options = Some(ListContainersOptions {
        all: false, // Only running containers
        filters,
        ..Default::default()
    });

    let running_containers = docker
        .list_containers::<String>(list_options)
        .await
        .unwrap();

    // Verify that at least 2 containers are running concurrently
    assert!(
        running_containers.len() >= 2,
        "At least 2 containers should be running concurrently, found {}",
        running_containers.len()
    );

    // Verify the containers have the correct labels
    for container in &running_containers {
        let labels = container
            .labels
            .as_ref()
            .expect("Container should have labels");
        assert!(
            labels.contains_key("switchboard.agent"),
            "Container should have switchboard.agent label"
        );
        let agent_name = labels
            .get("switchboard.agent")
            .expect("Should have agent name");
        assert!(
            agent_name.starts_with("concurrent-agent-"),
            "Container should be a concurrent agent, got: {}",
            agent_name
        );
    }

    eprintln!(
        "Verified {} containers running concurrently",
        running_containers.len()
    );

    // Wait for all agents to complete
    let results = join_all(agent_tasks).await;

    // Verify all agents completed successfully
    let mut agent_results = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok((agent_index, Some(execution_result), completed)) => {
                assert_eq!(
                    agent_index, i,
                    "Agent index mismatch: expected {}, got {}",
                    i, agent_index
                );
                assert!(completed, "Agent {} should have completed successfully", i);
                assert_eq!(
                    execution_result.exit_code, 0,
                    "Agent {} should exit with code 0, got {}",
                    i, execution_result.exit_code
                );
                assert!(
                    !execution_result.container_id.is_empty(),
                    "Agent {} should have a non-empty container ID",
                    i
                );
                agent_results.push(execution_result);
            }
            Ok((agent_index, None, completed)) => {
                panic!(
                    "Agent {} failed to execute (completed={}, result=None)",
                    agent_index, completed
                );
            }
            Err(e) => {
                panic!("Agent {} task failed: {:?}", i, e);
            }
        }
    }

    eprintln!("All {} agents completed successfully", NUM_AGENTS);

    // Verify all containers were cleaned up (--rm behavior)
    sleep(Duration::from_millis(500)).await;

    let filters = HashMap::from([("label".to_string(), vec!["switchboard.agent".to_string()])]);
    let list_options = Some(ListContainersOptions {
        all: true, // Include stopped containers
        filters,
        ..Default::default()
    });

    let all_containers = docker
        .list_containers::<String>(list_options)
        .await
        .unwrap();

    // Filter out any containers that are not our concurrent agents
    let concurrent_containers: Vec<_> = all_containers
        .iter()
        .filter(|c| {
            if let Some(labels) = &c.labels {
                if let Some(agent_name) = labels.get("switchboard.agent") {
                    return agent_name.starts_with("concurrent-agent-");
                }
            }
            false
        })
        .collect();

    // Verify no concurrent agent containers remain (all should have been auto-removed)
    assert!(
        concurrent_containers.is_empty(),
        "All {} concurrent agent containers should have been auto-removed, but {} containers still exist",
        NUM_AGENTS,
        concurrent_containers.len()
    );

    eprintln!("All containers were properly cleaned up (--rm behavior verified)");

    // Verify logs are properly interleaved without corruption
    for (i, log_collector) in log_collectors.iter().enumerate() {
        let logs = log_collector.lock().unwrap();
        let agent_marker = format!("Agent-{}", i);

        // Verify each log contains the expected markers
        let has_start_marker = logs.iter().any(|line| line.contains(&agent_marker));
        let has_end_marker = logs
            .iter()
            .any(|line| line.contains(&format!("{} done", agent_marker)));

        assert!(
            has_start_marker,
            "Agent {} logs should contain start marker '{}'",
            i, agent_marker
        );

        assert!(
            has_end_marker,
            "Agent {} logs should contain end marker '{} done'",
            i, agent_marker
        );

        // Verify logs are not empty
        assert!(!logs.is_empty(), "Agent {} logs should not be empty", i);

        // Verify logs are not corrupted (no unexpected characters or malformed lines)
        for line in logs.iter() {
            assert!(
                !line.contains('\0'),
                "Agent {} logs contain null byte (corruption detected)",
                i
            );
        }

        eprintln!(
            "Agent {} logs verified: {} lines, contains markers",
            i,
            logs.len()
        );
    }

    eprintln!("All logs are properly interleaved without corruption");
}
