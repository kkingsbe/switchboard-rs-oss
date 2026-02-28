//! SkillsManager - manages skill operations by delegating to npx skills CLI

use crate::traits::{ProcessExecutorTrait, RealProcessExecutor};
use crate::skills::SkillsError;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Arc;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
#[cfg(windows)]
use std::os::windows::process::ExitStatusExt;

/// SkillsManager manages skill operations by delegating to npx skills CLI
pub struct SkillsManager {
    /// Project-level skills directory (typically ./skills/)
    pub skills_dir: PathBuf,
    /// Global skills directory (typically ./skills/)
    pub global_skills_dir: PathBuf,
    /// Whether npx is available on the host system
    pub npx_available: bool,
    /// Process executor for running npx commands
    pub executor: Arc<dyn ProcessExecutorTrait>,
}

impl std::fmt::Debug for SkillsManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillsManager")
            .field("skills_dir", &self.skills_dir)
            .field("global_skills_dir", &self.global_skills_dir)
            .field("npx_available", &self.npx_available)
            .field("executor", &"<dyn ProcessExecutorTrait>")
            .finish()
    }
}

impl Default for SkillsManager {
    fn default() -> Self {
        Self::new(None)
    }
}

impl SkillsManager {
    /// Creates a new SkillsManager with default paths
    pub fn new(executor: Option<Arc<dyn ProcessExecutorTrait>>) -> Self {
        let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));
        Self {
            skills_dir: PathBuf::from("./skills"),
            global_skills_dir: PathBuf::from("./skills"),
            npx_available: false,
            executor,
        }
    }

    /// Creates a new SkillsManager with the specified project skills directory
    pub fn with_skills_dir(
        skills_dir: PathBuf,
        executor: Option<Arc<dyn ProcessExecutorTrait>>,
    ) -> Self {
        let executor = executor.unwrap_or_else(|| Arc::new(RealProcessExecutor::new()));
        Self {
            skills_dir,
            global_skills_dir: PathBuf::from("./skills"),
            npx_available: false,
            executor,
        }
    }

    /// Gets the project skills directory
    pub fn skills_dir(&self) -> &PathBuf {
        &self.skills_dir
    }

    /// Gets the global skills directory
    pub fn global_skills_dir(&self) -> &PathBuf {
        &self.global_skills_dir
    }

    /// Checks if npx is available on the host system.
    ///
    /// This method attempts to execute `npx --version` to verify that npx
    /// is installed and accessible. The result is stored in the `npx_available`
    /// field for future reference.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if npx is available and successfully executed
    /// * `Err(SkillsError::NpxNotFound)` if npx is not installed or not found
    ///
    /// # Example
    ///
    /// ```rust
    /// use switchboard::skills::SkillsManager;
    /// use switchboard::skills::SkillsError;
    ///
    /// let mut manager = SkillsManager::new(None);
    /// match manager.check_npx_available() {
    ///     Ok(()) => println!("npx is available"),
    ///     Err(SkillsError::NpxNotFound) => eprintln!("npx is not available"),
    ///     _ => (),
    /// }
    /// ```
    pub fn check_npx_available(&mut self) -> Result<(), SkillsError> {
        // On Windows, npx is a .cmd batch file that needs cmd /c to execute properly
        // Use create_npx_command() which handles this correctly on all platforms
        let mut cmd = create_npx_command();
        cmd.arg("--version");

        let result = cmd.output();

        match result {
            Ok(output) => {
                // npx command executed; check if it returned success
                if output.status.success() {
                    self.npx_available = true;
                    Ok(())
                } else {
                    // npx exists but returned non-zero - likely not properly installed
                    self.npx_available = false;
                    Err(SkillsError::NpxNotFound)
                }
            }
            Err(_) => {
                // npx command failed to execute - not installed or not in PATH
                self.npx_available = false;
                Err(SkillsError::NpxNotFound)
            }
        }
    }
}

/// Creates a new Command for executing npx.
///
/// On Windows, this uses cmd /c to properly find and execute .cmd batch files.
/// On other platforms, it directly invokes npx.
#[cfg(target_os = "windows")]
pub fn create_npx_command() -> Command {
    let mut cmd = Command::new("cmd");
    cmd.args(["/c", "npx"]);
    cmd
}

#[cfg(not(target_os = "windows"))]
pub fn create_npx_command() -> Command {
    Command::new("npx")
}

/// Runs an npx command with the specified arguments and returns the output.
///
/// This function builds and executes an npx process with the given command,
/// arguments, and optional working directory. It captures both stdout and stderr
/// and returns them in the [`Output`] struct along with the exit status.
///
/// # Arguments
///
/// * `command` - The command to run via npx (e.g., "skills")
/// * `args` - Arguments to pass to the command
/// * `working_dir` - Optional working directory for the command
/// * `executor` - Optional process executor. If None, uses direct Command execution.
///
/// # Returns
///
/// * `Ok(Output)` - The process output including stdout, stderr, and exit status
/// * `Err(SkillsError::NpxNotFound)` - If npx is not available on the system
/// * `Err(SkillsError::IoError)` - If there's an IO error executing the command
///
/// # Example
///
/// ```rust,ignore
/// use switchboard::skills::run_npx_command;
///
/// // Note: This function is async and must be awaited
/// // let output = run_npx_command("skills", &["list", "--json"], None, None).await?;
/// // println!("Output: {}", String::from_utf8_lossy(&output.stdout));
/// # Ok::<(), switchboard::skills::SkillsError>(())
/// ```
pub async fn run_npx_command(
    command: &str,
    args: &[&str],
    working_dir: Option<&PathBuf>,
    executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<Output, SkillsError> {
    // Build command arguments - convert &str to String
    let args_strings: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut full_args: Vec<String> = vec![command.to_string()];
    full_args.extend(args_strings);

    if let Some(exec) = executor {
        // Use the provided executor
        let process_output = exec
            .execute("npx", &full_args)
            .map_err(|e| SkillsError::IoError {
                operation: "execute npx command".to_string(),
                path: "npx".to_string(),
                message: e.to_string(),
            })?;
        // Convert ProcessOutput to Output
        // Convert String to Vec<u8> and traits::ExitStatus to std::process::ExitStatus
        let stdout: Vec<u8> = process_output.stdout.into_bytes();
        let stderr: Vec<u8> = process_output.stderr.into_bytes();
        let exit_code = process_output.status.code().unwrap_or(1);
        #[cfg(unix)]
        let status = std::process::ExitStatus::from_raw(exit_code);
        #[cfg(windows)]
        let status = std::process::ExitStatus::from_raw(exit_code as u32);
        return Ok(Output {
            stdout,
            stderr,
            status,
        });
    }

    // Fall back to direct Command execution for backward compatibility
    let mut cmd = Command::new("npx");
    cmd.args(&full_args);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| SkillsError::IoError {
        operation: "execute npx command".to_string(),
        path: command.to_string(),
        message: e.to_string(),
    })?;

    Ok(output)
}

/// Run `npx skills update` command, optionally with a specific skill name.
///
/// # Arguments
///
/// * `skill_name` - Optional skill name to update. If None, updates all installed skills.
/// * `executor` - Optional process executor. If None, uses direct Command execution.
///
/// # Returns
///
/// * `Result<ExitStatus, SkillsError>` - The exit status of the npx command
///
/// # Errors
///
/// Returns an error if the npx command fails to execute.
///
/// Note: This function uses stdin/stdout/stderr inheritance for interactive output.
/// ProcessExecutorTrait doesn't support this, so we use Command directly.
pub async fn run_npx_skills_update(
    skill_name: Option<&str>,
    _executor: Option<Arc<dyn ProcessExecutorTrait>>,
) -> Result<std::process::ExitStatus, SkillsError> {
    use std::process::Stdio;

    // Build command arguments
    let mut args = vec!["skills", "update"];
    if let Some(name) = skill_name {
        args.push(name);
    }

    // Execute npx command using Command directly because ProcessExecutorTrait
    // doesn't support stdin/stdout/stderr inheritance which is needed for interactive output
    let status = Command::new("npx")
        .args(&args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| SkillsError::NpxCommandFailed {
            command: "npx skills update".to_string(),
            exit_code: e.raw_os_error().unwrap_or(-1),
            stderr: e.to_string(),
        })?;

    Ok(status)
}
