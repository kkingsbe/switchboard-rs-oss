use clap::{Args, Parser, Subcommand};

/// Exit code for command execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    /// Command executed successfully
    Success,
    /// Command execution failed
    Error,
}

impl ExitCode {
    /// Converts an i32 exit code to ExitCode
    pub fn from_i32(code: i32) -> Self {
        if code == 0 {
            ExitCode::Success
        } else {
            ExitCode::Error
        }
    }
}

#[derive(Parser, Debug)]
#[clap(name = "workflows", about = "Manage Switchboard workflows")]
pub struct WorkflowsCommand {
    #[clap(subcommand)]
    pub subcommand: WorkflowsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum WorkflowsSubcommand {
    /// List available workflows from the registry
    ///
    /// List workflows from the switchboard-workflows repository.
    /// You can filter results using the --search flag to find specific workflows.
    ///
    /// # Examples
    ///
    /// List all available workflows:
    /// ```text
    /// switchboard workflows list
    /// ```
    ///
    /// Search for specific workflows:
    /// ```text
    /// switchboard workflows list --search docker
    /// switchboard workflows list --search "file operations"
    /// ```
    List(WorkflowsList),

    /// Install a workflow from the registry
    ///
    /// Install a workflow from the switchboard-workflows registry.
    /// Workflows are always installed to the project-level workflows directory.
    ///
    /// # Examples
    ///
    /// Install a workflow:
    /// ```text
    /// switchboard workflows install workflow-name
    /// ```
    ///
    /// Install without confirmation:
    /// ```text
    /// switchboard workflows install --yes workflow-name
    /// ```
    Install(WorkflowsInstall),

    /// List installed workflows
    ///
    /// List all currently installed workflows in the project scope.
    /// Shows workflow name, description, version, and source.
    ///
    /// # Examples
    ///
    /// List all installed workflows:
    /// ```text
    /// switchboard workflows installed
    /// ```
    Installed(WorkflowsInstalled),

    /// Update installed workflows to their latest versions
    ///
    /// If a specific workflow name is provided, only that workflow is updated.
    /// If no workflow name is provided, all installed workflows are updated.
    ///
    /// # Examples
    ///
    /// Update all installed workflows:
    /// ```text
    /// switchboard workflows update
    /// ```
    ///
    /// Update a specific workflow:
    /// ```text
    /// switchboard workflows update workflow-name
    /// ```
    Update(WorkflowsUpdate),

    /// Remove an installed workflow
    ///
    /// Removes a workflow from the project workflows directory.
    /// Shows a warning if the workflow is still referenced by agents in the configuration.
    /// Requires confirmation unless the --yes flag is used.
    ///
    /// # Examples
    ///
    /// Remove a workflow with confirmation:
    /// ```text
    /// switchboard workflows remove workflow-name
    /// ```
    ///
    /// Remove without confirmation:
    /// ```text
    /// switchboard workflows remove --yes workflow-name
    /// ```
    Remove(WorkflowsRemove),

    /// Validate a workflow's manifest.toml
    ///
    /// Validates that the manifest.toml file exists and is properly formatted.
    /// Checks that all referenced prompt files exist, cron schedules are valid,
    /// and overlap_mode values are valid.
    ///
    /// # Examples
    ///
    /// Validate a workflow's manifest:
    /// ```text
    /// switchboard workflows validate workflow-name
    /// ```
    Validate(WorkflowsValidate),

    /// Apply a workflow's manifest to generate switchboard.toml entries
    ///
    /// Generates switchboard.toml configuration entries from a workflow's manifest.toml.
    /// Creates agent configurations for each agent defined in the manifest, applying
    /// default values from the manifest where not specified.
    ///
    /// # Examples
    ///
    /// Apply a workflow to create switchboard.toml:
    /// ```text
    /// switchboard workflows apply bmad
    /// ```
    ///
    /// Append to existing config:
    /// ```text
    /// switchboard workflows apply bmad --append
    /// ```
    ///
    /// Preview what would be generated:
    /// ```text
    /// switchboard workflows apply bmad --dry-run
    /// ```
    Apply(WorkflowsApply),
}

/// Command to list available workflows
#[derive(Args, Debug)]
pub struct WorkflowsList {
    /// Filter workflows by search query
    ///
    /// Search terms to filter the workflows list. This filters by name,
    /// description, and other metadata from the switchboard-workflows registry.
    #[arg(short, long, help = "Filter workflows by search query")]
    pub search: Option<String>,

    /// Maximum number of results to return
    ///
    /// Limits the number of workflows returned from the GitHub API.
    /// Default is 10.
    #[arg(long, help = "Maximum number of results to return", value_parser = clap::value_parser!(u32))]
    pub limit: Option<u32>,
}

/// Command to install a workflow
#[derive(Parser, Debug)]
pub struct WorkflowsInstall {
    /// Name of the workflow to install
    ///
    /// The name of the workflow to install from the switchboard-workflows registry.
    /// Source is hardcoded to kkingsbe/switchboard-workflows.
    #[arg(value_name = "WORKFLOW_NAME")]
    pub workflow_name: String,

    /// Skip confirmation prompt
    ///
    /// When set, bypasses the confirmation prompt and installs the workflow
    /// immediately. Use with caution.
    #[arg(long, help = "Skip confirmation prompt")]
    pub yes: bool,
}

/// Command to list installed workflows
///
/// Lists all currently installed workflows in the project scope.
/// Shows workflow name, description, version, and source.
///
/// # Examples
///
/// List all installed workflows:
/// ```text
/// switchboard workflows installed
/// ```
#[derive(Parser, Debug)]
pub struct WorkflowsInstalled {
    // No extra arguments - workflows are always project-local
    // This struct is kept for consistency and potential future expansion
}

/// Command to remove an installed workflow
///
/// Removes a workflow from the project workflows directory.
/// Shows a warning if the workflow is still referenced by agents in the configuration.
/// Requires confirmation unless the --yes flag is used.
///
/// # Examples
///
/// Remove a workflow with confirmation:
/// ```text
/// switchboard workflows remove workflow-name
/// ```
///
/// Remove without confirmation:
/// ```text
/// switchboard workflows remove --yes workflow-name
/// ```
#[derive(Parser, Debug)]
pub struct WorkflowsRemove {
    /// Name of the workflow to remove
    ///
    /// The name of the workflow directory to remove from the workflows directory.
    #[arg(value_name = "WORKFLOW_NAME")]
    pub workflow_name: String,

    /// Skip confirmation prompt
    ///
    /// When set, bypasses the confirmation prompt and removes the workflow
    /// immediately. Use with caution.
    #[arg(long, help = "Skip confirmation prompt")]
    pub yes: bool,
}

/// Update installed workflows to their latest versions.
///
/// If a specific workflow name is provided, only that workflow is updated.
/// If no workflow name is provided, all installed workflows are updated.
#[derive(Parser, Debug)]
pub struct WorkflowsUpdate {
    /// Optional workflow name to update. If omitted, updates all installed workflows.
    #[arg(value_name = "workflow-name", last = true)]
    pub workflow_name: Option<String>,
}

/// Validate a workflow's manifest.toml
///
/// Validates that the manifest.toml file exists and is properly formatted.
/// Checks that all referenced prompt files exist, cron schedules are valid,
/// and overlap_mode values are valid.
///
/// # Examples
///
/// Validate a workflow's manifest:
/// ```text
/// switchboard workflows validate workflow-name
/// ```
#[derive(Parser, Debug)]
pub struct WorkflowsValidate {
    /// Name of the workflow to validate
    ///
    /// The name of the workflow directory in .switchboard/workflows/
    #[arg(value_name = "WORKFLOW_NAME")]
    pub workflow_name: String,
}

/// Apply a workflow's manifest to generate switchboard.toml entries
///
/// Generates switchboard.toml configuration entries from a workflow's manifest.toml.
/// Creates agent configurations for each agent defined in the manifest, applying
/// default values from the manifest where not specified.
///
/// # Examples
///
/// Apply a workflow to create switchboard.toml:
/// ```text
/// switchboard workflows apply bmad
/// ```
///
/// Append to existing config:
/// ```text
/// switchboard workflows apply bmad --append
/// ```
///
/// Preview what would be generated:
/// ```text
/// switchboard workflows apply bmad --dry-run
/// ```
#[derive(Parser, Debug)]
pub struct WorkflowsApply {
    /// Name of the workflow to apply
    ///
    /// The name of the workflow directory in .switchboard/workflows/
    #[arg(value_name = "WORKFLOW_NAME")]
    pub workflow_name: String,

    /// Agent name prefix
    ///
    /// Optional prefix to add to agent names. Defaults to the workflow name.
    /// Use this to avoid name conflicts with existing agents.
    #[arg(long, short, value_name = "PREFIX")]
    pub prefix: Option<String>,

    /// Output file path
    ///
    /// Path to write the generated configuration. Defaults to switchboard.toml.
    #[arg(long, short, value_name = "FILE")]
    pub output: Option<String>,

    /// Append to existing config
    ///
    /// If set, appends agents to an existing switchboard.toml instead of creating new.
    /// If not set, a new file is created (or overwritten with --output).
    #[arg(long, short = 'a')]
    pub append: bool,

    /// Skip confirmation prompt
    ///
    /// When set, bypasses the confirmation prompt and applies the workflow
    /// immediately. Use with caution.
    #[arg(long)]
    pub yes: bool,

    /// Preview changes without writing
    ///
    /// When set, shows what would be generated without actually writing the file.
    /// Useful for seeing the output before committing.
    #[arg(long)]
    pub dry_run: bool,
}
