# Backwards Compatibility: Skills Feature

## Overview

This document explains how the **skills feature** maintains full backwards compatibility with existing Switchboard projects. If your project was created before the skills feature was added, **your existing configuration will continue to work exactly as before**, with no modifications required.

---

## What is the Skills Feature?

The skills feature allows you to automatically install Kilo skills inside agent containers at startup. Skills are pre-built modules that extend the capabilities of your agents, such as frontend design tools, security auditors, or specialized code analysis utilities.

Skills are specified in the agent configuration using the optional `skills` field:

```toml
[[agent]]
name = "my-agent"
schedule = "0 * * * *"
prompt = "Analyze the codebase"
skills = [
    "vercel-labs/agent-skills@frontend-design",
    "anthropics/skills@security-audit",
]
```

When skills are specified, Switchboard generates a custom entrypoint script that installs them before the agent starts.

---

## Backwards Compatibility Guarantee

✅ **Your existing projects will continue to work without any changes.**

The `skills` field is **completely optional**. If your configuration file does not include the `skills` field:

- The configuration will parse successfully
- All switchboard commands will work normally
- No warnings or errors will be emitted
- Containers will use the default Dockerfile entrypoint
- Agent execution behavior is identical to before the skills feature was added

---

## How Configurations Without Skills Are Handled

Switchboard handles four scenarios for the skills field:

| Scenario | Config Value | Behavior |
|----------|-------------|----------|
| **No skills field** | `skills` field not present | Uses default Dockerfile entrypoint. No skill installation logic runs. |
| **Empty skills list** | `skills = []` | Uses default Dockerfile entrypoint. No skill installation logic runs. |
| **Preexisting skills** | Skills in `.kilocode/skills/` | Detected via `find_preexisting_skills()`, skipped during npx install, mounted as read-only |
| **Populated skills** | `skills = ["repo/skill"]` | Generates custom entrypoint script to install specified skills. |

### Technical Implementation

The code explicitly handles all three cases in [`src/docker/run/run.rs`](src/docker/run/run.rs:545-617):

```rust
// Three possible cases for config.skills:
// 1. None - No skills field exists, use default entrypoint from Dockerfile
// 2. Some([]) - Empty skills list exists, use default entrypoint from Dockerfile
// 3. Some([...]) - Non-empty skills list, generate custom entrypoint script
match &config.skills {
    Some(skills) if !skills.is_empty() => {
        // CASE 3: Non-empty skills list - generate custom entrypoint script
        // ... generate and set custom entrypoint
    }
    _ => {
        // CASE 1 & 2: No skills specified (None) or empty skills list (Some([]))
        // Use the default entrypoint from the Dockerfile (entrypoint: None)
        // No modification needed to container_config - skills integration is bypassed
    }
}
```

---

## Configuration Examples

### Example 1: Configuration Without Skills (Legacy)

```toml
# switchboard.toml - Legacy configuration without skills field
# This configuration works exactly as before the skills feature was added

[settings]
image_name = "switchboard-agent"
log_dir = ".switchboard/logs"

[[agent]]
name = "simple-agent"
schedule = "0 * * * *"
prompt = """
Analyze the current state of the codebase and provide a brief summary.
"""

[[agent]]
name = "comprehensive-agent"
schedule = "*/15 * * * *"
prompt = """
Review the recent changes in the codebase and identify potential issues.
"""
readonly = false
timeout = "1h"
```

**Behavior:**
- ✅ Parses successfully
- ✅ Validates without errors
- ✅ All commands work: `up`, `run`, `list`, `status`, `validate`
- ✅ Containers start normally using default entrypoint
- ✅ No warnings or errors about missing skills field

### Example 2: Configuration With Empty Skills

```toml
# switchboard.toml - Configuration with empty skills list
# This behaves identically to having no skills field

[[agent]]
name = "test-agent"
schedule = "0 9 * * *"
prompt = "Test prompt"
skills = []
```

**Behavior:**
- ✅ Parses successfully
- ✅ Uses default entrypoint (same as no skills field)
- ✅ No skill installation logic runs

### Example 3: Configuration With Populated Skills

```toml
# switchboard.toml - Configuration with skills specified
# Skills will be installed in the container at startup

[[agent]]
name = "skill-enhanced-agent"
schedule = "0 * * * *"
prompt = "Perform security analysis"
skills = [
    "vercel-labs/agent-skills@frontend-design",
    "anthropics/skills@security-audit",
]
```

**Behavior:**
- ✅ Parses successfully
- ✅ Generates custom entrypoint script
- ✅ Installs specified skills before agent starts
- ✅ Agent runs with skills available in its environment

---

### Example 4: Manually Managed Skills (Preexisting in `.kilocode/skills/`)

```toml
# switchboard.toml - Configuration where skills are manually managed
# Skills are pre-installed in the .kilocode/skills/ directory
# No skills field is specified - the system detects preexisting skills

[[agent]]
name = "manual-skill-agent"
schedule = "0 * * * *"
prompt = "Use pre-installed skills for analysis"
skills = [
    "owner/test-skill",
    "another/frontend-design",
]
```

**Directory Structure:**
```
project/
├── .kilocode/
│   └── skills/
│       ├── test-skill/
│       │   └── SKILL.md
│       └── frontend-design/
│           └── SKILL.md
└── switchboard.toml
```

**Behavior:**
- ✅ Skills are detected in `.kilocode/skills/` directory
- ✅ Each skill must contain a `SKILL.md` file to be recognized
- ✅ Preexisting skills skip `npx skills add` during container startup
- ✅ Log message: `[SKILL INSTALL] Using preexisting skill: <skill-name> (skipping npx installation)`
- ✅ Skills are mounted as read-only (already installed)
- ✅ Works with both `owner/repo` and `owner/repo@skill-name` formats

**Technical Implementation:**

The [`find_preexisting_skills()`](src/docker/run/run.rs:413-417) function detects manually installed skills:

```rust
pub fn find_preexisting_skills(
    skills: &[String],
    project_dir: &std::path::Path,
) -> Result<Vec<String>, SkillsError> {
    // Build the path to the .kilocode/skills directory
    let skills_dir = project_dir.join(".kilocode/skills");
    
    // If the skills directory doesn't exist, return empty vector
    if !skills_dir.exists() {
        return Ok(Vec::new());
    }
    
    // Read directory entries and check for SKILL.md files
    // Collect skill names from directories that contain SKILL.md
    // ...
}
```

The entrypoint script generation skips npx for preexisting skills ([`src/docker/skills.rs`](src/docker/skills.rs:382-384)):

```rust
skill_commands.push_str(&format!(
    "echo \"[SKILL INSTALL] Using preexisting skill: {} (skipping npx installation)\"\n",
    skill_name
));
// Skip the npx skills add command for preexisting skills
```

---

## Test Reference Configuration

A test configuration file without the skills field is included in the repository:

**File:** [`test-no-skills.toml`](test-no-skills.toml:1)

This file demonstrates a typical pre-skills configuration with:
- Global settings section
- Two agent configurations (simple-agent, comprehensive-agent)
- All standard fields except `skills`

You can reference this file to verify backwards compatibility:

```bash
# Validate the test config
./target/release/switchboard -c test-no-skills.toml validate

# List agents from the test config
./target/release/switchboard -c test-no-skills.toml list

# Check status with the test config
./target/release/switchboard -c test-no-skills.toml status
```

---

## Migration Path

If you want to add skills to an existing project, simply add the `skills` field to your agent configurations. No other changes are required.

### Step-by-Step Migration

1. **Open your existing `switchboard.toml` file**

2. **Add the `skills` field to any agents that need skills**

   ```toml
   # Before (no skills)
   [[agent]]
   name = "my-agent"
   schedule = "0 * * * *"
   prompt = "Analyze code"

   # After (with skills)
   [[agent]]
   name = "my-agent"
   schedule = "0 * * * *"
   prompt = "Analyze code"
   skills = [
       "vercel-labs/agent-skills@frontend-design",
   ]
   ```

3. **Validate your updated configuration**

   ```bash
   switchboard validate
   ```

4. **Rebuild your agent image (if needed)**

   ```bash
   switchboard build
   ```

5. **Start the scheduler**

   ```bash
   switchboard up
   ```

### Mixed Environments

You can have some agents with skills and some without in the same configuration file:

```toml
# Agent without skills (uses default entrypoint)
[[agent]]
name = "legacy-agent"
schedule = "0 * * * *"
prompt = "Simple analysis"

# Agent with skills (generates custom entrypoint)
[[agent]]
name = "skill-enhanced-agent"
schedule = "30 * * * *"
prompt = "Advanced analysis"
skills = [
    "anthropics/skills@security-audit",
]
```

Both agents will work correctly, with different container configurations.

---

## Testing

Comprehensive testing has been performed to verify backwards compatibility. The following test results are available:

### Test 1: Switchboard Commands Without Skills Field

**Test File:** [`comms/outbox/2026-02-20T14-28-00_worker4_test-results-no-skills-config.md`](comms/outbox/2026-02-20T14-28-00_worker4_test-results-no-skills-config.md)

**Tests Performed:**
- ✅ `switchboard --config test-no-skills.toml --help` - Help command
- ✅ `switchboard --config test-no-skills.toml validate` - Config validation
- ✅ `switchboard --config test-no-skills.toml list` - List agents
- ✅ `switchboard --config test-no-skills.toml status` - Check status

**Result:** All commands executed successfully with no warnings or errors about the missing skills field.

### Test 2: Container Creation Without Skills Field

**Test File:** [`comms/outbox/2026-02-20T14-37-00_worker4_test-results-container-creation-no-skills.md`](comms/outbox/2026-02-20T14-37-00_worker4_test-results-container-creation-no-skills.md)

**Tests Performed:**
- ✅ Configuration validation with `test-no-skills.toml`
- ✅ Code analysis of container creation logic
- ✅ Unit tests for no-skills scenario:
  - `test_skills_none_uses_default_entrypoint`
  - `test_skills_empty_uses_default_entrypoint`
  - `test_no_skills_configured`
  - `test_integration_complete_flow_no_skills`
  - `test_skills_field_handling_none`
  - `test_skills_field_handling_empty_vec`
- ✅ Type system verification

**Result:** All tests passed. The code correctly handles configurations without the skills field, using the default Dockerfile entrypoint.

### Test 3: Manually Managed Skills (Preexisting in `.kilocode/skills/`)

**Test File:** [`tests/integration/manual_skills_backwards_compat.rs`](tests/integration/manual_skills_backwards_compat.rs:1)

**Test Configuration:** [`test-manual-skills.toml`](test-manual-skills.toml:1)

**Tests Performed (7 tests):**
1. `test_find_preexisting_skills_detects_manual_skills`
   - Verifies that skills in `.kilocode/skills/` with `SKILL.md` are detected
   - Tests the `owner/repo` format
   
2. `test_find_preexisting_skills_with_skill_name_suffix`
   - Verifies correct handling of `owner/repo@skill-name` format
   - Tests skill name extraction from the `@` suffix

3. `test_find_preexisting_skills_handles_nonexistent_skills`
   - Verifies that configured skills not in `.kilocode/skills/` are not included
   - Tests mixed existing/non-existing skills scenario

4. `test_find_preexisting_skills_empty_list`
   - Verifies empty skills list returns empty result

5. `test_find_preexisting_skills_missing_skills_directory`
   - Verifies missing `.kilocode/skills/` directory returns empty result

6. `test_generate_entrypoint_script_skips_npx_for_preexisting_skills`
   - Verifies preexisting skills skip `npx skills add`
   - Verifies log format: `[SKILL INSTALL] Using preexisting skill: <skill-name> (skipping npx installation)`

7. `test_find_preexisting_skills_and_generate_entrypoint_script_integration`
   - Full integration test between detection and script generation

**Test Fixtures:**
- `tests/fixtures/manual-skills/.kilocode/skills/test-skill/SKILL.md`
- `tests/fixtures/manual-skills/.kilocode/skills/another-skill/SKILL.md`

**Result:** All tests passed. The system correctly:
- Detects manually installed skills in `.kilocode/skills/`
- Skips `npx` installation for preexisting skills
- Generates appropriate log messages
- Handles both `owner/repo` and `owner/repo@skill-name` formats

### Running the Tests

To verify backwards compatibility yourself, you can run:

```bash
# Validate test config
./target/release/switchboard -c test-no-skills.toml validate

# Run all docker tests
cargo test --lib docker:: -- --nocapture

# Run specific no-skills tests
cargo test --lib test_no_skills_configured -- --nocapture
cargo test --lib test_integration_complete_flow_no_skills -- --nocapture
cargo test --lib test_skills_none_uses_default_entrypoint -- --nocapture

# Run manual skills backwards compatibility tests
cargo test --test manual_skills_backwards_compat -- --nocapture

# Run specific manual skills tests
cargo test --lib test_find_preexisting_skills_detects_manual_skills -- --nocapture
cargo test --lib test_find_preexisting_skills_with_skill_name_suffix -- --nocapture
cargo test --lib test_generate_entrypoint_script_skips_npx_for_preexisting_skills -- --nocapture
```

---

## Frequently Asked Questions

### Q: Do I need to update my existing configuration files?

**A:** No. Your existing configuration files will continue to work without any changes. The `skills` field is optional.

### Q: Will I see warnings or errors if I don't use the skills field?

**A:** No. Switchboard will not emit any warnings or errors about the missing skills field. The feature is designed to be completely transparent to existing projects.

### Q: Can I have some agents with skills and some without in the same config?

**A:** Yes. Each agent can independently specify or omit the `skills` field. Switchboard will handle each agent appropriately.

### Q: What happens if I set `skills = []`?

**A:** An empty skills list behaves identically to having no skills field. Switchboard will use the default Dockerfile entrypoint and no skill installation logic will run.

### Q: Do I need to rebuild my Docker image to add skills?

**A:** The skills are installed dynamically at container startup. You do not need to rebuild your base Docker image. However, when you first add skills to an agent, the first run may take longer as the skills are downloaded and installed.

### Q: Can I remove the skills field after adding it?

**A:** Yes. Removing the `skills` field or setting it to an empty list will revert the agent to using the default entrypoint, with no skill installation.

### Q: What are manually managed skills?

**A:** Manually managed skills are skills that were installed directly into the `.kilocode/skills/` directory (instead of being installed via `npx skills add` at container startup). This is useful for legacy projects where skills were pre-installed before the skills feature was added to Switchboard.

### Q: How does Switchboard detect manually managed skills?

**A:** Switchboard uses the [`find_preexisting_skills()`](src/docker/run/run.rs:413-417) function to scan the `.kilocode/skills/` directory. Skills are identified by directories containing a `SKILL.md` file. The function matches configured skills against these directories.

### Q: What happens if a skill is both configured in switchboard.toml AND exists in .kilocode/skills/?

**A:** If a skill is already installed in `.kilocode/skills/`, Switchboard will skip the `npx skills add` installation step and log: `[SKILL INSTALL] Using preexisting skill: <skill-name> (skipping npx installation)`. This avoids redundant installation and works with read-only skill directories.

---

## Summary

| Aspect | Status |
|--------|--------|
| **Backwards Compatibility** | ✅ Fully maintained |
| **Scenario 1: No skills field** | ✅ Uses default entrypoint |
| **Scenario 2: Empty skills list** | ✅ Uses default entrypoint |
| **Scenario 3: Manually managed skills** | ✅ Detected in `.kilocode/skills/`, npx skipped |
| **Scenario 4: Populated skills** | ✅ Custom entrypoint generated |
| **Error Messages** | ✅ No warnings or errors for missing skills |
| **Container Behavior** | ✅ Uses default entrypoint when skills absent |
| **Test Coverage** | ✅ Comprehensive unit and integration tests |
| **Documentation** | ✅ Reference test configs provided |

The skills feature is designed as a **non-breaking addition** to Switchboard. Existing projects continue to work identically, while new projects (or updated projects) can optionally take advantage of automatic skill installation.

---

## Additional Resources

- **Skills Feature Documentation:** [`addtl-features/skills-feature.md`](addtl-features/skills-feature.md)
- **Sample Configuration:** [`switchboard.sample.toml`](switchboard.sample.toml)
- **Test Configuration (No Skills):** [`test-no-skills.toml`](test-no-skills.toml)
- **Test Configuration (Empty Skills):** [`test-empty-skills.toml`](test-empty-skills.toml)
- **Test Configuration (Manual Skills):** [`test-manual-skills.toml`](test-manual-skills.toml)
- **Integration Tests:** [`tests/integration/manual_skills_backwards_compat.rs`](tests/integration/manual_skills_backwards_compat.rs:1)
- **Test Results (Commands):** [`comms/outbox/2026-02-20T14-28-00_worker4_test-results-no-skills-config.md`](comms/outbox/2026-02-20T14-28-00_worker4_test-results-no-skills-config.md)
- **Test Results (Containers):** [`comms/outbox/2026-02-20T14-37-00_worker4_test-results-container-creation-no-skills.md`](comms/outbox/2026-02-20T14-37-00_worker4_test-results-container-creation-no-skills.md)
