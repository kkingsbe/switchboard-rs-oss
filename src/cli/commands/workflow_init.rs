//! Handler for the 'workflow' command
//!
//! This command initializes new Switchboard workflows with a scaffolded structure.

use crate::commands::workflow_init::{ExitCode, WorkflowInitCommand, WorkflowInitSubcommand};

/// Handler for the 'workflow' command
pub async fn run_workflow_init(
    args: WorkflowInitCommand,
    _config_path: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    match args.subcommand {
        WorkflowInitSubcommand::Init(init_args) => {
            // Workflow init may not need a config file, so we handle it directly
            let exit_code = crate::commands::workflow_init::run_workflow_init(init_args).await;
            match exit_code {
                ExitCode::Success => Ok(()),
                ExitCode::Error => Err("Workflow init failed".into()),
            }
        }
    }
}
