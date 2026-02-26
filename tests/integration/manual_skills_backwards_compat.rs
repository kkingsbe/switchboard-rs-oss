//! Integration tests for manually managed skills backwards compatibility
//!
//! These tests verify that:
//! 1. `find_preexisting_skills()` correctly detects skills in `.kilocode/skills/`
//! 2. The entrypoint script generation skips `npx` installation for preexisting skills
//! 3. The correct log format is generated for preexisting skills
//!
//! See [`BACKWARDS_COMPATIBILITY_SKILLS.md`](../../BACKWARDS_COMPATIBILITY_SKILLS.md:1) for
//! detailed documentation on backwards compatibility behavior.

use switchboard::docker::run::find_preexisting_skills;
use switchboard::docker::skills::generate_entrypoint_script;
use std::path::Path;

/// Test that `find_preexisting_skills()` correctly detects manual skills in `.kilocode/skills/`
///
/// This test verifies that:
/// 1. Skills that exist in the `.kilocode/skills/` directory with `SKILL.md` are detected
/// 2. Non-existent skills are not included in the result
/// 3. The skill name is correctly extracted from the skill source string
///
/// The test uses the fixture directory `tests/fixtures/manual-skills/` which contains:
/// - `.kilocode/skills/test-skill/SKILL.md` - a manually installed skill
/// - `.kilocode/skills/another-skill/SKILL.md` - another manually installed skill
#[test]
fn test_find_preexisting_skills_detects_manual_skills() {
    // Use the fixture directory
    let fixture_dir = Path::new("tests/fixtures/manual-skills");

    // Create a list of configured skills that match the manual skills
    // format: owner/repo where repo matches the skill directory name
    let configured_skills = vec![
        "test-owner/test-skill".to_string(),
        "another-owner/another-skill".to_string(),
    ];

    // Call find_preexisting_skills with the configured skills and fixture directory
    let result = find_preexisting_skills(&configured_skills, fixture_dir)
        .expect("find_preexisting_skills should succeed");

    // Verify that both test-skill and another-skill are detected
    assert_eq!(
        result.len(),
        2,
        "Should detect exactly 2 preexisting skills"
    );
    assert!(
        result.contains(&"test-skill".to_string()),
        "Result should contain 'test-skill'"
    );
    assert!(
        result.contains(&"another-skill".to_string()),
        "Result should contain 'another-skill'"
    );

    // Verify the order doesn't matter (use HashSet comparison)
    let expected: std::collections::HashSet<_> = result.iter().cloned().collect();
    let expected_skills: std::collections::HashSet<_> = vec!["test-skill", "another-skill"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        expected, expected_skills,
        "Result should contain both expected skills"
    );
}

/// Test that `find_preexisting_skills()` correctly handles skill names with @ suffix
///
/// This test verifies that skills in the format `owner/repo@skill-name` are correctly
/// detected when the skill directory name matches the skill-name part after the `@`.
#[test]
fn test_find_preexisting_skills_with_skill_name_suffix() {
    let fixture_dir = Path::new("tests/fixtures/manual-skills");

    // Test with @skill-name suffix format
    let configured_skills = vec![
        "test-owner/test-repo@test-skill".to_string(),
        "another-owner/another-repo@another-skill".to_string(),
    ];

    let result = find_preexisting_skills(&configured_skills, fixture_dir)
        .expect("find_preexisting_skills should succeed");

    // Verify that both skills are detected using the skill-name part after @
    assert_eq!(
        result.len(),
        2,
        "Should detect exactly 2 preexisting skills"
    );
    assert!(
        result.contains(&"test-skill".to_string()),
        "Result should contain 'test-skill' (extracted from 'test-owner/test-repo@test-skill')"
    );
    assert!(
        result.contains(&"another-skill".to_string()),
        "Result should contain 'another-skill' (extracted from 'another-owner/another-repo@another-skill')"
    );
}

/// Test that `find_preexisting_skills()` correctly handles non-existent skills
///
/// This test verifies that skills that are configured but don't exist in the
/// `.kilocode/skills/` directory are not included in the result.
#[test]
fn test_find_preexisting_skills_handles_nonexistent_skills() {
    let fixture_dir = Path::new("tests/fixtures/manual-skills");

    // Mix of existing and non-existing skills
    let configured_skills = vec![
        "test-owner/test-skill".to_string(),               // exists
        "nonexistent-owner/nonexistent-skill".to_string(), // does not exist
    ];

    let result = find_preexisting_skills(&configured_skills, fixture_dir)
        .expect("find_preexisting_skills should succeed");

    // Only test-skill should be detected
    assert_eq!(result.len(), 1, "Should detect exactly 1 preexisting skill");
    assert_eq!(
        result[0], "test-skill",
        "Only 'test-skill' should be detected"
    );
}

/// Test that `find_preexisting_skills()` handles empty skills list
///
/// This test verifies that an empty skills list returns an empty result.
#[test]
fn test_find_preexisting_skills_empty_list() {
    let fixture_dir = Path::new("tests/fixtures/manual-skills");
    let configured_skills: Vec<String> = vec![];

    let result = find_preexisting_skills(&configured_skills, fixture_dir)
        .expect("find_preexisting_skills should succeed");

    assert_eq!(
        result.len(),
        0,
        "Empty skills list should return empty result"
    );
}

/// Test that `find_preexisting_skills()` handles missing `.kilocode/skills/` directory
///
/// This test verifies that if the `.kilocode/skills/` directory doesn't exist,
/// the function returns an empty result without error.
#[test]
fn test_find_preexisting_skills_missing_skills_directory() {
    // Use a directory that doesn't have .kilocode/skills/
    let fixture_dir = Path::new("tests/fixtures");

    let configured_skills = vec!["test-owner/test-skill".to_string()];

    let result = find_preexisting_skills(&configured_skills, fixture_dir)
        .expect("find_preexisting_skills should succeed");

    assert_eq!(
        result.len(),
        0,
        "Missing .kilocode/skills/ should return empty result"
    );
}

/// Test that `generate_entrypoint_script()` skips npx for preexisting skills
///
/// This test verifies that:
/// 1. Preexisting skills have log messages about skipping npx
/// 2. Non-preexisting skills have `npx skills add` commands
/// 3. The expected log format is present: `[SKILL INSTALL] Using preexisting skill: <skill-name> (skipping npx installation)`
#[test]
fn test_generate_entrypoint_script_skips_npx_for_preexisting_skills() {
    // Create a list of configured skills
    let configured_skills = vec![
        "test-owner/test-skill".to_string(), // preexisting
        "hypothetical-owner/hypothetical-skill".to_string(), // not preexisting
        "another-owner/another-skill".to_string(), // preexisting
    ];

    // Create a list of preexisting skills (as returned by find_preexisting_skills)
    let preexisting_skills = vec!["test-skill".to_string(), "another-skill".to_string()];

    // Generate the entrypoint script
    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    // Verify the script contains the shebang
    assert!(
        script.contains("#!/bin/sh"),
        "Script should contain shebang"
    );

    // Verify the script contains error handling setup
    assert!(script.contains("set -e"), "Script should contain 'set -e'");

    // Verify preexisting skills have the skip npx log message
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: test-skill (skipping npx installation)"
        ),
        "Script should contain skip npx log for test-skill"
    );
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: another-skill (skipping npx installation)"
        ),
        "Script should contain skip npx log for another-skill"
    );

    // Verify non-preexisting skills have npx skills add command
    assert!(
        script.contains("npx skills add hypothetical-owner/hypothetical-skill -a kilo -y"),
        "Script should contain npx skills add command for hypothetical-skill"
    );

    // Verify preexisting skills do NOT have npx skills add command
    assert!(
        !script.contains("npx skills add test-owner/test-skill"),
        "Script should NOT contain npx skills add for preexisting test-skill"
    );
    assert!(
        !script.contains("npx skills add another-owner/another-skill"),
        "Script should NOT contain npx skills add for preexisting another-skill"
    );

    // Verify the script ends with exec kilocode
    assert!(
        script.contains("exec kilocode --yes \"$@\""),
        "Script should contain exec kilocode command"
    );
}

/// Test that `generate_entrypoint_script()` handles all skills preexisting
///
/// This test verifies that when all skills are preexisting, the script only
/// contains log messages and no npx commands.
#[test]
fn test_generate_entrypoint_script_all_preexisting() {
    let configured_skills = vec![
        "test-owner/test-skill".to_string(),
        "another-owner/another-skill".to_string(),
    ];

    let preexisting_skills = vec!["test-skill".to_string(), "another-skill".to_string()];

    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    // Verify no npx skills add commands
    assert!(
        !script.contains("npx skills add"),
        "Script should NOT contain any npx skills add commands when all skills are preexisting"
    );

    // Verify both skills have skip npx log messages
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: test-skill (skipping npx installation)"
        ),
        "Script should contain skip npx log for test-skill"
    );
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: another-skill (skipping npx installation)"
        ),
        "Script should contain skip npx log for another-skill"
    );
}

/// Test that `generate_entrypoint_script()` handles no preexisting skills
///
/// This test verifies that when no skills are preexisting, the script contains
/// npx commands for all skills.
#[test]
fn test_generate_entrypoint_script_no_preexisting() {
    let configured_skills = vec![
        "test-owner/test-skill".to_string(),
        "another-owner/another-skill".to_string(),
    ];

    let preexisting_skills: Vec<String> = vec![];

    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    // Verify no skip npx log messages
    assert!(
        !script.contains("[SKILL INSTALL] Using preexisting skill"),
        "Script should NOT contain skip npx log messages when no skills are preexisting"
    );

    // Verify all skills have npx skills add commands
    assert!(
        script.contains("npx skills add test-owner/test-skill -a kilo -y"),
        "Script should contain npx skills add for test-skill"
    );
    assert!(
        script.contains("npx skills add another-owner/another-skill -a kilo -y"),
        "Script should contain npx skills add for another-skill"
    );
}

/// Test that `generate_entrypoint_script()` handles empty skills list
///
/// This test verifies that an empty skills list returns an empty script.
#[test]
fn test_generate_entrypoint_script_empty_skills() {
    let configured_skills: Vec<String> = vec![];
    let preexisting_skills: Vec<String> = vec![];

    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    assert!(
        script.is_empty(),
        "Empty skills list should return empty script"
    );
}

/// Test integration between `find_preexisting_skills()` and `generate_entrypoint_script()`
///
/// This test verifies the end-to-end flow:
/// 1. Find preexisting skills from the fixture directory
/// 2. Generate entrypoint script with the preexisting skills
/// 3. Verify the script correctly skips npx for preexisting skills
#[test]
fn test_find_preexisting_skills_and_generate_entrypoint_script_integration() {
    // Use the fixture directory
    let fixture_dir = Path::new("tests/fixtures/manual-skills");

    // Create a list of configured skills (mix of manual and hypothetical)
    let configured_skills = vec![
        "test-owner/test-skill".to_string(),       // exists in fixtures
        "another-owner/another-skill".to_string(), // exists in fixtures
        "hypothetical-owner/hypothetical-skill".to_string(), // does not exist
    ];

    // Step 1: Find preexisting skills
    let preexisting_skills = find_preexisting_skills(&configured_skills, fixture_dir)
        .expect("find_preexisting_skills should succeed");

    // Verify the correct skills were found
    assert_eq!(
        preexisting_skills.len(),
        2,
        "Should find exactly 2 preexisting skills"
    );
    assert!(
        preexisting_skills.contains(&"test-skill".to_string()),
        "Should find test-skill"
    );
    assert!(
        preexisting_skills.contains(&"another-skill".to_string()),
        "Should find another-skill"
    );

    // Step 2: Generate entrypoint script with the preexisting skills
    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    // Step 3: Verify the script correctly skips npx for preexisting skills
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: test-skill (skipping npx installation)"
        ),
        "Script should skip npx for test-skill"
    );
    assert!(
        script.contains(
            "[SKILL INSTALL] Using preexisting skill: another-skill (skipping npx installation)"
        ),
        "Script should skip npx for another-skill"
    );
    assert!(
        script.contains("npx skills add hypothetical-owner/hypothetical-skill -a kilo -y"),
        "Script should include npx for hypothetical-skill"
    );
}

/// Test that `generate_entrypoint_script()` preserves skill order
///
/// This test verifies that the entrypoint script preserves the order of skills
/// as specified in the configuration.
#[test]
fn test_generate_entrypoint_script_preserves_order() {
    let configured_skills = vec![
        "owner1/skill1".to_string(),
        "owner2/skill2".to_string(),
        "owner3/skill3".to_string(),
    ];

    let preexisting_skills = vec!["skill2".to_string()];

    let script = generate_entrypoint_script("test-agent", &configured_skills, &preexisting_skills)
        .expect("generate_entrypoint_script should succeed");

    // Find the positions of each skill in the script
    let skill1_pos = script.find("skill1");
    let skill2_pos = script.find("skill2");
    let skill3_pos = script.find("skill3");

    // Verify all skills are present
    assert!(skill1_pos.is_some(), "skill1 should be in the script");
    assert!(skill2_pos.is_some(), "skill2 should be in the script");
    assert!(skill3_pos.is_some(), "skill3 should be in the script");

    // Verify order is preserved
    let skill1_pos = skill1_pos.unwrap();
    let skill2_pos = skill2_pos.unwrap();
    let skill3_pos = skill3_pos.unwrap();

    assert!(
        skill1_pos < skill2_pos && skill2_pos < skill3_pos,
        "Skills should appear in the script in the order they were configured"
    );
}
