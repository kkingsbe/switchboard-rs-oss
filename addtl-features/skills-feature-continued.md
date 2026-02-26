# Feature Requirements Document: Skills.sh Integration for Switchboard

**Feature:** Skills Management CLI  
**Status:** Draft  
**Version:** 0.3.0  
**Author:** TBD  
**Last Updated:** 2026-02-22

---

## 1. Overview

### 1.1 Summary

This document defines the requirements for integrating [skills.sh](https://skills.sh) skill discovery and management into the Switchboard CLI. The feature introduces a `switchboard skills` subcommand family that allows users to browse, install, and manage Agent Skills directly from the terminal.

Skill discovery is performed by Switchboard directly via the skills.sh search API. Skill installation is delegated to the `npx skills` CLI, which handles fetching and writing skill files. After installation, Switchboard moves skills from the default `.agents/skills/` location into the project's `./skills/` directory, which is Switchboard's canonical skill storage location. A lockfile at `./skills/skills.lock.json` tracks the source origin of each installed skill.

At container runtime, skills are not re-installed from the network. Instead, the host's `./skills/` directory (or the relevant subset) is mounted into each agent's container, making skill availability instant and offline-safe.

### 1.2 Background

Agent Skills are a lightweight, open standard for extending AI agent behavior with reusable procedural knowledge. The skills.sh marketplace, created by Vercel Labs, aggregates hundreds of community-authored skills. The `npx skills` CLI provides installation tooling, while skills.sh exposes a search API for programmatic discovery.

Currently, Switchboard users must manage skills separately from their agent configuration. This creates friction and breaks the "single tool" workflow that Switchboard aims to provide.

### 1.3 Goals

- Give users a first-class `switchboard skills` CLI experience for browsing, installing, and managing skills
- Use the skills.sh search API directly for skill discovery, giving Switchboard full control over search presentation and result handling
- Delegate skill installation to `npx skills` for fetching and writing files, then move the result into `./skills/`
- Track installed skill sources in a lockfile so skills can be updated without re-specifying their origin
- Mount host-installed skills into agent containers at runtime rather than re-installing per run
- Allow skills to be declared in `switchboard.toml` and automatically provisioned
- Support per-agent skill scoping so different agents can receive different skill sets
- Remain compatible with the open Agent Skills specification so skills work portably across other agents

### 1.4 Non-Goals

- Switchboard will not host its own skills registry or marketplace
- Switchboard will not author or curate skills (users source from skills.sh or GitHub)
- Switchboard will not implement skill authoring tooling (that remains the responsibility of `npx skills init`)
- Switchboard will not require a paid skills.sh account or API key
- Switchboard will not implement its own GitHub cloning or raw file fetching — `npx skills add` is the sole mechanism for downloading skill content
- Switchboard will not provide a fallback implementation if `npx` / Node.js is unavailable; `npx` is a hard prerequisite for skill installation
- Switchboard will not pin `npx skills` to a specific version — it always uses the latest available

---

## 2. User Stories

| ID | Role | Story | Priority |
|----|------|-------|----------|
| US-01 | Developer | I want to search for skills from the terminal so I don't have to leave my workflow to browse skills.sh | High |
| US-02 | Developer | I want to install a skill into my project with one command so it's immediately available to my agents | High |
| US-03 | Developer | I want to see which skills are currently installed in my project at a glance | High |
| US-04 | Developer | I want to remove a skill I no longer need | Medium |
| US-05 | Developer | I want to declare required skills per agent in `switchboard.toml` so each agent automatically has the right skills when it runs | High |
| US-06 | Developer | I want different agents to use different skills so I can give each agent focused, relevant context | Medium |
| US-07 | Developer | I want skills to be updated to their latest versions on demand | Low |
| US-08 | CI/CD | I want skills to be available inside containers without network access at runtime | High |

---

## 3. Functional Requirements

### 3.1 Skill Discovery: skills.sh Search API

Switchboard queries the skills.sh search API directly rather than delegating discovery to `npx skills find`. This gives Switchboard full control over how results are presented, filtered, and acted upon.

#### 3.1.1 API Endpoint

```
GET https://skills.sh/api/search?q={query}&limit={limit}
```

#### 3.1.2 Parameters

| Parameter | Type   | Required | Default | Description                     |
|-----------|--------|----------|---------|---------------------------------|
| `q`       | string | Yes      | —       | Search query (min 2 characters) |
| `limit`   | number | No       | 10      | Max results to return (up to 50)|

#### 3.1.3 Response Shape

```json
{
  "skills": [
    {
      "id": "vercel-labs/agent-skills/vercel-react-best-practices",
      "name": "vercel-react-best-practices",
      "installs": 152900,
      "source": "vercel-labs/agent-skills"
    }
  ]
}
```

#### 3.1.4 Response Fields

| Field      | Type   | Description                                              |
|------------|--------|----------------------------------------------------------|
| `id`       | string | Unique slug in format `{owner}/{repo}/{skill-name}`      |
| `name`     | string | Skill name (used for installation and folder naming)     |
| `installs` | number | Total install count                                      |
| `source`   | string | GitHub `{owner}/{repo}` that contains the skill          |

---

### 3.2 Skill Lockfile: `./skills/skills.lock.json`

Switchboard maintains a lockfile that tracks the source origin of each installed skill. This enables the `update` command to re-install skills without requiring the user to re-specify the source.

#### 3.2.1 Location

```
./skills/skills.lock.json
```

#### 3.2.2 Schema

```json
{
  "version": 1,
  "skills": {
    "frontend-design": {
      "source": "vercel-labs/agent-skills",
      "name": "frontend-design",
      "installed_at": "2026-02-22T14:30:00Z"
    },
    "security-audit": {
      "source": "anthropics/skills",
      "name": "security-audit",
      "installed_at": "2026-02-22T14:31:00Z"
    }
  }
}
```

#### 3.2.3 Behavior

- The lockfile is created on first skill install if it does not exist
- Each `switchboard skills install` adds or updates the entry for the installed skill(s)
- Each `switchboard skills remove` removes the entry for the removed skill
- The lockfile is the source of truth for `switchboard skills update` when determining where to fetch each skill from
- The lockfile should be committed to version control so teammates and CI share the same skill sources

---

### 3.3 `switchboard skills` Subcommand Family

#### 3.3.1 `switchboard skills list`

Search for available skills from skills.sh.

**Behavior:**
- Accepts a required `--search <query>` flag (or positional query argument)
- Queries `https://skills.sh/api/search?q={query}&limit=50` directly via HTTP
- Parses the JSON response and renders results in a formatted table showing name, source, and install count
- Results are presented by Switchboard's own rendering — not delegated to `npx skills find`
- Query must be at least 2 characters; shorter queries are rejected with a user-facing message

**Prerequisites:**
- Network access to `skills.sh` (no `npx` or Node.js required for search)

**Acceptance Criteria:**
- `switchboard skills list --search react` queries the API and displays formatted results
- `switchboard skills list react` (positional shorthand) behaves identically
- Queries shorter than 2 characters are rejected with an informative message
- API errors or network failures produce a clear error message
- No dependency on `npx` or Node.js for this command

**Example Output:**
```
$ switchboard skills list --search react

  Skills matching "react" (showing top 10 of 47)

  #   Name                              Source                        Installs
  ──────────────────────────────────────────────────────────────────────────────
  1   vercel-react-best-practices       vercel-labs/agent-skills       152.9K
  2   vercel-composition-patterns       vercel-labs/agent-skills        50.3K
  3   vercel-react-native-skills        vercel-labs/agent-skills        35.6K
  4   react-doctor                      millionco/react-doctor           4.5K
  5   react-native-best-practices       callstackincubator/agent-skills  5.8K
  ...

  Install a skill: switchboard skills install vercel-labs/agent-skills@vercel-react-best-practices
```

---

#### 3.3.2 `switchboard skills install`

Install a skill into the project's `./skills/` directory.

**Behavior:**
- Accepts a source in `owner/repo@skill-name` format (single skill) or `owner/repo` format (all skills from repo)
- Delegates installation to `npx skills add <source> -y`, which installs to the default `.agents/skills/` directory
- After `npx skills add` completes successfully, Switchboard moves the installed skill folder(s) from `.agents/skills/<skill-name>/` to `./skills/<skill-name>/`
- Creates `./skills/` if it does not exist
- Verifies `./skills/<skill-name>/SKILL.md` exists after the move
- Updates `./skills/skills.lock.json` with the source and install timestamp for each installed skill
- Cleans up `.agents/skills/` after a successful move (removes the moved directory; removes `.agents/skills/` and `.agents/` if they are now empty)
- If the destination `./skills/<skill-name>/` already exists, prompts the user to overwrite or skip (bypass with `--yes`)
- Forwards stdout/stderr from `npx skills add` to the terminal during the installation phase

**Post-Install Sequence:**
1. Check `npx` availability
2. Run `npx skills add <source> -y`
3. Identify installed skill folder(s) in `.agents/skills/`
4. Create `./skills/` if needed
5. Move each skill folder from `.agents/skills/<n>/` to `./skills/<n>/`
6. Verify `SKILL.md` exists in each destination
7. Update `./skills/skills.lock.json`
8. Remove empty `.agents/skills/` and `.agents/` directories

**Prerequisites:**
- `npx` and Node.js must be available on the host

**Acceptance Criteria:**
- `switchboard skills install vercel-labs/agent-skills@frontend-design` runs `npx skills add vercel-labs/agent-skills --skill frontend-design -y` then moves the result to `./skills/frontend-design/`
- `switchboard skills install vercel-labs/agent-skills` installs all skills from the repo then moves each to `./skills/`
- After a successful install, `./skills/<skill-name>/SKILL.md` exists and `skills.lock.json` is updated
- `.agents/skills/` is cleaned up after a successful move
- If `./skills/<skill-name>/` already exists, user is prompted (unless `--yes` is passed)
- If `npx` is not found, a clear error is shown directing the user to install Node.js
- Exit code from `npx skills add` is forwarded on failure; Switchboard's own exit code reflects move failures separately

**Example Output:**
```
$ switchboard skills install vercel-labs/agent-skills@frontend-design

Installing frontend-design from vercel-labs/agent-skills...
[npx skills add output]
Moved to ./skills/frontend-design/
Updated skills.lock.json
Done.
```

---

#### 3.3.3 `switchboard skills installed`

List all skills currently installed in the project.

**Behavior:**
- Scans `./skills/` for directories containing a `SKILL.md` file
- Parses the YAML frontmatter from each `SKILL.md` to extract name and description
- Cross-references `./skills/skills.lock.json` to show the source for each skill
- Indicates which agents (if any) have the skill explicitly assigned via `switchboard.toml`

**Acceptance Criteria:**
- Output lists all installed skills with name, description, source, and assigned agents
- Skills present on disk but missing from the lockfile are listed with a warning
- Skills with malformed or missing frontmatter are listed with a warning rather than silently omitted
- Empty state message is shown if no skills are installed

**Example Output:**
```
$ switchboard skills installed

  Installed Skills (./skills/)
  ──────────────────────────────────────────────────────────────────────────────
  frontend-design          High-quality UI/UX design      vercel-labs/agent-skills   [all agents]
  security-audit           Security vulnerability scan    anthropics/skills          [security-scan]

  2 skills installed
```

---

#### 3.3.4 `switchboard skills remove`

Remove an installed skill from the project.

**Behavior:**
- Accepts the skill name as a positional argument
- Removes `./skills/<n>/` directory
- Removes the skill's entry from `./skills/skills.lock.json`
- Prompts for confirmation before deletion (bypass with `--yes`)

**Acceptance Criteria:**
- `switchboard skills remove frontend-design` removes `./skills/frontend-design/` and its lockfile entry after confirmation
- Attempting to remove a skill that isn't installed returns a clear error
- If the skill is referenced in `switchboard.toml`, a warning is printed but removal proceeds

---

#### 3.3.5 `switchboard skills update`

Update installed skills to their latest versions.

**Behavior:**
- Reads `./skills/skills.lock.json` to determine the source for each installed skill
- For each skill (or a single named skill), re-runs the install flow: `npx skills add <source> --skill <n> -y` followed by the move step, overwriting the existing skill directory
- Updates the `installed_at` timestamp in the lockfile
- Accepts an optional skill name argument to update a single skill
- Forwards stdout/stderr from `npx skills add` to the terminal

**Acceptance Criteria:**
- `switchboard skills update` re-installs all skills listed in `skills.lock.json`
- `switchboard skills update frontend-design` re-installs only that skill using its lockfile source
- A skill present on disk but missing from the lockfile cannot be updated; an error directs the user to re-install it
- Exit code reflects whether any update failed
- If `npx` is not found, a clear error is shown directing the user to install Node.js

---

### 3.4 `switchboard.toml` Config: Per-Agent Skill Declaration

#### 3.4.1 Per-Agent Skill Declaration

The `[[agent]]` section gains an optional `skills` field to declare which skills are available to a given agent:

```toml
[[agent]]
name = "security-scan"
schedule = "0 2 * * *"
prompt = "Scan for security vulnerabilities."
skills = ["security-audit"]

[[agent]]
name = "ui-reviewer"
schedule = "0 9 * * 1"
prompt = "Review UI components."
skills = ["frontend-design", "vercel-react-best-practices"]
```

If `skills` is omitted, the agent receives no skills. Skills must be explicitly declared per agent. Entries reference skill names (matching directory names in `./skills/`), not full `owner/repo` sources.

**Acceptance Criteria:**
- `skills` field in `[[agent]]` accepts a list of skill names corresponding to directories in `./skills/`
- `switchboard validate` warns if an agent references a skill name that does not exist in `./skills/`
- `switchboard validate` warns if an agent has a `skills` field that is present but empty
- `switchboard validate` reports an error if the same skill name appears more than once in a single agent's list

---

### 3.5 `switchboard build` — No Skills Involvement

`switchboard build` is responsible only for building the base Docker image. It has no involvement in skill installation and requires no knowledge of declared skills.

**Behavior:**
- `switchboard build` builds the Docker image as before, with no skill-related steps
- No `skills` configuration is read or validated during `switchboard build`

---

### 3.6 Container Execution: Skill Mounting at Runtime

Skills are installed once on the host via `switchboard skills install` and mounted into each agent's container at runtime. No network access or `npx` invocation is needed inside the container for skills.

**Behavior:**
- When launching a container for an agent with a non-empty `skills` list, Switchboard bind-mounts the relevant skill directories from the host into the container
- For each skill name in the agent's `skills` list, the host path `./skills/<skill-name>/` is mounted read-only into the container at the agent's expected skill location (e.g. `./skills/<skill-name>/` inside the container's working directory)
- If a declared skill does not exist in `./skills/` on the host at launch time, the container launch fails immediately with a clear error indicating which skill is missing
- Agents with no `skills` field receive no skill mounts and launch normally
- Because skills are mounted rather than installed at runtime, containers do not require `npx`, Node.js, or network access for skill availability

**Mount Pattern:**

For an agent declared as:
```toml
[[agent]]
name = "ui-reviewer"
skills = ["frontend-design", "vercel-react-best-practices"]
```

Switchboard adds these bind mounts when creating the container:
```
./skills/frontend-design/           → <container-workdir>/skills/frontend-design/           (read-only)
./skills/vercel-react-best-practices/ → <container-workdir>/skills/vercel-react-best-practices/ (read-only)
```

**Acceptance Criteria:**
- Declared skills are bind-mounted from the host into the container at launch
- Mounts are read-only
- A missing skill on the host causes an immediate, clear launch failure — the container is not started
- Agents with no `skills` field launch with no skill mounts
- No `npx`, Node.js, or network access is required inside the container for skills
- Container startup time is not impacted by skill installation (mounts are near-instant)

---

### 3.7 `switchboard validate` Updates

The existing `validate` command is extended to check skill-related configuration:

- Warns if an `[[agent]]` has a `skills` field that is present but empty
- Reports an error if a declared skill name does not have a corresponding directory in `./skills/`
- Reports an error if the same skill name appears more than once in a single agent's `skills` list
- Warns if a skill directory exists in `./skills/` but is missing from `skills.lock.json`

---

## 4. Non-Functional Requirements

### 4.1 Performance

- `switchboard skills list` must return results within 3 seconds under normal network conditions (API call only, no `npx` involved)
- Container startup is not impacted by skill provisioning since skills are bind-mounted from the host
- Skill installation via `npx skills add` should complete within 15 seconds per skill on a standard broadband connection

### 4.2 Reliability

- `switchboard skills list` degrades gracefully when the network is unavailable, with a clear offline error message
- `switchboard skills install` and `switchboard skills update` degrade gracefully when the network or `npx` is unavailable
- A missing skill at container launch time causes an immediate, unambiguous failure before any agent work begins
- The lockfile is treated as the source of truth for skill origins; if it becomes inconsistent with the filesystem, `switchboard validate` reports the discrepancy

### 4.3 Security

- Switchboard must not execute any code contained within a skill's `SKILL.md` — skills are data, not executable packages
- Users are warned that skills are community-authored and to review `SKILL.md` contents before installing
- No credentials or API keys should be written into skill files or transmitted to skills.sh
- Skills are mounted read-only into containers to prevent agents from modifying skill content

### 4.4 Compatibility

- Skills must conform to the open [Agent Skills specification](https://github.com/vercel-labs/skills) so they remain portable to other agents
- The feature must not break existing Switchboard projects that manually manage skills outside of `switchboard.toml`
- Skills installed into `./skills/` must be loadable by any agent that supports the standard skill directory convention

### 4.5 Dependency Management

- `npx` and Node.js are required on the **host** for `switchboard skills install` and `switchboard skills update` only
- `switchboard skills list` uses the skills.sh API directly and has no `npx` dependency
- `switchboard skills installed`, `switchboard skills remove`, and container execution have no `npx` dependency — they operate on local files and mounts only
- Switchboard must check for `npx` availability at the start of commands that require it and fail fast with: `Error: npx is required for this command. Install Node.js from https://nodejs.org`
- Containers do **not** require `npx` or Node.js for skill availability (skills are bind-mounted)
- No new Rust-level network dependencies are introduced beyond what is needed for the skills.sh API HTTP call (which may use the existing HTTP client)

---

## 5. Technical Design Notes

### 5.1 New Module: `src/skills/mod.rs`

Introduces a `SkillsManager` struct responsible for:

- Making HTTP requests to the skills.sh search API and parsing JSON responses
- Invoking `npx skills add` as a child process and forwarding its output
- Performing the post-install move from `.agents/skills/` to `./skills/`
- Managing `./skills/skills.lock.json` (read, write, update, remove entries)
- Checking `npx` availability on the host and returning a structured error if absent
- Reading and parsing `SKILL.md` YAML frontmatter for the `installed` command
- Removing skill directories and lockfile entries for the `remove` command
- Cleaning up empty `.agents/` directories after moves

The module contains a single HTTP integration point (the skills.sh search API). All skill fetching and file writing is delegated to `npx skills add`.

### 5.2 Skill Installation Flow

```
switchboard skills install owner/repo@skill-name
    │
    ├─ 1. Check npx availability
    │
    ├─ 2. Run: npx skills add owner/repo --skill skill-name -y
    │      └─ Installs to .agents/skills/skill-name/
    │
    ├─ 3. Create ./skills/ if needed
    │
    ├─ 4. Move .agents/skills/skill-name/ → ./skills/skill-name/
    │
    ├─ 5. Verify ./skills/skill-name/SKILL.md exists
    │
    ├─ 6. Update ./skills/skills.lock.json
    │
    └─ 7. Clean up empty .agents/skills/ and .agents/
```

### 5.3 `npx` Process Invocation Pattern

All delegated commands follow this pattern in Rust:

```rust
fn run_npx_skills(args: &[&str]) -> Result<ExitStatus> {
    let status = std::process::Command::new("npx")
        .args(["skills"].iter().chain(args.iter()))
        .status()
        .context("Failed to invoke npx. Is Node.js installed?")?;
    Ok(status)
}
```

stdout and stderr are inherited from the parent process so `npx skills` output renders directly in the user's terminal.

### 5.4 Config Schema Additions

`src/config/mod.rs` is extended with:

- `Vec<String>` field on `AgentConfig` for the per-agent `skills` list
- Entries are skill names (matching `./skills/` directory names), not full `owner/repo` sources

### 5.5 Container Skill Mounting

In `src/docker/mod.rs`, when an agent has a non-empty `skills` field, Switchboard adds bind mounts for each declared skill:

```rust
for skill_name in &agent.skills {
    let host_path = format!("./skills/{}", skill_name);
    let container_path = format!("{}/skills/{}", container_workdir, skill_name);
    // Add read-only bind mount: host_path → container_path
}
```

If any declared skill directory does not exist on the host, container creation is aborted before any mounts are attempted.

---

## 6. Error Handling

| Scenario | Behavior |
|----------|----------|
| `npx` not found on host | Immediate failure: `Error: npx is required for this command. Install Node.js from https://nodejs.org` |
| skills.sh API unreachable | `switchboard skills list` fails: `Error: Could not reach skills.sh. Check your network connection.` |
| skills.sh API returns empty results | Display: `No skills found matching "{query}"` |
| `npx skills add` exits with non-zero | Forward exit code and stderr to user |
| Skill not found in repo | `npx skills` handles and reports this; Switchboard forwards the message |
| Post-install move fails | Report which skill failed to move and the filesystem error; exit non-zero |
| `./skills/<n>/SKILL.md` missing after move | Warn that installation may be incomplete |
| Destination `./skills/<n>/` already exists | Prompt user to overwrite or skip (bypass with `--yes`) |
| Malformed `SKILL.md` frontmatter (during `installed`) | Warn and list the skill with degraded info; do not crash |
| Skill declared in `switchboard.toml` missing from `./skills/` | Container launch aborted with clear error naming the missing skill |
| Skill on disk but missing from lockfile | `switchboard validate` warns; `switchboard skills update` cannot update it (directs user to re-install) |
| Agent has empty `skills = []` field | `switchboard validate` warns; container runs with no skill mounts |
| Duplicate skill name in agent's `skills` list | `switchboard validate` reports an error |
| Lockfile missing or corrupt | Switchboard recreates it from current `./skills/` contents where possible, warns about missing source info |

---

## 7. Out of Scope for v0.3

- Skill version pinning (install a skill at a specific git SHA)
- Private/internal skills hosted outside of public GitHub
- A `switchboard skills publish` command for submitting skills to the marketplace
- Skill dependency resolution (skills that depend on other skills)
- Web UI dashboard showing installed skills and agent assignments
- A native Rust fallback for environments where Node.js is unavailable
- Global skill installation — all skills are project-level

---

## 8. Acceptance Criteria Summary

| ID | Criteria | Priority |
|----|----------|----------|
| AC-01 | `switchboard skills list --search <query>` queries skills.sh API directly and renders formatted results | High |
| AC-02 | `switchboard skills list` requires no `npx` or Node.js dependency | High |
| AC-03 | `switchboard skills install <source>` runs `npx skills add` then moves the result to `./skills/` | High |
| AC-04 | After install, `./skills/<n>/SKILL.md` exists, `.agents/skills/` is cleaned up, and `skills.lock.json` is updated | High |
| AC-05 | `switchboard skills installed` lists installed skills by scanning `./skills/` and cross-referencing the lockfile | High |
| AC-06 | `switchboard skills remove <n>` removes `./skills/<n>/` and its lockfile entry after confirmation | Medium |
| AC-07 | `switchboard skills update` re-installs skills using sources from `skills.lock.json` | Low |
| AC-08 | Per-agent `skills = [...]` in `[[agent]]` references skill names in `./skills/` | High |
| AC-09 | Declared skills are bind-mounted read-only from the host into the container at launch | High |
| AC-10 | A missing skill at container launch time aborts the run with a clear error | High |
| AC-11 | `switchboard validate` checks that declared skills exist in `./skills/` and reports lockfile inconsistencies | High |
| AC-12 | Commands requiring `npx` fail fast with a clear prerequisite error if `npx` is not found | High |
| AC-13 | Existing `./skills/<n>/` prompts for overwrite unless `--yes` is passed | Medium |
| AC-14 | Containers require no `npx`, Node.js, or network access for skill availability | High |
| AC-15 | A failed skill install always aborts — there is no optional/soft-fail mode | High |