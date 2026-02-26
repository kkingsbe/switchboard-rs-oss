//! Custom assertion macros for Switchboard tests.
//! 
//! Provides specialized assertion helpers for common test patterns
//! in the Switchboard CLI tool.

/// Asserts that a command execution succeeds (exit code 0).
#[macro_export]
macro_rules! assert_success {
    ($result:expr) => {
        let output = &$result;
        assert!(
            output.status.success(),
            "Expected command to succeed, but failed with: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    };
}

/// Asserts that a command execution fails (non-zero exit code).
#[macro_export]
macro_rules! assert_failure {
    ($result:expr) => {
        let output = &$result;
        assert!(
            !output.status.success(),
            "Expected command to fail, but it succeeded"
        );
    };
}

/// Asserts that output contains a specific substring.
#[macro_export]
macro_rules! assert_output_contains {
    ($output:expr, $needle:expr) => {
        let output_str = String::from_utf8_lossy($output);
        assert!(
            output_str.contains($needle),
            "Expected output to contain '{}', but got: {}",
            $needle,
            output_str
        );
    };
}

/// Asserts that output does NOT contain a specific substring.
#[macro_export]
macro_rules! assert_output_not_contains {
    ($output:expr, $needle:expr) => {
        let output_str = String::from_utf8_lossy($output);
        assert!(
            !output_str.contains($needle),
            "Expected output NOT to contain '{}', but got: {}",
            $needle,
            output_str
        );
    };
}

/// Helper to check if docker is available in the test environment.
#[cfg(feature = "docker")]
pub mod docker {
    use std::process::Command;

    /// Checks if docker is available and running.
    pub fn is_docker_available() -> bool {
        Command::new("docker")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Skips test if docker is not available.
    #[macro_export]
    macro_rules! skip_if_no_docker {
        () => {
            if !docker::is_docker_available() {
                println!("SKIPPED: Docker not available");
                return;
            }
        };
    }
}

/// Helper for path-related assertions.
pub mod path {
    use std::path::Path;

    /// Asserts that a path exists.
    pub fn assert_path_exists(path: &Path) {
        assert!(
            path.exists(),
            "Expected path '{}' to exist",
            path.display()
        );
    }

    /// Asserts that a path does not exist.
    pub fn assert_path_not_exists(path: &Path) {
        assert!(
            !path.exists(),
            "Expected path '{}' to not exist",
            path.display()
        );
    }

    /// Asserts that a path is a directory.
    pub fn assert_is_dir(path: &Path) {
        assert!(
            path.is_dir(),
            "Expected '{}' to be a directory",
            path.display()
        );
    }

    /// Asserts that a path is a file.
    pub fn assert_is_file(path: &Path) {
        assert!(
            path.is_file(),
            "Expected '{}' to be a file",
            path.display()
        );
    }
}
