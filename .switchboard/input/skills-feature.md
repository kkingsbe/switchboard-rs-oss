# Feature Requirements Document: Skills.sh Integration for Switchboard

**Feature:** Skills Management CLI  
**Status:** Draft  
**Version:** 0.1.0  
**Author:** TBD  
**Last Updated:** 2026-02-19

---

## 1. Overview

### 1.1 Summary

This document defines the requirements for integrating [skills.sh](https://skills.sh) skill discovery and management into the Switchboard CLI. The feature introduces a `switchboard skills` subcommand family that allows users to browse, install, and manage Agent Skills directly from the terminal — without needing to separately invoke `npx skills` or manually place `SKILL.md` files into `.kilocode/skills/`.

All skill discovery and installation operations are delegated exclusively to the `npx skills` CLI. Switchboard does not implement its own GitHub API integration, web scraping, or skill fetching logic — it acts as a thin, ergonomic wrapper around `npx skills` that integrates the workflow into `switchboard.toml` and the existing CLI command family.

### 1.2 Background

Agent Skills are a lightweight, open standard for extending AI agent behavior with reusable procedural knowledge. Kilo Code CLI (the agent runner at the core of Switchboard) natively supports skills placed in `.kilocode/skills/`. The skills.sh marketplace, created by Vercel Labs, aggregates hundreds of community-authored skills installable via the `npx skills` CLI.

Currently, Switchboard users must manage skills separately from their agent configuration — running `npx skills add` on the side, then reloading their environment. This creates friction and breaks the "single tool" workflow that Switchboard aims to provide.

### 1.3 Goals

- Give users a first-class `switchboard skills` CLI experience for browsing, installing, and managing skills
- Delegate all skill discovery and installation exclusively to `npx skills` — Switchboard is a thin wrapper, not a reimplementation
- Allow skills to be declared in `switchboard.toml` and automatically provisioned at build time
- Support per-agent skill scoping so different agents can receive different skill sets
- Remain compatible with the open Agent Skills specification so skills work portably across other agents

### 1.4 Non-Goals

- Switchboard will not host its own skills registry or marketplace
- Switchboard will not author or curate skills (users source from skills.sh or GitHub)
- Switchboard will not implement skill authoring tooling (that remains the responsibility of `npx skills init`)
- Switchboard will not require a paid skills.sh account or API key
- Switchboard will not implement its own skill fetching, GitHub API integration, or web scraping — `npx skills` is the sole mechanism for all remote skill operations
- Switchboard will not provide a fallback implementation if `npx` / Node.js is unavailable; `npx` is a hard prerequisite for any remote skill operation

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
| US-08 | CI/CD | I want skills to be installed automatically inside each agent container at startup so pipelines require no host-side skill management | High |

---

## 3. Functional Requirements

### 3.1 `switchboard skills` Subcommand Family

A new top-level `skills` subcommand is introduced with the following sub-subcommands:

#### 3.1.1 `switchboard skills list`

Browse and search available skills from skills.sh.

**Behavior:**
- Delegates to `npx skills find` under the hood
- Without arguments, launches the interactive `fzf`-style skill browser provided by `npx skills find`
- Accepts an optional `--search <query>` flag which passes the query to `npx skills find <query>` for non-interactive filtering
- Raw output from `npx skills find` is passed through to the user's terminal unchanged, preserving formatting and interactivity
- No caching, result parsing, or post-processing is performed by Switchboard

**Prerequisites:**
- `npx` and Node.js must be available on the host (see Section 4.5)

**Acceptance Criteria:**
- `switchboard skills list` invokes `npx skills find` and surfaces its interactive UI
- `switchboard skills list --search react` invokes `npx skills find react` and passes output through
- If `npx` is not found, a clear error is shown directing the user to install Node.js
- Exit code from `npx skills find` is forwarded as Switchboard's exit code

**Example Output:**
```
$ switchboard skills list --search react

[output from npx skills find react]
```

---

#### 3.1.2 `switchboard skills install`

Install a skill into the project's `.kilocode/skills/` directory.

**Behavior:**
- Accepts a source in any format supported by `npx skills add`:
  - `owner/repo` — installs all skills from the repo
  - `owner/repo@skill-name` — installs a specific skill from the repo
  - A full GitHub or GitLab URL
- Delegates entirely to `npx skills add <source> -a kilo -y`, which handles fetching, writing files, and placing them in the correct `.kilocode/skills/` location
- Switchboard does not post-process or validate the installed files — it trusts `npx skills` to handle this correctly
- Accepts `--global` to pass `-g` to `npx skills add`, installing into `~/.kilocode/skills/` instead of the project directory
- Forwards stdout/stderr from `npx skills add` to the terminal

**Prerequisites:**
- `npx` and Node.js must be available on the host

**Acceptance Criteria:**
- `switchboard skills install vercel-labs/agent-skills@frontend-design` runs `npx skills add vercel-labs/agent-skills@frontend-design -a kilo -y`
- `switchboard skills install vercel-labs/agent-skills` runs `npx skills add vercel-labs/agent-skills -a kilo -y`
- `switchboard skills install --global vercel-labs/agent-skills@frontend-design` passes `-g` to `npx skills add`
- Exit code from `npx skills add` is forwarded as Switchboard's exit code
- If `npx` is not found, a clear error is shown directing the user to install Node.js

**Example Output:**
```
$ switchboard skills install vercel-labs/agent-skills@frontend-design

[output from npx skills add vercel-labs/agent-skills@frontend-design -a kilo -y]
```

---

#### 3.1.3 `switchboard skills installed`

List all skills currently installed in the project.

**Behavior:**
- Scans `.kilocode/skills/` and `~/.kilocode/skills/` for directories containing a `SKILL.md` file
- Parses the YAML frontmatter from each `SKILL.md` to extract name and description
- Indicates whether each skill is project-level or global
- Indicates which agents (if any) have the skill explicitly assigned via `switchboard.toml`

**Acceptance Criteria:**
- Output lists all installed skills with name, description, scope (project/global), and assigned agents
- Skills with malformed or missing frontmatter are listed with a warning rather than silently omitted
- Empty state message is shown if no skills are installed

**Example Output:**
```
Installed Skills

  Project (.kilocode/skills/)
  ─────────────────────────────────────────────────────────────────
  frontend-design          High-quality UI/UX design guidelines     [all agents]
  security-audit           Security vulnerability scanning           [security-scan]

  Global (~/.kilocode/skills/)
  ─────────────────────────────────────────────────────────────────
  skill-creator            Create and improve agent skills           [all agents]

  3 skills installed (2 project, 1 global)
```

---

#### 3.1.4 `switchboard skills remove`

Remove an installed skill from the project.

**Behavior:**
- Accepts the skill name as a positional argument
- Searches `.kilocode/skills/<name>/` and removes the directory
- Prompts for confirmation before deletion (bypass with `--yes`)
- Accepts `--global` to remove from `~/.kilocode/skills/` instead

**Acceptance Criteria:**
- `switchboard skills remove frontend-design` removes `.kilocode/skills/frontend-design/` after confirmation
- Attempting to remove a skill that isn't installed returns a clear error
- If the skill is referenced in `switchboard.toml`, a warning is printed but removal proceeds

---

#### 3.1.5 `switchboard skills update`

Update installed skills to their latest versions.

**Behavior:**
- Delegates to `npx skills update` for updating all installed project-level skills
- Accepts an optional skill name argument, passing it through to `npx skills update <name>`
- Forwards stdout/stderr from `npx skills update` to the terminal unchanged

**Acceptance Criteria:**
- `switchboard skills update` invokes `npx skills update` and passes output through
- `switchboard skills update frontend-design` invokes `npx skills update frontend-design`
- Exit code from `npx skills update` is forwarded as Switchboard's exit code
- If `npx` is not found, a clear error is shown directing the user to install Node.js

---

### 3.2 `switchboard.toml` Config: Per-Agent Skill Declaration

#### 3.2.1 Per-Agent Skill Declaration

The `[[agent]]` section gains an optional `skills` field to restrict which skills are available to a given agent:

```toml
[[agent]]
name = "security-scan"
schedule = "0 2 * * *"
prompt = "Scan for security vulnerabilities."
skills = ["security-audit"]      # Only this skill is mounted for this agent

[[agent]]
name = "ui-reviewer"
schedule = "0 9 * * 1"
prompt = "Review UI components."
skills = ["frontend-design", "vercel-react-best-practices"]
```

If `skills` is omitted, the agent receives no skills. Skills must be explicitly declared per agent.

**Acceptance Criteria:**
- `skills` field in `[[agent]]` accepts a list of skill package sources in `owner/repo` or `owner/repo@skill-name` format
- `switchboard validate` warns if an agent references a `skills` entry but the field is empty
- Skills are installed inside the container at startup, not on the host

---

### 3.3 `switchboard build` — No Skills Involvement

`switchboard build` is responsible only for building the base Docker image. It has no involvement in skill installation and requires no knowledge of declared skills. Skills are entirely a runtime concern.

**Behavior:**
- `switchboard build` builds the Docker image as before, with no skill-related steps
- The built image must have `npx` available (satisfied by the existing `node:22-slim` base)
- No `[[skill]]` configuration is read or validated during `switchboard build`

**Rationale:**
Installing skills at build time would bake a single set of skills into a shared image, preventing per-agent skill differentiation. Installing at container startup keeps skills dynamic, agent-specific, and always up to date without requiring an image rebuild.

---

### 3.4 Container Execution: Per-Agent Skill Installation at Startup

Skills are installed inside each agent container as the first step of container execution, before the Kilo Code CLI is invoked with the agent's prompt. Each agent installs only the skills declared in its `skills` field.

**Behavior:**
- When a container starts, Switchboard generates a startup script that runs `npx skills add <source> -a kilo -y` for each entry in the agent's `skills` list
- Skills are installed into the container's `.kilocode/skills/` directory (the container's working copy, not the host)
- Skill installation runs sequentially in declaration order before any agent work begins
- If any skill installation fails, the container exits with a non-zero code and the failure is logged; the agent's prompt is not executed
- If an agent has no `skills` field, no skill installation runs and the container proceeds directly to agent execution
- Because installation happens inside the container, `npx` on the host is **not required** — only the container image needs Node.js (already satisfied by `node:22-slim`)
- Skills are not persisted between container runs; each run installs fresh from source

**Startup Script Pattern:**

Switchboard generates and injects an entrypoint script into the container at creation time:

```sh
#!/bin/sh
set -e

# Install declared skills
npx skills add vercel-labs/agent-skills@frontend-design -a kilo -y
npx skills add anthropics/skills@security-audit -a kilo -y

# Hand off to Kilo Code CLI
exec kilocode --yes "$@"
```

**Acceptance Criteria:**
- Skills are installed inside the container before the Kilo Code CLI is invoked
- Each agent installs only its own declared skills; agents do not share a skill install step
- A failed skill install aborts the container and is reported in `switchboard logs`
- Agents with no `skills` field skip the install step with no overhead
- `npx` is not required on the host machine for skill installation to work

---

### 3.5 `switchboard validate` Updates

The existing `validate` command is extended to check skill-related configuration:

- Warns if an `[[agent]]` has a `skills` field that is present but empty
- Reports an error if a `skills` entry is not a valid `owner/repo` or `owner/repo@skill-name` format
- Reports an error if the same skill source appears more than once in a single agent's `skills` list

---

## 4. Non-Functional Requirements

### 4.1 Performance

- `switchboard skills list` must return results within 3 seconds under normal network conditions
- Skill installation of a single skill inside a container should complete within 15 seconds on a standard broadband connection; this is accounted for in agent timeout budgets
- Container startup time including skill installation should be reflected in `switchboard metrics` so users can tune their timeout values accordingly

### 4.2 Reliability

- All `switchboard skills` CLI commands must degrade gracefully when the network is unavailable, with clear offline error messages
- A failed skill installation inside a container must result in a non-zero container exit code that Switchboard surfaces in logs and metrics
- Container skill installation failures must be distinguishable from agent execution failures in logs (e.g. via a distinct log prefix or exit code range)

### 4.3 Security

- Switchboard must not execute any code contained within a skill's `SKILL.md` — skills are data, not executable packages
- Users are warned that skills are community-authored and to review `SKILL.md` contents before installing
- No credentials or API keys should be written into skill files or transmitted to skills.sh

### 4.4 Compatibility

- Skills must conform to the open [Agent Skills specification](https://github.com/vercel-labs/skills) so they remain portable to other agents
- The feature must not break existing Switchboard projects that manually manage skills outside of `switchboard.toml`
- Skills installed by Switchboard must be loadable by Kilo Code CLI without any additional configuration

### 4.5 Dependency Management

- `npx` and Node.js are **required inside the agent container** for skill installation at runtime. This is satisfied by the existing `node:22-slim` Docker base image — no host-level Node.js is required.
- `npx` on the host is only required for the `switchboard skills list`, `switchboard skills install`, `switchboard skills update` CLI commands. If a user only declares skills in `switchboard.toml` and never uses the CLI skill commands manually, Node.js on the host is not needed.
- Switchboard must check for `npx` availability on the host at startup of `switchboard skills list/install/update`, and fail fast with: `Error: npx is required for this command. Install Node.js from https://nodejs.org`
- No new Rust-level network or HTTP dependencies are introduced by this feature

---

## 5. Technical Design Notes

### 5.1 New Module: `src/skills/mod.rs`

Introduces a `SkillsManager` struct responsible for:

- Invoking `npx skills` subcommands as child processes and forwarding their output
- Checking `npx` availability on the host and returning a structured error if absent
- Reading and parsing `SKILL.md` YAML frontmatter for the `installed` and `remove` commands (local filesystem only — no network calls)
- Removing skill directories for the `remove` command

The module deliberately contains no HTTP client code, no GitHub API calls, and no caching logic. All remote operations are fully owned by `npx skills`.

### 5.2 `npx` Process Invocation Pattern

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

stdout and stderr are inherited from the parent process (not captured), so `npx skills` output renders directly in the user's terminal including any color, interactivity, or progress indicators.

### 5.3 Config Schema Additions

`src/config/mod.rs` is extended with:

- `Vec<String>` field on `AgentConfig` for the per-agent `skills` list (list of `owner/repo` or `owner/repo@skill-name` strings)
- The top-level `[skills]` table and `[[skill]]` global declarations are removed from the schema — skills are now declared exclusively per agent

### 5.4 Container Entrypoint Script Generation

In `src/docker/mod.rs`, when an agent has a non-empty `skills` field, Switchboard generates a shell entrypoint script and writes it into the container via a bind-mounted temporary file or by embedding it as a Docker entrypoint override. The script installs skills via `npx skills add` then execs the Kilo Code CLI. When `skills` is empty or omitted, the container uses its default entrypoint unchanged.

The generated script is created fresh for each container run and does not persist on the host after the container exits.

---

## 6. Error Handling

| Scenario | Behavior |
|----------|----------|
| `npx` not found on host | Immediate failure with: `Error: npx is required for this command. Install Node.js from https://nodejs.org` |
| `npx skills add` exits with non-zero | Forward exit code and stderr to user; Switchboard adds no additional wrapping |
| Skill not found in repo | `npx skills` handles and reports this; Switchboard forwards the message |
| Network unavailable during install/list | `npx skills` handles and reports this; Switchboard forwards the message |
| Malformed `SKILL.md` frontmatter (during `installed`) | Warn and skip the affected skill; do not crash |
| Skill name collision (project vs global) | Project-level takes precedence; warn user |
| Skill install fails inside container | Container exits non-zero; failure logged with distinction from agent execution failure; reported in `switchboard metrics` |
| Agent has empty `skills = []` field | `switchboard validate` warns; container runs without skill installation |
| Invalid `skills` entry format | `switchboard validate` reports an error before any container is started |

---

## 7. Out of Scope for v0.1

The following are acknowledged as desirable but deferred to future iterations:

- Skill version pinning (install a skill at a specific git SHA)
- Private/internal skills hosted outside of public GitHub
- A `switchboard skills publish` command for submitting skills to the marketplace
- Skill dependency resolution (skills that depend on other skills)
- Web UI dashboard showing installed skills and agent assignments
- A native Rust fallback for environments where Node.js is unavailable (not planned — `npx` is treated as a hard prerequisite)

---

## 8. Acceptance Criteria Summary

| ID | Criteria | Priority |
|----|----------|----------|
| AC-01 | `switchboard skills list` invokes `npx skills find` and passes output through | High |
| AC-02 | `switchboard skills list --search <query>` invokes `npx skills find <query>` | High |
| AC-03 | `switchboard skills install <source>` invokes `npx skills add <source> -a kilo -y` | High |
| AC-04 | `switchboard skills installed` lists installed skills by scanning `.kilocode/skills/` | High |
| AC-05 | `switchboard skills remove <name>` removes an installed skill after confirmation | Medium |
| AC-06 | `switchboard skills update` invokes `npx skills update` and passes output through | Low |
| AC-07 | Per-agent `skills = [...]` in `[[agent]]` declares skills to install inside that agent's container | High |
| AC-08 | Skills are installed inside the container at startup before the Kilo Code CLI is invoked | High |
| AC-09 | A failed skill install inside a container aborts the run and is surfaced in logs and metrics | High |
| AC-10 | `switchboard validate` checks skill references are satisfied | High |
| AC-11 | All commands requiring `npx` fail fast with a clear prerequisite error if `npx` is not found | High |
| AC-12 | Exit codes from all `npx skills` invocations are forwarded as Switchboard's exit code | High |

---

## 9. Open Questions

1. **Skill install latency and agent timeouts**: Installing skills at container startup adds latency before the agent begins work. Should Switchboard automatically subtract an estimated skill install time from the agent's configured timeout, or should users be expected to account for this manually in their `timeout` values?

2. **Skill version pinning**: Should the `skills` field support pinning to a specific git SHA or tag (e.g. `owner/repo@skill-name#abc1234`) to make agent behavior reproducible across runs? The `npx skills` CLI may support this in the future.

3. **Skill caching across runs**: Installing skills fresh on every container run adds startup time. Should Switchboard optionally cache installed skills in a named Docker volume that persists between runs for the same agent, invalidated only when the `skills` list changes?

4. **`npx skills` version pinning**: Should Switchboard target a specific version of the `npx skills` package (e.g. `npx skills@1.1.4`) in the generated entrypoint script to avoid breaking changes in future releases, at the cost of not getting automatic improvements?

5. **Skill install failure policy**: Should a failed skill install always abort the agent run, or should Switchboard support a `skills_optional = true` flag per agent that logs the failure but proceeds with execution anyway?