use clap::{Args, Parser, Subcommand};

/// Exit code for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Error,
}

/// Main command struct for workflow management
#[derive(Parser, Debug)]
#[clap(name = "workflow", about = "Create a new Switchboard workflow")]
pub struct WorkflowInitCommand {
    #[clap(subcommand)]
    pub subcommand: WorkflowInitSubcommand,
}

/// Subcommand variants for workflow initialization
#[derive(Subcommand, Debug)]
pub enum WorkflowInitSubcommand {
    /// Initialize a new Switchboard workflow
    Init(WorkflowInit),
}

/// Command to initialize a new Switchboard workflow
///
/// Creates a new Switchboard workflow with the specified agents and schedule.
/// Generates a workflow scaffold in the specified directory.
///
/// # Examples
///
/// Initialize in current directory:
/// ```text
/// switchboard workflow init --name my-workflow
/// ```
///
/// Initialize with agents:
/// ```text
/// switchboard workflow init --name my-workflow --agents agent1,agent2
/// ```
///
/// Initialize with cron schedule:
/// ```text
/// switchboard workflow init --name my-workflow --schedule "0 0 * * *"
/// ```
///
/// Initialize in a specific directory:
/// ```text
/// switchboard workflow init --name my-workflow --path ./workflows/my-workflow
/// ```
#[derive(Args, Debug)]
pub struct WorkflowInit {
    /// Name of the workflow
    ///
    /// The unique name identifier for this workflow.
    #[arg(short, long)]
    pub name: String,

    /// List of agent names for the workflow
    ///
    /// Comma-separated list of agent names to include in this workflow.
    /// Agents must be defined in the switchboard.toml configuration.
    #[arg(short, long)]
    pub agents: Option<String>,

    /// Cron schedule for the workflow
    ///
    /// Optional cron expression for scheduling the workflow.
    /// Example: "0 0 * * *" for daily at midnight.
    #[arg(short, long)]
    pub schedule: Option<String>,

    /// Path where the workflow will be created
    ///
    /// Directory where the workflow scaffold will be created.
    /// Defaults to the current directory.
    #[arg(short, long, default_value = ".")]
    pub path: String,
}
