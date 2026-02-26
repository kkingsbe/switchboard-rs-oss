//! Git executor module
//!
//! Provides a wrapper around the ProcessExecutorTrait for executing git commands.
//! This enables dependency injection for testability.

use std::sync::Arc;

use crate::architect::{ArchitectError, Result};
use crate::traits::ProcessExecutorTrait;

/// GitExecutor wraps a process executor to provide git-specific operations.
///
/// This struct allows for dependency injection of the process executor,
/// making it easy to test git operations without actually running git commands.
pub struct GitExecutor {
    executor: Arc<dyn ProcessExecutorTrait>,
}

impl GitExecutor {
    /// Creates a new GitExecutor with the given process executor.
    ///
    /// # Arguments
    ///
    /// * `executor` - A thread-safe, async process executor
    ///
    /// # Returns
    ///
    /// A new GitExecutor instance
    pub fn new(executor: Arc<dyn ProcessExecutorTrait>) -> Self {
        Self { executor }
    }

    /// Commits the architect state files to git.
    ///
    /// This method runs `git add` followed by `git commit` with the provided message.
    ///
    /// # Arguments
    ///
    /// * `files` - A slice of file paths to add and commit
    /// * `message` - The commit message
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The `git add` command fails
    /// - The `git commit` command fails
    /// - Git is not installed or not in the PATH
    /// - The repository is not initialized (not a git repository)
    pub async fn commit(&self, files: &[&str], message: &str) -> Result<()> {
        // Add all relevant files
        let add_args: Vec<&str> = std::iter::once("add")
            .chain(files.iter().copied())
            .collect::<Vec<_>>();
        
        let add_output = self
            .executor
            .execute("git", &add_args)
            .await
            .map_err(|e| ArchitectError::Git(format!("git add failed: {e}")))?;

        if !add_output.status.success() {
            let stderr = String::from_utf8_lossy(&add_output.stderr);
            return Err(ArchitectError::Git(format!("git add failed: {stderr}")));
        }

        // Commit with the provided message
        let commit_output = self
            .executor
            .execute("git", &["commit", "-m", message])
            .await
            .map_err(|e| ArchitectError::Git(format!("git commit failed: {e}")))?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            return Err(ArchitectError::Git(format!("git commit failed: {stderr}")));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{ProcessError, ProcessOutput};
    use std::process::ExitStatus;
    use std::sync::Mutex;

    /// Mock executor for testing
    struct MockExecutor {
        calls: Mutex<Vec<(String, Vec<String>)>>,
        should_fail: bool,
    }

    impl MockExecutor {
        fn new(should_fail: bool) -> Self {
            Self {
                calls: Mutex::new(Vec::new()),
                should_fail,
            }
        }
    }

    impl ProcessExecutorTrait for MockExecutor {
        async fn execute(
            &self,
            program: &str,
            args: &[&str],
        ) -> Result<ProcessOutput, ProcessError> {
            let args_owned: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            self.calls.lock().unwrap().push((program.to_string(), args_owned));

            if self.should_fail {
                Err(ProcessError::Execution(String::from("Mock failure")))
            } else {
                Ok(ProcessOutput {
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                    exit_status: ExitStatus::default(),
                })
            }
        }

        async fn execute_with_env(
            &self,
            _program: &str,
            _args: &[&str],
            _env: std::collections::HashMap<String, String>,
            _working_dir: Option<std::path::PathBuf>,
        ) -> Result<ProcessOutput, ProcessError> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_commit_success() {
        let executor = Arc::new(MockExecutor::new(false));
        let git_executor = GitExecutor::new(executor.clone());

        let result = git_executor
            .commit(&["file1", "file2"], "test commit")
            .await;

        assert!(result.is_ok());

        let calls = executor.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "git");
        assert!(calls[0].1.iter().any(|a| a == "add"));
        assert_eq!(calls[1].0, "git");
        assert!(calls[1].1.iter().any(|a| a == "commit"));
    }

    #[tokio::test]
    async fn test_commit_add_fails() {
        let executor = Arc::new(MockExecutor::new(true));
        let git_executor = GitExecutor::new(executor.clone());

        let result = git_executor
            .commit(&["file1"], "test commit")
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ArchitectError::Git(_)));
    }
}
