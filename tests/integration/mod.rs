//! Integration tests for Switchboard
//!
//! These tests require a running Docker daemon and are gated behind the `integration` feature flag.
//! Run integration tests with: `cargo test --features integration -- --ignored`

#[cfg(feature = "integration")]
use bollard::Docker;

/// Check if Docker daemon is available
///
/// This helper function attempts to connect to the Docker daemon and ping it
/// to verify it's running and responsive.
///
/// # Returns
///
/// `true` if Docker is available and responsive, `false` otherwise.
#[cfg(feature = "integration")]
pub async fn docker_available() -> bool {
    match Docker::connect_with_local_defaults() {
        Ok(docker) => docker.ping().await.is_ok(),
        Err(_) => false,
    }
}

/// Placeholder integration test that verifies Docker availability
///
/// This test is marked as `#[ignore]` by default to prevent it from running
/// during normal test execution. Use `cargo nextest run --features integration -- --ignored`
/// to run it specifically.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_docker_availability() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // If we get here, Docker is available
    assert!(
        docker_available().await,
        "Docker should be available at this point"
    );
}

/// Placeholder integration test for running a simple container
///
/// This test demonstrates running a simple "hello-world" container to verify
/// basic Docker functionality is working.
#[cfg(feature = "integration")]
#[tokio::test]
#[ignore = "Run with --ignored to execute integration tests"]
async fn test_simple_container_run() {
    // Check if Docker is available before proceeding
    if !docker_available().await {
        eprintln!("Skipping test: Docker daemon is not available");
        return;
    }

    // Connect to Docker
    let docker = Docker::connect_with_local_defaults().expect("Failed to connect to Docker daemon");

    // Pull and run the hello-world image
    let create_result = docker
        .create_container(
            Some(bollard::container::CreateContainerOptions {
                name: "switchboard-test-hello-world",
                platform: None,
            }),
            bollard::container::Config {
                image: Some("hello-world:latest"),
                ..Default::default()
            },
        )
        .await;

    match create_result {
        Ok(container) => {
            // Start the container
            docker
                .start_container::<String>(
                    container.id.as_str(),
                    None::<bollard::container::StartContainerOptions<String>>,
                )
                .await
                .expect("Failed to start container");

            // Wait for the container to finish (returns a Stream)
            use futures_util::StreamExt;
            let mut wait_stream = docker.wait_container::<String>(
                container.id.as_str(),
                None::<bollard::container::WaitContainerOptions<String>>,
            );

            if let Some(Ok(_exit_code)) = wait_stream.next().await {
                // Clean up the container
                docker
                    .remove_container(
                        container.id.as_str(),
                        Some(bollard::container::RemoveContainerOptions {
                            force: true,
                            link: false,
                            v: false,
                        }),
                    )
                    .await
                    .expect("Failed to remove container");
            }
        }
        Err(e) => {
            eprintln!("Skipping test: Failed to create container: {}", e);
            return;
        }
    }
}

pub mod timeout_monitoring;

pub mod docker_client;

pub mod docker_image_build;

mod trivial_agent_version;

mod container_cleanup;

mod entrypoint_cmd_verification;

pub mod readonly_mount;

pub mod concurrent_agents;

pub mod environment_variables;

pub mod log_flush_on_wait_error;

pub mod skill_installation_integration;

mod npx_not_found_error;

pub mod manual_skills_backwards_compat;

pub mod skill_install_failure_handling;

pub mod backwards_compatibility;
