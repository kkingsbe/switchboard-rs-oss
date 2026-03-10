//! Handler for the 'project' command
//!
//! This command manages Switchboard projects - initializing new projects.

use crate::commands::project::{ProjectCommand, ProjectSubcommand};

/// Handler for the 'project' command
pub async fn run_project(
    args: ProjectCommand,
    _config_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        ProjectSubcommand::Init(init_args) => {
            // Project init may not need a config file, so we handle it directly
            let exit_code = crate::commands::project::run_project_init(init_args).await;
            match exit_code {
                crate::commands::project::ExitCode::Success => Ok(()),
                crate::commands::project::ExitCode::Error => {
                    Err("Project init failed".into())
                }
            }
        }
    }
}
