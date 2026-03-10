use clap::{Args, Parser, Subcommand};

/// Exit code for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Error,
}

/// Main command struct for project management
#[derive(Parser, Debug)]
#[clap(name = "project", about = "Manage Switchboard projects")]
pub struct ProjectCommand {
    #[clap(subcommand)]
    pub subcommand: ProjectSubcommand,
}

/// Subcommand variants for project management
#[derive(Subcommand, Debug)]
pub enum ProjectSubcommand {
    /// Initialize a new Switchboard project
    Init(ProjectInit),
}

/// Command to initialize a new Switchboard project
///
/// Creates a new Switchboard project with the specified directory structure.
/// By default, creates prompts, skills, and workflows directories unless
/// the --minimal flag is used.
///
/// # Examples
///
/// Initialize in current directory:
/// ```text
/// switchboard project init
/// ```
///
/// Initialize in a new directory:
/// ```text
/// switchboard project init --path ./my-project
/// ```
///
/// Create with custom name:
/// ```text
/// switchboard project init --name my-project
/// ```
///
/// Create minimal structure without prompts:
/// ```text
/// switchboard project init --minimal
/// ```
#[derive(Args, Debug)]
pub struct ProjectInit {
    /// Path where the project will be created
    ///
    /// Directory where the Switchboard project structure will be created.
    /// Defaults to the current directory.
    #[arg(short, long, default_value = ".")]
    pub path: String,

    /// Name of the project
    ///
    /// Optional name for the project. If not provided, uses the directory name.
    #[arg(short, long)]
    pub name: Option<String>,

    /// Overwrite existing files
    ///
    /// If set, allows overwriting existing files in the target directory.
    /// Without this flag, the command will fail if files already exist.
    #[arg(short, long, action)]
    pub force: bool,

    /// Create minimal project structure
    ///
    /// If set, creates a minimal project structure without prompts, skills,
    /// and workflows directories. Useful for quick testing or minimal setups.
    #[arg(short, long, action)]
    pub minimal: bool,
}
