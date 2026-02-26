# Switchboard-RS Implementation Plan: Closing Remaining Gaps

## Executive Summary

After thorough analysis of the switchboard-rs codebase, I found that **most gaps described in the task have already been implemented**. The primary remaining gap is the orphaned skills warning in the validate command.

### Gap Status Summary

| Gap Category | Gap Description | Status |
|--------------|-----------------|--------|
| Skills | Skills directory location | ✅ Already Correct |
| Skills | `switchboard validate` skills integration | ✅ Already Implemented |
| Skills | Orphaned skills warning | ⚠️ **Needs Implementation** |
| Skills | Post-install cleanup | ✅ Already Implemented |
| Discord | Message chunking | ✅ Already Implemented |
| Discord | Tool iteration limit | ✅ Already Implemented |
| Discord | System prompt file wiring | ✅ Already Implemented |
| Discord | Resume connection handling | ✅ Already Implemented |

---

## Skills Feature Gaps

### Gap 1: Skills Directory Location

**Status**: ✅ Already Correct

The project uses `./skills/` as the skills directory (not `.kilocode/skills/`). This is confirmed in:
- [`src/skills/mod.rs:66`](src/skills/mod.rs:66) - `skills_dir: PathBuf::from("./skills")`
- [`src/commands/skills.rs:429-433`](src/commands/skills.rs:429-433) - Uses `skills_manager.skills_dir`

The `.agents/skills/` directory is only used as a **temporary staging area** during installation, which is then moved to `./skills/` by [`perform_post_install_move()`](src/commands/skills.rs:514).

**No action required.**

---

### Gap 2: `switchboard validate` Command - Skills Validation Integration

**Status**: ✅ Already Implemented

The validate command integrates skills validation through multiple functions in [`src/commands/validate.rs`](src/commands/validate.rs):

1. **`validate_agent_skills()`** (line 166) - Validates:
   - Empty skills list warning
   - Invalid skill format errors
   - Duplicate skill errors

2. **`validate_skills_exist_in_directory()`** (line 344) - Checks that skills declared in agent config exist in skills directory

3. **`validate_lockfile_consistency()`** (line 411) - Checks lockfile consistency

4. **Synchronization** (line 649) - `--sync` flag to synchronize lockfile with skills directory

**No action required.**

---

### Gap 3: Orphaned Skills Warning

**Status**: ⚠️ **Needs Implementation**

**Location**: [`src/commands/validate.rs:validate_lockfile_consistency()`](src/commands/validate.rs:411)

**Current Behavior**:
The function checks:
- Skills in lockfile but not in config (line 448-455)
- Skills in lockfile but not in skills directory (line 470-478)

**Missing**: Skills in skills directory but NOT in lockfile

**Impact**: If a skill exists in `./skills/` but is not tracked in `skills.lock.json`, there's no warning. This can lead to:
- Skills being used but not tracked in version control
- Stale skills that were removed from config but not deleted
- Inconsistent state between filesystem and lockfile

#### Implementation Steps

1. **Add new validation function** in [`src/commands/validate.rs`](src/commands/validate.rs):

```rust
/// Validate that skills in the skills directory are tracked in the lockfile.
///
/// This checks for "orphaned" skills - skills that exist in the skills directory
/// but are not referenced in the lockfile. These may be manually added skills
/// or skills from failed installations.
///
/// # Arguments
///
/// * `skills_dir` - Path to the skills directory
/// * `lockfile` - The parsed lockfile (if it exists)
///
/// # Returns
///
/// * `Vec<String>` - Warning messages for orphaned skills
pub(crate) fn validate_orphaned_skills(
    skills_dir: &Path,
    lockfile: &Option<LockfileStruct>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    
    // If skills directory doesn't exist, nothing to check
    if !skills_dir.exists() {
        return warnings;
    }
    
    // Scan skills directory
    let scan_result = scan_skill_directory(skills_dir);
    let (installed_skills, _) = match scan_result {
        Ok(result) => result,
        Err(_) => return warnings, // Can't read directory, skip
    };
    
    // Get skills from lockfile if it exists
    let lockfile_skill_names: HashSet<String> = match lockfile {
        Some(lf) => lf.skills.keys().cloned().collect(),
        None => HashSet::new(),
    };
    
    // Check each installed skill
    for skill_metadata in installed_skills {
        if !lockfile_skill_names.contains(&skill_metadata.name) {
            warnings.push(format!(
                "Warning: Skill '{}' exists in {}/ but is not tracked in skills.lock.json. \
                 This skill may have been manually added or the lockfile may be out of sync. \
                 Run 'switchboard validate --sync' to update the lockfile.",
                skill_metadata.name,
                skills_dir.display()
            ));
        }
    }
    
    warnings
}
```

2. **Update `ValidateCommand::run()`** to call the new function:

Add at line 647 (after lockfile consistency check):

```rust
// Validate for orphaned skills in directory but not in lockfile
let orphaned_warnings = validate_orphaned_skills(&skills_dir, &lockfile);
for warning in orphaned_warnings {
    println!("  {}", warning);
}
```

3. **Add import** for `LockfileStruct`:

```rust
use crate::skills::{..., LockfileStruct, ...};
```

**Testing Approach**:

1. **Unit test**: Add test in [`src/commands/validate.rs`](src/commands/validate.rs) test module:
   - Create temp skills directory with a skill
   - Create lockfile WITHOUT that skill
   - Verify warning is produced

2. **Integration test**:
   - Run `switchboard validate` on current project (which has `.agents/skills/test-skill/` not in lockfile)
   - Verify warning is produced

---

### Gap 4: Post-Install Cleanup

**Status**: ✅ Already Implemented

The [`cleanup_agents_directory()`](src/commands/skills.rs:584) function handles cleanup:

- Removes `.agents/skills/` if empty (line 590-594)
- Removes `.agents/` if empty (line 598-604)

However, there is a **test artifact** remaining: `.agents/skills/test-skill/`. This appears to be from manual testing and should be manually cleaned up.

**No code changes required.** Manual cleanup recommended:
```bash
rm -rf .agents/
```

---

## Discord Agent Gaps

### Gap 5: Message Chunking

**Status**: ✅ Already Implemented

The [`send_message_chunked()`](src/discord/api.rs:105) function handles Discord's 2000-character limit:

- Splits on paragraph boundaries (`\n\n`) first (line 139)
- Falls back to splitting on newlines (`\n`) (line 197)
- Last resort: splits at character boundaries with continuation marker (line 164)
- Adds 250ms delay between chunks to maintain order (line 129)

Used in [`src/discord/mod.rs:813`](src/discord/mod.rs:813):
```rust
api_client.send_message_chunked(&channel_id, &response_text).await
```

**No action required.**

---

### Gap 6: Tool Iteration Limit

**Status**: ✅ Already Implemented

[`MAX_TOOL_ITERATIONS`](src/discord/llm.rs:40) is set to 10:

```rust
const MAX_TOOL_ITERATIONS: usize = 10;
```

This is enforced in [`process_with_tools()`](src/discord/llm.rs) which stops the tool-use loop after 10 iterations to prevent infinite loops.

**No action required.**

---

### Gap 7: System Prompt File

**Status**: ✅ Already Implemented

The system prompt file is fully wired:

1. **Loading**: [`load_system_prompt()`](src/discord/config.rs:29) reads from file or returns default
2. **Passed to BotState**: [`discord/mod.rs:338-339`](src/discord/mod.rs:338-339)
3. **Used in conversation**: [`discord/mod.rs:785`](src/discord/mod.rs:785)
   ```rust
   let messages = conv_manager.get_messages_for_llm(&user_id_str, &state_guard.system_prompt)
   ```

**No action required.**

---

### Gap 8: Resume Connection Handling

**Status**: ✅ Already Implemented

The Discord gateway handles resume/reconnection:

1. **Event handling**: [`Event::Resumed`](src/discord/gateway.rs:267) is handled
2. **twilight-gateway library** handles automatic reconnection with backoff
3. **Connection state tracking**: [`ConnectionState::Reconnecting`](src/discord/gateway.rs:48) exists
4. **Ready event**: Session ID is captured for potential resume ([`discord/mod.rs:517-527`](src/discord/mod.rs:517-527))

**No action required.**

---

## Execution Order

Based on the analysis, here's the recommended execution order:

### Priority 1: Implement Orphaned Skills Warning
This is the only remaining functional gap that requires code changes.

1. Add `validate_orphaned_skills()` function to [`src/commands/validate.rs`](src/commands/validate.rs)
2. Update `ValidateCommand::run()` to call the new function
3. Add unit tests
4. Run integration test on current project

### Priority 2: Manual Cleanup
Clean up test artifacts:
```bash
rm -rf .agents/skills/test-skill/
# Verify .agents/ is empty or remove entirely
```

---

## Files Modified

| File | Change Type | Description |
|------|-------------|-------------|
| [`src/commands/validate.rs`](src/commands/validate.rs) | Modify | Add orphaned skills validation |
| `.agents/` | Manual | Clean up test artifacts |

---

## Testing Strategy

### For Orphaned Skills Warning:

1. **Unit Tests** (in [`src/commands/validate.rs`](src/commands/validate.rs)):
   ```rust
   #[test]
   fn test_orphaned_skills_warning() {
       // Create temp dir with skill but no lockfile entry
       // Verify warning is produced
   }
   
   #[test]
   fn test_no_warning_when_lockfile_matches() {
       // Create lockfile with matching skill
       // Verify no warning
   }
   ```

2. **Integration Test**:
   ```bash
   # Current state: .agents/skills/test-skill/ exists but not in lockfile
   switchboard validate
   # Should show warning about orphaned skill
   ```

---

## Appendix: Current Project State

### Skills Directory (`./skills/`):
- `DISCLI.md`, `README.md`
- `skills.lock.json` - contains: `repo`, `repo1`, `skill-name`
- `repo/`, `repo1/`, `skill-name/` - each with `SKILL.md`

### Staging Directory (`.agents/`):
- `.agents/skills/test-skill/` - orphaned test artifact (should be cleaned)

### Lockfile (`skills/skills.lock.json`):
```json
{
  "skills": {
    "repo1": { "source": "local/repo1", ... },
    "skill-name": { "source": "local/skill-name", ... },
    "repo": { "source": "local/repo", ... }
  }
}
```
