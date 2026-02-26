//! Integration tests for the `switchboard skills installed` command

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Test listing installed skills with project-level skills
#[test]
fn test_installed_with_project_skills() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create the project skills directory
    let skills_dir = temp_dir.path().join(".kilocode").join("skills");
    fs::create_dir_all(&skills_dir).unwrap();

    // Create first skill directory with SKILL.md
    let skill1_dir = skills_dir.join("frontend-design");
    fs::create_dir_all(&skill1_dir).unwrap();
    let skill1_md = skill1_dir.join("SKILL.md");
    fs::write(
        &skill1_md,
        r#"---
name: frontend-design
description: A skill for frontend design and UI components
version: 0.1.0
---

# Frontend Design Skill

This skill provides capabilities for designing frontend interfaces.
"#,
    )
    .unwrap();

    // Create second skill directory with SKILL.md
    let skill2_dir = skills_dir.join("backend-api");
    fs::create_dir_all(&skill2_dir).unwrap();
    let skill2_md = skill2_dir.join("SKILL.md");
    fs::write(
        &skill2_md,
        r#"---
name: backend-api
description: Backend API development and REST endpoints
version: 0.2.0
---

# Backend API Skill

This skill provides capabilities for developing backend APIs.
"#,
    )
    .unwrap();

    // Create third skill directory with SKILL.md
    let skill3_dir = skills_dir.join("database-operations");
    fs::create_dir_all(&skill3_dir).unwrap();
    let skill3_md = skill3_dir.join("SKILL.md");
    fs::write(
        &skill3_md,
        r#"---
name: database-operations
description: Database query operations and schema management
version: 1.0.0
---

# Database Operations Skill

This skill provides capabilities for database operations.
"#,
    )
    .unwrap();

    // Run the installed command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "skills",
            "installed",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(contains("Installed Skills"))
        .stdout(contains("Project (.kilocode/skills/)"))
        .stdout(contains("frontend-design"))
        .stdout(contains("A skill for frontend design"))
        .stdout(contains("backend-api"))
        .stdout(contains("Backend API development"))
        .stdout(contains("database-operations"))
        .stdout(contains("Database query operations"))
        .stdout(contains("3 skills installed"));
}

/// Test listing installed skills when no skills are present
#[test]
fn test_installed_no_skills() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create the project skills directory but leave it empty
    let skills_dir = temp_dir.path().join(".kilocode").join("skills");
    fs::create_dir_all(&skills_dir).unwrap();

    // Run the installed command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "skills",
            "installed",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(contains("Installed Skills"))
        .stdout(contains("No skills installed"))
        .stdout(contains(
            "Browse available skills with: switchboard skills list",
        ))
        .stdout(contains(
            "Install a skill with: switchboard skills install <source>",
        ));
}

/// Test listing installed global skills with --global flag
#[test]
fn test_installed_with_global_flag() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file
    let config_content = r#"
[[agent]]
name = "test-agent"
schedule = "0 0 9 * * *"
prompt = "Test prompt"
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create the global skills directory structure: ~/.kilocode/skills/
    let global_skills_dir = temp_dir.path().join(".kilocode").join("skills");
    fs::create_dir_all(&global_skills_dir).unwrap();

    // Create first global skill directory with SKILL.md
    let skill1_dir = global_skills_dir.join("code-review");
    fs::create_dir_all(&skill1_dir).unwrap();
    let skill1_md = skill1_dir.join("SKILL.md");
    fs::write(
        &skill1_md,
        r#"---
name: code-review
description: Automated code review and quality analysis
version: 1.0.0
---

# Code Review Skill

This skill provides automated code review capabilities.
"#,
    )
    .unwrap();

    // Create second global skill directory with SKILL.md
    let skill2_dir = global_skills_dir.join("documentation");
    fs::create_dir_all(&skill2_dir).unwrap();
    let skill2_md = skill2_dir.join("SKILL.md");
    fs::write(
        &skill2_md,
        r#"---
name: documentation
description: Documentation generation and maintenance
version: 0.5.0
---

# Documentation Skill

This skill provides documentation generation capabilities.
"#,
    )
    .unwrap();

    // Create third global skill directory with SKILL.md
    let skill3_dir = global_skills_dir.join("testing");
    fs::create_dir_all(&skill3_dir).unwrap();
    let skill3_md = skill3_dir.join("SKILL.md");
    fs::write(
        &skill3_md,
        r#"---
name: testing
description: Unit and integration test generation
version: 2.1.0
---

# Testing Skill

This skill provides test generation capabilities.
"#,
    )
    .unwrap();

    // Run the installed command with --global flag
    // Set HOME to temp_dir.path() so ~/.kilocode/skills/ resolves correctly
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "skills",
            "installed",
            "--global",
        ])
        .env("HOME", temp_dir.path())
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(contains("Installed Skills"))
        .stdout(contains("Global (~/.kilocode/skills/)"))
        .stdout(contains("code-review"))
        .stdout(contains("Automated code review"))
        .stdout(contains("documentation"))
        .stdout(contains("Documentation generation"))
        .stdout(contains("testing"))
        .stdout(contains("Unit and integration test"))
        .stdout(contains("3 skills installed (0 project, 3 global)"));
}

/// Test listing installed skills with agent assignments
#[test]
fn test_installed_with_agent_assignments() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("switchboard.toml");

    // Create a valid switchboard.toml config file with multiple agents and skill assignments
    // - frontend-design: assigned to agent1 and agent2 (specific agents)
    // - backend-api: assigned to all agents (all 3 agents)
    // - testing: not assigned to any agents (none)
    let config_content = r#"
[[agent]]
name = "agent1"
schedule = "0 0 9 * * *"
prompt = "Test prompt for agent1"
skills = ["owner1/repo1@frontend-design", "owner2/repo2@backend-api"]

[[agent]]
name = "agent2"
schedule = "0 0 10 * * *"
prompt = "Test prompt for agent2"
skills = ["owner1/repo1@frontend-design", "owner2/repo2@backend-api"]

[[agent]]
name = "agent3"
schedule = "0 0 11 * * *"
prompt = "Test prompt for agent3"
skills = ["owner2/repo2@backend-api"]
"#;
    fs::write(&config_path, config_content).unwrap();

    // Create the project skills directory
    let skills_dir = temp_dir.path().join(".kilocode").join("skills");
    fs::create_dir_all(&skills_dir).unwrap();

    // Create frontend-design skill (assigned to agent1, agent2)
    let skill1_dir = skills_dir.join("frontend-design");
    fs::create_dir_all(&skill1_dir).unwrap();
    let skill1_md = skill1_dir.join("SKILL.md");
    fs::write(
        &skill1_md,
        r#"---
name: frontend-design
description: A skill for frontend design and UI components
version: 0.1.0
---

# Frontend Design Skill

This skill provides capabilities for designing frontend interfaces.
"#,
    )
    .unwrap();

    // Create backend-api skill (assigned to all agents)
    let skill2_dir = skills_dir.join("backend-api");
    fs::create_dir_all(&skill2_dir).unwrap();
    let skill2_md = skill2_dir.join("SKILL.md");
    fs::write(
        &skill2_md,
        r#"---
name: backend-api
description: Backend API development and REST endpoints
version: 0.2.0
---

# Backend API Skill

This skill provides capabilities for developing backend APIs.
"#,
    )
    .unwrap();

    // Create testing skill (assigned to none)
    let skill3_dir = skills_dir.join("testing");
    fs::create_dir_all(&skill3_dir).unwrap();
    let skill3_md = skill3_dir.join("SKILL.md");
    fs::write(
        &skill3_md,
        r#"---
name: testing
description: Unit and integration test generation
version: 1.0.0
---

# Testing Skill

This skill provides test generation capabilities.
"#,
    )
    .unwrap();

    // Run the installed command
    Command::new(assert_cmd::cargo::cargo_bin!("switchboard"))
        .args([
            "--config",
            config_path.to_str().unwrap(),
            "skills",
            "installed",
        ])
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stdout(contains("Installed Skills"))
        .stdout(contains("Project (.kilocode/skills/)"))
        // Verify frontend-design shows specific agent names (agent1, agent2)
        .stdout(contains("frontend-design"))
        .stdout(contains("A skill for frontend design"))
        .stdout(contains("agent1, agent2"))
        // Verify backend-api shows [all agents]
        .stdout(contains("backend-api"))
        .stdout(contains("Backend API development"))
        .stdout(contains("[all agents]"))
        // Verify testing shows [none]
        .stdout(contains("testing"))
        .stdout(contains("Unit and integration test"))
        .stdout(contains("[none]"))
        .stdout(contains("3 skills installed (3 project, 0 global)"));
}
