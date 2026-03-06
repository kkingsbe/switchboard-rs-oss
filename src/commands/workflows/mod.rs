pub mod apply;
pub mod install;
pub mod installed;
pub mod list;
pub mod remove;
pub mod skills;
pub mod types;
pub mod update;
pub mod validate;

pub use types::*;

use crate::config::Config;

/// Run the workflows command based on the provided subcommand.
///
/// This function serves as the main entry point for all workflows-related operations.
/// It dispatches the execution to the appropriate handler function based on the
/// subcommand specified in the [`WorkflowsCommand`].
///
/// # Supported Subcommands
///
/// - `list` - List available workflows from the registry
/// - `install` - Install a workflow from the registry
/// - `installed` - List currently installed workflows
/// - `update` - Update installed workflows to their latest versions
/// - `remove` - Remove an installed workflow
///
/// # Parameters
///
/// * `args` - The [`WorkflowsCommand`] containing the subcommand and its arguments
/// * `config` - Reference to the application configuration
///
/// # Returns
///
/// Returns an [`ExitCode`] indicating success or failure:
/// - [`ExitCode::Success`] - The command executed successfully
/// - [`ExitCode::Error`] - The command execution failed
///
/// # Examples
///
/// Listing available workflows:
/// ```text
/// switchboard workflows list
/// ```
///
/// Installing a workflow:
/// ```text
/// switchboard workflows install workflow-name
/// ```
///
/// Listing installed workflows:
/// ```text
/// switchboard workflows installed
/// ```
pub async fn run_workflows(args: WorkflowsCommand, config: &Config) -> ExitCode {
    match args.subcommand {
        WorkflowsSubcommand::List(list_args) => list::run_workflows_list(list_args, config).await,
        WorkflowsSubcommand::Install(install_args) => {
            install::run_workflows_install(install_args, config).await
        }
        WorkflowsSubcommand::Installed(installed_args) => {
            installed::run_workflows_installed(installed_args, config).await
        }
        WorkflowsSubcommand::Update(update_args) => {
            update::handle_workflows_update(update_args, config).await
        }
        WorkflowsSubcommand::Remove(remove_args) => {
            remove::run_workflows_remove(remove_args, config).await
        }
        WorkflowsSubcommand::Validate(validate_args) => {
            validate::run_workflows_validate(validate_args, config).await
        }
        WorkflowsSubcommand::Apply(apply_args) => {
            apply::run_workflows_apply(apply_args, config).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_workflows_command_parsing() {
        // Test that the main command parses correctly
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "list"]).unwrap();
        assert!(matches!(cmd.subcommand, WorkflowsSubcommand::List(_)));
    }

    #[test]
    fn test_workflows_install_parsing() {
        let cmd =
            WorkflowsCommand::try_parse_from(["workflows", "install", "test-workflow"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::Install(args) => {
                assert_eq!(args.workflow_name, "test-workflow");
                assert!(!args.yes);
            }
            _ => panic!("Expected Install subcommand"),
        }
    }

    #[test]
    fn test_workflows_install_with_yes_flag() {
        let cmd = WorkflowsCommand::try_parse_from([
            "workflows",
            "install",
            "--yes",
            "test-workflow",
        ])
        .unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::Install(args) => {
                assert_eq!(args.workflow_name, "test-workflow");
                assert!(args.yes);
            }
            _ => panic!("Expected Install subcommand"),
        }
    }

    #[test]
    fn test_workflows_remove_parsing() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "remove", "test-workflow"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::Remove(args) => {
                assert_eq!(args.workflow_name, "test-workflow");
                assert!(!args.yes);
            }
            _ => panic!("Expected Remove subcommand"),
        }
    }

    #[test]
    fn test_workflows_remove_with_yes_flag() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "remove", "--yes", "test-workflow"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::Remove(args) => {
                assert_eq!(args.workflow_name, "test-workflow");
                assert!(args.yes);
            }
            _ => panic!("Expected Remove subcommand"),
        }
    }

    #[test]
    fn test_workflows_update_no_args() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "update"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::Update(args) => {
                assert!(args.workflow_name.is_none());
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn test_workflows_update_with_workflow_name() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "update", "test-workflow"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::Update(args) => {
                assert_eq!(args.workflow_name, Some("test-workflow".to_string()));
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn test_workflows_list_parsing() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "list"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::List(args) => {
                assert!(args.search.is_none());
                assert!(args.limit.is_none());
            }
            _ => panic!("Expected List subcommand"),
        }
    }

    #[test]
    fn test_workflows_list_with_search() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "list", "--search", "docker"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::List(args) => {
                assert_eq!(args.search, Some("docker".to_string()));
            }
            _ => panic!("Expected List subcommand"),
        }
    }

    #[test]
    fn test_workflows_list_with_limit() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "list", "--limit", "20"]).unwrap();
        match cmd.subcommand {
            WorkflowsSubcommand::List(args) => {
                assert_eq!(args.limit, Some(20));
            }
            _ => panic!("Expected List subcommand"),
        }
    }

    #[test]
    fn test_workflows_installed_parsing() {
        let cmd = WorkflowsCommand::try_parse_from(["workflows", "installed"]).unwrap();
        assert!(matches!(cmd.subcommand, WorkflowsSubcommand::Installed(_)));
    }
}
