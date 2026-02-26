# Skills Feature - Implementation Plan for Remaining Work

## Overview

This document outlines the remaining work (~5-10%) to complete the skills feature implementation. Based on analysis of the codebase, the implementation is largely complete with three remaining areas requiring attention.

---

## Current State Analysis

### 1. Lockfile Population
- **Location**: [`skills/skills.lock.json`](../../skills/skills.lock.json)
- **Current State**: Empty `{"version":1,"skills":{}}`
- **Skills Present**: `repo`, `repo1`, `skill-name` (in [`skills/`](../../skills/) directory)
- **Infrastructure**: Lockfile functions exist in [`src/skills/mod.rs`](../../src/skills/mod.rs):
  - `load_lockfile()` (line 622)
  - `save_lockfile()` (line 678)
  - `add_skill_to_lockfile()` (line 888)
  - `read_lockfile()` (line 764)
  - `write_lockfile()` (line 835)

### 2. Container Runtime Skill Mounting
- **Location**: [`src/docker/run/run.rs`](../../src/docker/run/run.rs)
- **Function**: `build_host_config()` (lines 232-333)
- **Implementation Status**: ✅ COMPLETE
  - Global skills mount: `./skills` → `/workspace/.kilocode/skills` (lines 252-271)
  - Individual skill mounts: `./skills/<skill-name>/` → `/workspace/skills/<skill-name>/` (lines 273-326)

### 3. Per-Agent Skill Scoping
- **Location**: [`src/docker/run/run.rs`](../../src/docker/run/run.rs)
- **Functions**:
  - `run_agent()` (line 950) - passes skills to container config
  - `validate_skills_exist()` (line 364) - validates skills before container creation
  - `build_container_config()` (line 654) - builds container with skill mounts
- **Implementation Status**: ✅ COMPLETE

---

## Remaining Tasks

### Task 1: Lockfile Population

#### Problem
Skills exist in the `./skills/` directory but are not recorded in the lockfile. The lockfile should be populated when:
1. Skills are manually added to `./skills/`
2. Skills are used in agent configurations

#### Implementation Approach

**Option A: On-Demand Population (Recommended)**
Create a function to sync skills from `./skills/` directory to the lockfile.

**Function to implement**: `sync_skills_to_lockfile()`

Location: [`src/skills/mod.rs`](../../src/skills/mod.rs) - add near line 950

```rust
/// Syncs skills from the skills directory to the lockfile.
///
/// This function scans the ./skills/ directory for skill directories
/// containing SKILL.md files and adds them to the lockfile if not present.
///
/// # Arguments
///
/// * `directory` - The skills directory path
///
/// # Returns
///
/// * `Ok(u32)` - Number of skills synced
/// * `Err(SkillsError)` - If there's an error
pub fn sync_skills_to_lockfile(directory: &Path) -> Result<u32, SkillsError> {
    // 1. Scan the skills directory
    let (skills, warnings) = scan_skill_directory(directory)?;
    
    // 2. Load or create lockfile
    let mut lockfile = match read_lockfile(directory) {
        Ok(lf) => lf,
        Err(SkillsError::LockfileNotFound { .. }) => default_lockfile(),
        Err(e) => return Err(e),
    };
    
    // 3. Add each skill to lockfile if not present
    let mut synced = 0u32;
    for skill in skills {
        if !lockfile.skills.contains_key(&skill.name) {
            let entry = SkillLockEntry {
                skill_name: skill.name.clone(),
                source: skill.source.unwrap_or_else(|| format!("local:{}", skill.name)),
                installed_at: Utc::now().to_rfc3339(),
            };
            lockfile.skills.insert(skill.name, entry);
            synced += 1;
        }
    }
    
    // 4. Write the updated lockfile
    if synced > 0 {
        write_lockfile(&lockfile, directory)?;
    }
    
    Ok(synced)
}
```

**Integration Points**:

1. **On startup/scheduler init** - Call in [`src/scheduler/mod.rs`](../../src/scheduler/mod.rs)
2. **On validate command** - Already has `validate_lockfile_consistency()` in [`src/commands/validate.rs`](../../src/commands/validate.rs:393)
3. **On container run** - Optionally sync before container execution

#### Affected Files
- [`src/skills/mod.rs`](../../src/skills/mod.rs) - Add `sync_skills_to_lockfile()` function

---

### Task 2: Container Runtime Skill Mounting Verification

#### Current Implementation (✅ Complete)

The [`build_host_config()`](../../src/docker/run/run.rs:232) function already implements bind-mounts:

```rust
// Global skills directory mount (lines 252-271)
let skills_dir = std::path::Path::new(workspace).join("skills");
if skills_dir.exists() && skills_dir.is_dir() {
    binds.push(format!(
        "{}:/workspace/.kilocode/skills:ro",
        skills_docker_path
    ));
}

// Individual skill mounts (lines 273-326)
for skill in skills_list {
    // Parse skill name and mount ./skills/<skill-name>/ to /workspace/skills/<skill-name>/
}
```

#### Verification Steps

1. **Code Review**: ✅ Already reviewed - implementation is correct
2. **Unit Test Verification**: Run existing tests
   ```bash
   cargo test --lib build_host_config
   cargo test --lib validate_skills_exist
   ```
3. **Integration Test**: Create a test that verifies bind mounts are created correctly

#### No Code Changes Required
The container runtime skill mounting is already fully implemented.

---

### Task 3: Per-Agent Skill Scoping Verification

#### Current Implementation (✅ Complete)

The [`run_agent()`](../../src/docker/run/run.rs:950) function:

1. **Validates skills exist** (line 967):
   ```rust
   validate_skills_exist(workspace, config.skills.as_deref())?;
   ```

2. **Passes skills to container config** (line 1008):
   ```rust
   skills: skills_ref,
   ```

3. **Builds host config with skill mounts** ([`build_host_config()`](../../src/docker/run/run.rs:232)):
   - Mounts individual skills based on config

#### Verification Steps

1. **Run unit tests**:
   ```bash
   cargo test --lib validate_skills_exist
   ```

2. **Verify skill validation is called before container creation**

#### No Code Changes Required
The per-agent skill scoping is already fully implemented.

---

## Implementation Roadmap

### Phase 1: Lockfile Population (Primary Task)

| Step | Action | File | Line |
|------|--------|------|------|
| 1.1 | Add `sync_skills_to_lockfile()` function | `src/skills/mod.rs` | ~950 |
| 1.2 | Add import for `Utc` if needed | `src/skills/mod.rs` | ~1 |
| 1.3 | Add unit tests for sync function | `src/skills/mod.rs` | ~2572 |
| 1.4 | Integrate with validate command | `src/commands/validate.rs` | ~630 |

### Phase 2: Verification (No Code Changes)

| Step | Action | Command |
|------|--------|---------|
| 2.1 | Run build_host_config tests | `cargo test --lib build_host_config` |
| 2.2 | Run validate_skills_exist tests | `cargo test --lib validate_skills_exist` |
| 2.3 | Run entrypoint script tests | `cargo test --lib generate_entrypoint_script` |

---

## Summary

| Task | Status | Effort |
|------|--------|--------|
| Lockfile Population | ❌ Needs Implementation | ~2-3 hours |
| Container Runtime Skill Mounting | ✅ Complete | Verification only |
| Per-Agent Skill Scoping | ✅ Complete | Verification only |

### Recommended Next Steps

1. **Implement `sync_skills_to_lockfile()` function** in `src/skills/mod.rs`
2. **Add integration with validate command** to auto-sync on validation
3. **Run existing tests** to verify container mounting works correctly
4. **Manual testing** with a running container to verify bind-mounts

---

## Appendix: Key Code Locations

| Component | File | Lines |
|-----------|------|-------|
| Lockfile structures | [`src/skills/mod.rs`](../../src/skills/mod.rs) | 950-1010 |
| Lockfile read/write | [`src/skills/mod.rs`](../../src/skills/mod.rs) | 622-710 |
| Scan skills directory | [`src/skills/mod.rs`](../../src/skills/mod.rs) | 1235-1280 |
| Build host config | [`src/docker/run/run.rs`](../../src/docker/run/run.rs) | 232-333 |
| Validate skills exist | [`src/docker/run/run.rs`](../../src/docker/run/run.rs) | 364-414 |
| Run agent | [`src/docker/run/run.rs`](../../src/docker/run/run.rs) | 950-1100 |
| Entrypoint script | [`src/docker/skills.rs`](../../src/docker/skills.rs) | 339-416 |
| Validate command | [`src/commands/validate.rs`](../../src/commands/validate.rs) | 590-640 |
