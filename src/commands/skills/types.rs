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
#[clap(name = "skills", about = "Manage Kilo skills")]
pub struct SkillsCommand {
    #[clap(subcommand)]
    pub subcommand: SkillsSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SkillsSubcommand {
    /// List available skills from the registry
    ///
    /// List skills from the skills.sh registry. You can filter results
    /// using the --search flag to find specific skills.
    ///
    /// # Examples
    ///
    /// List all available skills:
    /// ```text
    /// switchboard skills list
    /// ```
    ///
    /// Search for specific skills:
    /// ```text
    /// switchboard skills list --search docker
    /// switchboard skills list --search "file operations"
    /// ```
    List(SkillsList),

    /// Install a skill from a source
    ///
    /// Install a skill from a GitHub repository, npm package, or local path.
    /// Skills are installed by default to the project-level skills directory.
    ///
    /// # Examples
    ///
    /// Install a skill from GitHub:
    /// ```text
    /// switchboard skills install owner/repo
    /// ```
    ///
    /// Install a specific skill from a repo:
    /// ```text
    /// switchboard skills install owner/repo@skill-name
    /// ```
    ///
    /// Install globally (available to all projects):
    /// ```text
    /// switchboard skills install --global owner/repo
    /// ```
    Install(SkillsInstall),

    /// List installed skills
    ///
    /// List all currently installed skills in both project and global scopes.
    /// Shows skill name, description, version, source, and which agents use each skill.
    ///
    /// # Examples
    ///
    /// List all installed skills:
    /// ```text
    /// switchboard skills installed
    /// ```
    ///
    /// List only global skills:
    /// ```text
    /// switchboard skills installed --global
    /// ```
    Installed(SkillsInstalled),

    /// Update installed skills to their latest versions
    ///
    /// If a specific skill name is provided, only that skill is updated.
    /// If no skill name is provided, all installed skills are updated.
    ///
    /// # Examples
    ///
    /// Update all installed skills:
    /// ```text
    /// switchboard skills update
    /// ```
    ///
    /// Update a specific skill:
    /// ```text
    /// switchboard skills update frontend-design
    /// ```
    Update(SkillsUpdate),

    /// Remove an installed skill
    ///
    /// Removes a skill from either the project or global skills directory.
    /// Shows a warning if the skill is still referenced by agents in the configuration.
    /// Requires confirmation unless the --yes flag is used.
    ///
    /// # Examples
    ///
    /// Remove a project skill with confirmation:
    /// ```text
    /// switchboard skills remove frontend-design
    /// ```
    ///
    /// Remove a global skill:
    /// ```text
    /// switchboard skills remove --global skill-creator
    /// ```
    ///
    /// Remove without confirmation:
    /// ```text
    /// switchboard skills remove --yes frontend-design
    /// ```
    Remove(SkillsRemove),
}

/// Command to list available skills
#[derive(Args, Debug)]
pub struct SkillsList {
    /// Search query (positional argument)
    ///
    /// Search terms to filter the skills list. This filters by name,
    /// description, and other metadata from the skills.sh registry.
    /// Minimum 2 characters required.
    /// Can also be specified with --search flag.
    #[arg(default_value = "ai")]
    pub query: Option<String>,

    /// Filter skills by query string (alternative to positional argument)
    #[arg(short, long, help = "Filter skills by query string")]
    pub search: Option<String>,

    /// Maximum number of results to return
    ///
    /// Limits the number of skills returned from the skills.sh API.
    /// Default is 10 per requirements.
    #[arg(long, help = "Maximum number of results to return", value_parser = clap::value_parser!(u32))]
    pub limit: Option<u32>,
}

/// Command to install a skill
#[derive(Parser, Debug)]
pub struct SkillsInstall {
    /// Skill source (e.g., npm package name, GitHub URL, or local path)
    ///
    /// The source can be:
    /// - GitHub repository: `owner/repo` or `owner/repo@skill-name`
    /// - Full GitHub URL: `https://github.com/owner/repo`
    /// - GitLab URL: `https://gitlab.com/owner/repo`
    /// - npm package name
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Install globally instead of project-local
    ///
    /// When set, installs the skill to the global skills directory
    /// (./skills/) instead of the project-level directory
    /// (./skills/). Global skills are available to all projects.
    #[arg(long)]
    pub global: bool,

    /// Skip confirmation prompt when destination exists
    ///
    /// When set, bypasses the confirmation prompt and overwrites
    /// the skill if it already exists. Use with caution.
    #[arg(
        long,
        help = "Skip confirmation prompt and overwrite if destination exists"
    )]
    pub yes: bool,
}

/// Command to list installed skills
///
/// Lists all currently installed skills in both project and global scopes.
/// Shows skill name, description, version, source, and which agents use each skill.
///
/// # Examples
///
/// List all installed skills:
/// ```text
/// switchboard skills installed
/// ```
///
/// List only global skills:
/// ```text
/// switchboard skills installed --global
/// ```
#[derive(Parser, Debug)]
pub struct SkillsInstalled {
    /// Show only global skills
    ///
    /// When set, only shows skills from the global skills directory
    /// (./skills/). Project-level skills from ./skills/
    /// are not displayed.
    #[arg(long, help = "Show only global skills")]
    pub global: bool,
}

/// Command to remove an installed skill
///
/// Removes a skill from either the project or global skills directory.
/// Shows a warning if the skill is still referenced by agents in the configuration.
/// Requires confirmation unless the --yes flag is used.
///
/// # Examples
///
/// Remove a project skill with confirmation:
/// ```text
/// switchboard skills remove frontend-design
/// ```
///
/// Remove a global skill:
/// ```text
/// switchboard skills remove --global skill-creator
/// ```
///
/// Remove without confirmation:
/// ```text
/// switchboard skills remove --yes frontend-design
/// ```
#[derive(Parser, Debug)]
pub struct SkillsRemove {
    /// Name of the skill to remove
    ///
    /// The name of the skill directory to remove from the skills directory.
    #[arg(value_name = "SKILL_NAME")]
    pub skill_name: String,

    /// Remove from global skills directory
    ///
    /// When set, removes the skill from the global skills directory
    /// (./skills/) instead of the project-level directory
    /// (./skills/).
    #[arg(long, help = "Remove from global skills directory")]
    pub global: bool,

    /// Skip confirmation prompt
    ///
    /// When set, bypasses the confirmation prompt and removes the skill
    /// immediately. Use with caution.
    #[arg(long, help = "Skip confirmation prompt")]
    pub yes: bool,
}

/// Update installed skills to their latest versions.
///
/// If a specific skill name is provided, only that skill is updated.
/// If no skill name is provided, all installed skills are updated.
#[derive(Parser, Debug)]
pub struct SkillsUpdate {
    /// Optional skill name to update. If omitted, updates all installed skills.
    #[arg(value_name = "skill-name", last = true)]
    pub skill_name: Option<String>,
}
