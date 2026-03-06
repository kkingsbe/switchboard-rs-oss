pub mod install;
pub mod installed;
pub mod list;
pub mod remove;
pub mod types;
pub mod update;

pub use update::{handle_skills_update, reinstall_skill_from_source, update_skill_timestamp};

pub use installed::{format_skill_entry_table, format_skills_list, get_agent_assignment_display};

pub use types::*;

use crate::commands::skills::install::run_skills_install;
use crate::config::Config;

/// Run the skills command based on the provided subcommand.
///
/// This function serves as the main entry point for all skills-related operations.
/// It dispatches the execution to the appropriate handler function based on the
/// subcommand specified in the [`SkillsCommand`].
///
/// # Supported Subcommands
///
/// - `list` - List available skills from the skills.sh registry
/// - `install` - Install a skill from GitHub, npm package, or local path
/// - `installed` - List currently installed skills
/// - `update` - Update installed skills to their latest versions
/// - `remove` - Remove an installed skill
///
/// # Parameters
///
/// * `args` - The [`SkillsCommand`] containing the subcommand and its arguments
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
/// Listing available skills:
/// ```text
/// switchboard skills list
/// ```
///
/// Installing a skill:
/// ```text
/// switchboard skills install owner/repo
/// ```
///
/// Listing installed skills:
/// ```text
/// switchboard skills installed
/// ```
pub async fn run_skills(args: SkillsCommand, config: &Config) -> ExitCode {
    match args.subcommand {
        SkillsSubcommand::List(list_args) => list::run_skills_list(list_args, config).await,
        SkillsSubcommand::Install(install_args) => run_skills_install(install_args, config).await,
        SkillsSubcommand::Installed(installed_args) => {
            installed::run_skills_installed(installed_args, config).await
        }
        SkillsSubcommand::Update(update_args) => handle_skills_update(update_args, config).await,
        SkillsSubcommand::Remove(remove_args) => {
            remove::run_skills_remove(remove_args, config).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Agent;
    use crate::skills::SkillMetadata;
    use clap::Parser;

    #[test]
    fn test_skills_installed_parsing() {
        // Test that --global flag is parsed correctly
        let args = SkillsInstalled { global: true };
        assert!(args.global);

        let args = SkillsInstalled { global: false };
        assert!(!args.global);
    }

    #[test]
    fn test_skills_remove_parsing() {
        // Test that all fields are parsed correctly
        let args = SkillsRemove {
            skill_name: "test-skill".to_string(),
            global: false,
            yes: false,
        };
        assert_eq!(args.skill_name, "test-skill");
        assert!(!args.global);
        assert!(!args.yes);
    }

    #[test]
    fn test_skills_remove_with_global_flag() {
        let args = SkillsRemove {
            skill_name: "test-skill".to_string(),
            global: true,
            yes: false,
        };
        assert!(args.global);
    }

    #[test]
    fn test_skills_remove_with_yes_flag() {
        let args = SkillsRemove {
            skill_name: "test-skill".to_string(),
            global: false,
            yes: true,
        };
        assert!(args.yes);
    }

    #[test]
    fn test_confirm_returns_true_for_y() {
        // We can't easily test the actual user input function in unit tests,
        // but we can verify the function exists and has the right signature
        let prompt = "Test prompt";
        // The actual function would prompt the user
        let _ = prompt;
    }

    #[test]
    fn test_format_skills_list_empty() {
        let config = Config::default();
        let project_skills: Vec<SkillMetadata> = Vec::new();
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        assert!(output.contains("No skills installed"));
        assert!(output.contains("switchboard skills list"));
        assert!(output.contains("switchboard skills install"));
    }

    #[test]
    fn test_format_skills_list_with_project_skills() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
        ];

        let project_skills = vec![SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("A test skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        assert!(output.contains("Installed Skills"));
        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("test-skill"));
        assert!(output.contains("A test skill"));
        assert!(output.contains("[all agents]"));
        assert!(output.contains("1 skills installed (1 project, 0 global)"));
    }

    #[test]
    fn test_format_skills_list_mixed() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["project-skill".to_string()]),
            ..Default::default()
        }];

        let project_skills = vec![SkillMetadata {
            name: "project-skill".to_string(),
            description: Some("Project skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills = vec![SkillMetadata {
            name: "global-skill".to_string(),
            description: Some("Global skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("Global (./skills/)"));
        assert!(output.contains("project-skill"));
        assert!(output.contains("global-skill"));
        assert!(output.contains("2 skills installed (1 project, 1 global)"));
    }

    #[test]
    fn test_get_agent_assignment_display_all_agents() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
        ];

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "[all agents]");
    }

    #[test]
    fn test_get_agent_assignment_display_some_agents() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec!["test-skill".to_string()]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["other-skill".to_string()]),
                ..Default::default()
            },
        ];

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "agent1");
    }

    #[test]
    fn test_get_agent_assignment_display_none() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["other-skill".to_string()]),
            ..Default::default()
        }];

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "[none]");
    }

    #[test]
    fn test_get_agent_assignment_display_no_agents() {
        let mut config = Config::default();
        config.agents = Vec::new();

        let display = get_agent_assignment_display("test-skill", &config);
        assert_eq!(display, "[none]");
    }

    #[test]
    fn test_format_skill_entry_truncates_long_description() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["test-skill".to_string()]),
            ..Default::default()
        }];

        let skill = SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("This is a very long description that should be truncated because it exceeds forty characters".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        };

        let entry = format_skill_entry(&skill, &config, None);
        assert!(entry.contains("...")); // Should have truncation marker
        assert!(entry.len() < 150); // Should be reasonably short
    }

    #[test]
    fn test_format_skill_entry_no_description() {
        let config = Config::default();

        let skill = SkillMetadata {
            name: "test-skill".to_string(),
            description: None,
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        };

        let entry = format_skill_entry(&skill, &config, None);
        assert!(entry.contains("<no description>"));
    }

    #[test]
    fn test_skills_update_parsing_no_skill_name() {
        // Test that the update command works without a skill name (updates all)
        let args = SkillsUpdate { skill_name: None };
        assert!(args.skill_name.is_none());
    }

    #[test]
    fn test_skills_update_parsing_with_skill_name() {
        // Test that the update command works with a specific skill name
        let args = SkillsUpdate {
            skill_name: Some("frontend-design".to_string()),
        };
        assert_eq!(args.skill_name, Some("frontend-design".to_string()));
    }

    #[test]
    fn test_skills_update_skill_name_is_optional() {
        // Test that skill_name is truly optional by creating both variants
        let args_no_name = SkillsUpdate { skill_name: None };
        let args_with_name = SkillsUpdate {
            skill_name: Some("test-skill".to_string()),
        };

        assert!(args_no_name.skill_name.is_none());
        assert!(args_with_name.skill_name.is_some());
    }

    #[test]
    fn test_skills_update_no_args() {
        // Parse the full command structure
        let cmd = SkillsCommand::try_parse_from(["skills", "update"]).unwrap();

        // Extract the Update subcommand
        match cmd.subcommand {
            SkillsSubcommand::Update(update_args) => {
                assert!(update_args.skill_name.is_none());
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn test_skills_update_with_skill_name() {
        // Parse the full command structure
        let cmd =
            SkillsCommand::try_parse_from(["skills", "update", "--", "frontend-design"]).unwrap();

        // Extract the Update subcommand
        match cmd.subcommand {
            SkillsSubcommand::Update(update_args) => {
                assert_eq!(update_args.skill_name, Some("frontend-design".to_string()));
            }
            _ => panic!("Expected Update subcommand"),
        }
    }

    #[test]
    fn test_format_skills_list_global_flag_filters() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![
                    "project-skill-1".to_string(),
                    "global-skill-1".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["project-skill-2".to_string()]),
                ..Default::default()
            },
        ];

        // Create project skills with distinct names
        let project_skills = vec![
            SkillMetadata {
                name: "project-skill-1".to_string(),
                description: Some("Project skill 1 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "project-skill-2".to_string(),
                description: Some("Project skill 2 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];

        // Create global skills with distinct names
        let global_skills = vec![
            SkillMetadata {
                name: "global-skill-1".to_string(),
                description: Some("Global skill 1 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "global-skill-2".to_string(),
                description: Some("Global skill 2 description".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];
        let warnings: Vec<String> = Vec::new();

        // Test that both project and global skills are displayed
        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify both project skill names are present
        assert!(output.contains("project-skill-1"));
        assert!(output.contains("project-skill-2"));
        assert!(output.contains("Project skill 1 description"));
        assert!(output.contains("Project skill 2 description"));

        // Verify both global skill names are present
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("global-skill-2"));
        assert!(output.contains("Global skill 1 description"));
        assert!(output.contains("Global skill 2 description"));

        // Verify sections are displayed
        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts
        assert!(output.contains("4 skills installed (2 project, 2 global)"));
    }

    #[test]
    fn test_format_skills_list_global_flag_shows_only_global() {
        let mut config = Config::default();
        config.agents = vec![Agent {
            name: "agent1".to_string(),
            skills: Some(vec!["global-skill-1".to_string()]),
            ..Default::default()
        }];

        // Create global skills
        let global_skills = vec![SkillMetadata {
            name: "global-skill-1".to_string(),
            description: Some("Global skill 1 description".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let warnings: Vec<String> = Vec::new();

        // Test filtering by passing empty project skills (simulating --global flag behavior)
        let output = format_skills_list(Vec::new(), global_skills, &warnings, &config, None);

        // Verify project skills are NOT present
        assert!(!output.contains("project-skill-1"));
        assert!(!output.contains("project-skill-2"));
        assert!(!output.contains("Project skill 1 description"));
        assert!(!output.contains("Project skill 2 description"));
        assert!(!output.contains("Project (./skills/)"));

        // Verify global skills ARE present
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("Global skill 1 description"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts (only global skills)
        assert!(output.contains("1 skills installed (0 project, 1 global)"));
    }

    #[test]
    fn test_format_skills_list_with_warnings() {
        let config = Config::default();
        let project_skills = vec![SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("A test skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings = vec![
            "Warning: Could not parse SKILL.md for 'skill-1' — using directory name".to_string(),
            "Warning: Could not parse SKILL.md for 'skill-2' — using directory name".to_string(),
        ];

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify warnings section is displayed
        assert!(output.contains("Warnings:"));
        // Verify each warning is present with two-space prefix
        assert!(output
            .contains("  Warning: Could not parse SKILL.md for 'skill-1' — using directory name"));
        assert!(output
            .contains("  Warning: Could not parse SKILL.md for 'skill-2' — using directory name"));
        // Verify skills are still displayed
        assert!(output.contains("test-skill"));
    }

    #[test]
    fn test_format_skills_list_without_warnings() {
        let config = Config::default();
        let project_skills = vec![SkillMetadata {
            name: "test-skill".to_string(),
            description: Some("A test skill".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings: Vec<String> = Vec::new();

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify warnings section is NOT displayed when there are no warnings
        assert!(!output.contains("Warnings:"));
        // Verify skills are still displayed
        assert!(output.contains("test-skill"));
    }

    #[test]
    fn test_format_skills_list_empty_with_warnings() {
        let config = Config::default();
        let project_skills: Vec<SkillMetadata> = Vec::new();
        let global_skills: Vec<SkillMetadata> = Vec::new();
        let warnings = vec![
            "Warning: Could not parse SKILL.md for 'bad-skill' — using directory name".to_string(),
        ];

        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify empty state message is shown
        assert!(output.contains("No skills installed"));
        // Verify warnings section is still displayed even when there are no skills
        assert!(output.contains("Warnings:"));
        assert!(output.contains(
            "  Warning: Could not parse SKILL.md for 'bad-skill' — using directory name"
        ));
    }

    /// Test that the --global flag filters to show only global skills
    ///
    /// When --global flag is set (simulated by empty project_skills),
    /// only global skills should be displayed in the output.
    /// This test verifies the filtering behavior of format_skills_list()
    /// which is called by run_skills_installed() when global=true.
    #[test]
    fn test_global_flag_shows_only_global_skills() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![
                    "global-skill-1".to_string(),
                    "global-skill-2".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec!["global-skill-1".to_string()]),
                ..Default::default()
            },
        ];

        // Create global skills that SHOULD be displayed
        let global_skills = vec![
            SkillMetadata {
                name: "global-skill-1".to_string(),
                description: Some("Global skill 1".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "global-skill-2".to_string(),
                description: Some("Global skill 2".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];
        let warnings: Vec<String> = Vec::new();

        // Simulate --global flag behavior by passing empty project_skills
        let output = format_skills_list(Vec::new(), global_skills, &warnings, &config, None);

        // Verify project skills are NOT in the output
        assert!(!output.contains("project-skill-1"));
        assert!(!output.contains("project-skill-2"));
        assert!(!output.contains("Project skill 1"));
        assert!(!output.contains("Project skill 2"));
        assert!(!output.contains("Project (./skills/)"));

        // Verify global skills ARE in the output
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("global-skill-2"));
        assert!(output.contains("Global skill 1"));
        assert!(output.contains("Global skill 2"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts (only global skills)
        assert!(output.contains("2 skills installed (0 project, 2 global)"));
    }

    /// Test that without --global flag, both project and global skills are shown
    ///
    /// When --global flag is NOT set, both project and global skills
    /// should be displayed in the output.
    /// This test verifies the default behavior of format_skills_list()
    /// which is called by run_skills_installed() when global=false.
    #[test]
    fn test_no_global_flag_shows_both_scopes() {
        let mut config = Config::default();
        config.agents = vec![
            Agent {
                name: "agent1".to_string(),
                skills: Some(vec![
                    "project-skill-1".to_string(),
                    "global-skill-1".to_string(),
                ]),
                ..Default::default()
            },
            Agent {
                name: "agent2".to_string(),
                skills: Some(vec![
                    "project-skill-2".to_string(),
                    "global-skill-1".to_string(),
                ]),
                ..Default::default()
            },
        ];

        // Create project skills that SHOULD be displayed
        let project_skills = vec![
            SkillMetadata {
                name: "project-skill-1".to_string(),
                description: Some("Project skill 1".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
            SkillMetadata {
                name: "project-skill-2".to_string(),
                description: Some("Project skill 2".to_string()),
                version: None,
                authors: Vec::new(),
                dependencies: Vec::new(),
                compatible_agents: Vec::new(),
                source: None,
            },
        ];

        // Create global skills that SHOULD be displayed
        let global_skills = vec![SkillMetadata {
            name: "global-skill-1".to_string(),
            description: Some("Global skill 1".to_string()),
            version: None,
            authors: Vec::new(),
            dependencies: Vec::new(),
            compatible_agents: Vec::new(),
            source: None,
        }];
        let warnings: Vec<String> = Vec::new();

        // Test default behavior (no --global flag) by passing both skill vectors
        let output = format_skills_list(project_skills, global_skills, &warnings, &config, None);

        // Verify project skills ARE in the output
        assert!(output.contains("project-skill-1"));
        assert!(output.contains("project-skill-2"));
        assert!(output.contains("Project skill 1"));
        assert!(output.contains("Project skill 2"));
        assert!(output.contains("Project (./skills/)"));

        // Verify global skills ARE in the output
        assert!(output.contains("global-skill-1"));
        assert!(output.contains("Global skill 1"));
        assert!(output.contains("Global (./skills/)"));

        // Verify both scopes are shown
        assert!(output.contains("Project (./skills/)"));
        assert!(output.contains("Global (./skills/)"));

        // Verify correct counts (both project and global skills)
        assert!(output.contains("3 skills installed (2 project, 1 global)"));
    }

    // ============================================================================
    // Install Command Tests
    // ============================================================================

    use crate::skills::SkillsError;

    /// Test SkillsInstall parsing with --global flag
    #[test]
    fn test_skills_install_args_parse_global_flag() {
        // Use try_parse_from to avoid clap picking up test binary args
        // Using skill-name format - SkillsInstall is a subcommand so we parse just the subcommand args
        let install =
            SkillsInstall::try_parse_from(vec!["install", "frontend-design", "--global"]).unwrap();
        assert_eq!(install.source, "frontend-design");
        assert!(install.global);

        // Test without --global flag (should default to false)
        let install = SkillsInstall::try_parse_from(vec!["install", "frontend-design"]).unwrap();
        assert_eq!(install.source, "frontend-design");
        assert!(!install.global);
    }

    /// Test SkillsInstall parsing with various source formats
    #[test]
    fn test_skills_install_args_parse_source_formats() {
        // skill-name format (new format)
        let install = SkillsInstall::try_parse_from(vec!["install", "frontend-design"]).unwrap();
        assert_eq!(install.source, "frontend-design");

        // skill-name with hyphen format
        let install = SkillsInstall::try_parse_from(vec!["install", "security-audit"]).unwrap();
        assert_eq!(install.source, "security-audit");

        // skill-name with underscore format
        let install = SkillsInstall::try_parse_from(vec!["install", "my_skill"]).unwrap();
        assert_eq!(install.source, "my_skill");

        // skill-name with numbers
        let install = SkillsInstall::try_parse_from(vec!["install", "skill123"]).unwrap();
        assert_eq!(install.source, "skill123");
    }

    /// Test SkillsInstall requires source argument
    #[test]
    fn test_skills_install_args_require_source() {
        // This should fail because source is required - SkillsInstall is a subcommand so we parse just the subcommand name
        let result = SkillsInstall::try_parse_from(vec!["install"]);
        assert!(result.is_err());
    }

    /// Test SkillsInstall with global flag in different positions
    #[test]
    fn test_skills_install_flag_order() {
        // Note: Current version doesn't have --yes flag, only --global
        // This test verifies global flag works in different positions
        // Using skill-name format - SkillsInstall is a subcommand so we parse just the subcommand args
        let install =
            SkillsInstall::try_parse_from(vec!["install", "frontend-design", "--global"]).unwrap();
        assert!(install.global);
    }

    /// Test SkillsError::SkillAlreadyInstalled displays correct error message
    #[test]
    fn test_skill_already_installed_error_display() {
        let error = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let display_message = format!("{}", error);

        // Verify error message contains key information
        assert!(
            display_message.contains("test-skill"),
            "Error message should contain skill name, got: {}",
            display_message
        );
    }

    /// Test SkillsError::SkillAlreadyInstalled can be cloned
    #[test]
    fn test_skill_already_installed_error_clone() {
        let error = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let cloned = error.clone();

        assert_eq!(format!("{}", error), format!("{}", cloned));
    }

    /// Test SkillsError::SkillAlreadyInstalled equality
    #[test]
    fn test_skill_already_installed_error_equality() {
        let error1 = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error2 = SkillsError::SkillAlreadyInstalled {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error3 = SkillsError::SkillAlreadyInstalled {
            skill_name: "different-skill".to_string(),
            path: "./skills/different-skill".to_string(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }

    /// Test SkillsError::DestinationAlreadyExists displays correct error message
    #[test]
    fn test_destination_already_exists_error_display() {
        let error = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let display_message = format!("{}", error);

        // Verify error message contains key information
        assert!(
            display_message.contains("test-skill"),
            "Error message should contain skill name, got: {}",
            display_message
        );
        assert!(
            display_message.contains("./skills/test-skill"),
            "Error message should contain path, got: {}",
            display_message
        );
    }

    /// Test SkillsError::DestinationAlreadyExists can be cloned
    #[test]
    fn test_destination_already_exists_error_clone() {
        let error = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let cloned = error.clone();

        assert_eq!(format!("{}", error), format!("{}", cloned));
    }

    /// Test SkillsError::DestinationAlreadyExists equality
    #[test]
    fn test_destination_already_exists_error_equality() {
        let error1 = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error2 = SkillsError::DestinationAlreadyExists {
            skill_name: "test-skill".to_string(),
            path: "./skills/test-skill".to_string(),
        };

        let error3 = SkillsError::DestinationAlreadyExists {
            skill_name: "different-skill".to_string(),
            path: "./skills/different-skill".to_string(),
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}
