# Agents Guide

This guide provides comprehensive documentation for working with agents in Switchboard. Agents are the fundamental units of automation in Switchboard — scheduled tasks that execute AI-powered prompts against your workspace via the Kilo Code CLI, each running in its own isolated Docker container.

## Overview

An agent in Switchboard represents a recurring task that performs automated work on your codebase. Each agent is defined in [`switchboard.toml`](../switchboard.toml) with:

- A unique **name** for identification
- A **schedule** using cron syntax for when it runs
- A **prompt** (either inline text or a file reference) containing the task instructions
- Optional configuration like timeout, read-only mode, and environment variables

Agents run in isolated Docker containers with your workspace mounted at `/workspace`, allowing them to read and modify files just like a human developer would.

---

## Creating effective agent prompts

Writing effective prompts is crucial for getting reliable, useful results from your agents. The prompt defines the agent's behavior, constraints, and output expectations.

### Prompt Structure

Well-structured prompts follow a clear, hierarchical pattern:

```markdown
# Agent Role

Brief description of the agent's purpose and scope.

## Core Responsibilities
- What the agent does
- What the agent does NOT do
- Key rules and constraints

## Tasks/Phases
Ordered steps the agent should follow
- Phase 1: Initial setup and context loading
- Phase 2: Primary work execution
- Phase 3: Verification and cleanup
```

### Best Practices

#### 1. Define Role and Scope Clearly

Start by explicitly defining who the agent is and what it's responsible for:

```markdown
# ARCHITECT.md

You are the Lead Architect. You run periodically to ensure the project is on track.
You do NOT write code. You plan. Each task should be delegated to a subagent.
```

This immediately establishes boundaries — the agent knows to focus on planning, not implementation.

#### 2. Use Golden Rules for Critical Constraints

For agents with hard constraints, state them prominently:

```markdown
## The Golden Rule
**NEVER MODIFY `PRD.md`.** It is the immutable source of truth.
```

Golden rules should be placed near the top where they can't be missed.

#### 3. Support Idempotency with Session Protocols

Agents can be interrupted by timeouts. Design prompts that can be resumed across sessions:

```markdown
## Session Protocol (Idempotency)

This prompt may be interrupted by timeouts. You MUST follow this protocol to ensure
work can be resumed across multiple sessions.

### On Session Start
1. **Check for continuation:** Look for `.agent_in_progress` marker file
2. **If marker exists:** Read `AGENT_STATE.md` to see what was completed and resume from there
3. **If no marker:** Create `.agent_in_progress` and start fresh

### On Session End
**If ALL tasks complete:**
1. Delete `.agent_in_progress` marker
2. Commit: `chore(agent): session complete`

**If interrupted:**
1. Keep `.agent_in_progress` marker
2. Update `AGENT_STATE.md` with current state
3. Commit: `chore(agent): session partial - will continue`
```

#### 4. Use Ordered Phases for Complex Work

Break complex tasks into sequential phases that must be completed in order:

```markdown
## Your Investigation Phases

Work through these **in order**. Update `QA_STATE.md` after each.

### Phase 0: Load Planned Work Context
- Read `TODO.md` and all `TODO*.md` files
- Read `BACKLOG.md`
- Build a mental map of what's done, in-progress, and planned
- ✅ Mark complete in `QA_STATE.md`

### Phase 1: Automated Test Sweep
- Run the full test suite
- Run the linter
- Document every failure
- ✅ Mark complete in `QA_STATE.md`

### Phase 2: Write Bug Report
- Compile findings into `BUGS.md`
- Prioritize by severity
- ✅ Mark complete in `QA_STATE.md`
```

#### 5. Include Acceptance Criteria

Define clear success conditions that can be programmatically verified:

```markdown
### Subtask: Create User model

### Acceptance Criteria
- [ ] File exists at apps/api/src/models/user.model.ts
- [ ] Exports a Mongoose model named "User"
- [ ] `npx ts-node -e "import './apps/api/src/models/user.model'"` exits without error

### Do NOT
- [Anything the subagent should avoid]
```

#### 6. Provide "Do NOT" Instructions

Explicitly state what the agent should avoid doing:

```markdown
### Do NOT
- Modify source code, tests, or config files
- Report issues already tracked in `BLOCKERS.md`
- Report stub implementations that correspond to a TODO or BACKLOG item
```

#### 7. Use Concrete Examples

When possible, include working examples that demonstrate expected behavior:

```markdown
### Good vs. Bad Examples

**❌ BAD — Too vague:**
```
Review the recent commits for issues.
```

**✅ GOOD — Specific and actionable:**
```
Read the last 24 hours of git commits. For each commit:
1. Check if the author has a valid email domain
2. Verify commit messages follow the conventional commit format
3. Flag commits with "fix:" in the subject that don't reference an issue
```
```

#### 8. Specify Output Formats

Define exactly how results should be formatted:

```markdown
## Bug Report Format (`BUGS.md`)

```markdown
# Bug Report
> Generated: [timestamp]

## Critical

### BUG-001: [Short descriptive title]
- **Location:** `src/module/file.rs:42`
- **Category:** Logic Bug
- **Found by:** Phase 1 — Test Sweep
- **Description:** [What's wrong, in 2-3 sentences]
- **Evidence:** [Exact error output]
- **Expected behavior:** [What should happen per PRD]
- **Actual behavior:** [What happens now]
- **Fix estimate:** S / M / L
```
```

### Examples from sample_prompts/

The [`sample_prompts/`](../sample_prompts/) directory contains several well-structured agent prompts that demonstrate these best practices:

- [`ARCHITECT.md`](../sample_prompts/ARCHITECT.md) — Planning and sprint management
  - Uses golden rules
  - Implements session protocols for idempotency
  - Defines ordered phases for sprint management
  - Uses state files to track progress

- [`DEV.md`](../sample_prompts/DEV.md) — Task decomposition and orchestration
  - Clear role definition vs. subagent roles
  - Detailed decomposition protocol with rules
  - Good/bad examples for comparison
  - Verification protocol for subtask completion

- [`QA.md`](../sample_prompts/QA.md) — Bug hunting and reporting
  - Explicit "do NOT modify source code" constraint
  - 6-phase investigation workflow
  - Structured bug report format
  - Clear exclusion rules to avoid false positives

- [`SUMMARIZER.md`](../sample_prompts/SUMMARIZER.md) — Log and activity aggregation
  - Focused, single-purpose role
  - Clear output format specification

### Common Prompt Patterns

#### Pattern 1: Read-Only Analysis Agent

```markdown
# Agent Name

You are **read-only analyzer**. You investigate, report, but never modify.

## The Golden Rule
**NEVER MODIFY source code or configuration files.**

## Your Investigation
1. Read relevant files
2. Analyze against criteria
3. Generate report in specific format

## Report Format
[Define output structure here]
```

#### Pattern 2: Orchestrator Agent

```markdown
# Orchestrator

You are an **orchestrator agent**. You do NOT write code directly.
Your job is to plan, decompose, delegate to subagents, verify results.

## Role vs. Subagents
| You | Subagents |
|------|-----------|
| Plan | Write code |
| Decompose tasks | Execute one subtask |
| Verify results | Report back |

## Task Decomposition Protocol
[Detailed rules for breaking work into atomic subtasks]

## Verification Protocol
[How to verify each subtask before proceeding]
```

#### Pattern 3: Stateful Long-Running Agent

```markdown
# Long-Running Agent

## Session Protocol
### On Session Start
1. Check for `in_progress` marker
2. If exists, load state and resume
3. If not, start fresh

### During Session
- Update state file after each milestone
- Commit progress incrementally

### On Session End
- If complete: clean up markers
- If incomplete: save state for resumption

## Your Tasks (In Order)
[Numbered list with clear checkpoints]
```

---

## Best practices for cron scheduling

Cron expressions define when agents run. Switchboard uses standard 5-field cron syntax, which offers powerful scheduling capabilities.

### Cron Expression Format

Cron expressions consist of 5 space-separated fields:

```
┌───────────── minute (0 - 59)
│ ┌─────────── hour (0 - 23)
│ │ ┌───────── day of month (1 - 31)
│ │ │ ┌─────── month (1 - 12)
│ │ │ │ ┌───── day of week (0 - 6, 0 = Sunday)
│ │ │ │ │
* * * * *
```

### Valid Field Values

| Field | Valid Values | Special Characters |
|-------|--------------|---------------------|
| Minute | 0-59 | `*`, `,`, `-`, `/` |
| Hour | 0-23 | `*`, `,`, `-`, `/` |
| Day of Month | 1-31 | `*`, `,`, `-`, `/`, `?` |
| Month | 1-12 | `*`, `,`, `-`, `/` |
| Day of Week | 0-6 (Sun-Sat) | `*`, `,`, `-`, `/`, `?` |

### Special Characters

| Character | Meaning | Example |
|-----------|---------|---------|
| `*` | Any value | `*` = every minute/hour/day |
| `,` | Value list separator | `1,3,5` = 1st, 3rd, and 5th |
| `-` | Range | `1-5` = 1 through 5 |
| `/` | Step values | `*/10` = every 10th, `0-30/5` = every 5 between 0-30 |
| `?` | No specific value (day of month/week only) | `?` = ignore this field |

### Common Scheduling Patterns

#### Run Every N Minutes

```toml
# Every minute
schedule = "* * * * *"

# Every 5 minutes
schedule = "*/5 * * * *"

# Every 30 minutes
schedule = "*/30 * * * *"

# Every 15 minutes
schedule = "*/15 * * * *"
```

#### Run Every N Hours

```toml
# Every hour (at minute 0)
schedule = "0 * * * *"

# Every 2 hours
schedule = "0 */2 * * *"

# Every 6 hours
schedule = "0 */6 * * *"

# Every 12 hours
schedule = "0 */12 * * *"
```

#### Run at Specific Times Daily

```toml
# Daily at midnight
schedule = "0 0 * * *"

# Daily at 9:00 AM
schedule = "0 9 * * *"

# Daily at 5:00 PM (17:00)
schedule = "0 17 * * *"

# Twice daily: 9 AM and 5 PM
schedule = "0 9,17 * * *"
```

#### Run on Specific Days

```toml
# Every Monday at midnight
schedule = "0 0 * * 1"

# Every Friday at 5 PM
schedule = "0 17 * * 5"

# Monday through Friday at 9 AM
schedule = "0 9 * * 1-5"

# Weekends at 10 AM (Saturday and Sunday)
schedule = "0 10 * * 0,6"
```

#### Run Monthly

```toml
# First day of every month at midnight
schedule = "0 0 1 * *"

# 15th of every month at noon
schedule = "0 12 15 * *"

# Last day of month (works for most months)
schedule = "0 23 28-31 * *"

# First Monday of every month
schedule = "0 0 1-7 * 1"
```

#### Complex Patterns

```toml
# Every 6 hours during business hours (9 AM - 5 PM)
schedule = "0 9-17/6 * * *"

# Every 30 minutes from 8 AM to 6 PM, Monday through Friday
schedule = "*/30 8-18 * * 1-5"

# At 9 AM, 1 PM, and 5 PM on weekdays
schedule = "0 9,13,17 * * 1-5"

# Every 2 hours on weekends
schedule = "0 */2 * * 0,6"
```

### Timing Considerations

#### 1. Consider Agent Runtime Duration

Schedule intervals should be longer than expected agent runtime to avoid overlap issues:

```toml
# Bad: Agent takes ~10 minutes, runs every 5 minutes
schedule = "*/5 * * * *"

# Good: Agent takes ~10 minutes, runs every 30 minutes
schedule = "*/30 * * * *"
```

If agents may overlap, configure overlap mode in [`switchboard.toml`](../switchboard.toml):

```toml
[settings]
overlap_mode_str = "queue"  # Queue runs instead of skipping

[[agent]]
name = "long-running-agent"
schedule = "*/20 * * * *"  # Every 20 minutes
timeout = "15m"             # Timeout shorter than schedule
```

#### 2. Align with Business Hours

For agents that require human oversight, run during business hours:

```toml
# US Eastern Time business hours (9 AM - 5 PM, Mon-Fri)
schedule = "0 9-17 * * 1-5"
timezone = "America/New_York"
```

#### 3. Spread Resource-Intensive Agents

Avoid running multiple resource-heavy agents simultaneously:

```toml
# Heavy code review at 2 AM
[[agent]]
name = "nightly-code-review"
schedule = "0 2 * * *"

# Security scan at 4 AM (2 hours later)
[[agent]]
name = "nightly-security-scan"
schedule = "0 4 * * *"

# Dependency check at 6 AM (2 hours later)
[[agent]]
name = "nightly-dependency-check"
schedule = "0 6 * * *"
```

#### 4. Use Appropriate Granularity

Match schedule frequency to the rate of relevant activity:

```toml
# Real-time monitoring: every minute
[[agent]]
name = "uptime-monitor"
schedule = "* * * * *"

# Daily review: once per day
[[agent]]
name = "daily-log-summary"
schedule = "0 9 * * *"

# Weekly cleanup: once per week
[[agent]]
name = "weekly-cache-cleanup"
schedule = "0 2 * * 0"
```

#### 5. Account for Timezones

Specify the timezone for cron evaluation in settings:

```toml
[settings]
timezone = "America/New_York"

[[agent]]
schedule = "0 9 * * *"  # Runs at 9 AM Eastern Time
```

Common IANA timezone formats:
- `America/New_York` — Eastern Time
- `America/Los_Angeles` — Pacific Time
- `Europe/London` — London Time
- `Asia/Tokyo` — Tokyo Time
- `UTC` — Coordinated Universal Time

#### 6. Use Non-Conflicting Schedules

Avoid multiple agents running at exactly the same time unless necessary:

```toml
# Stagger agents by a few minutes
[[agent]]
name = "agent-1"
schedule = "0 9 * * *"      # 9:00 AM

[[agent]]
name = "agent-2"
schedule = "5 9 * * *"      # 9:05 AM

[[agent]]
name = "agent-3"
schedule = "10 9 * * *"     # 9:10 AM
```

### Testing Cron Schedules

Validate cron expressions using [`switchboard validate`](../README.md):

```bash
# Validate configuration (includes cron syntax check)
switchboard validate

# List agents with next run times
switchboard list
```

Use online cron expression testers to verify schedules before deployment:
- [crontab.guru](https://crontab.guru/) — Visual cron schedule explainer
- [Cron-Expression](https://www.freeformatter.com/cron-expression-generator-quartz.html) — Generator and tester

### Predefined Schedule Keywords

Switchboard supports these predefined schedule aliases:

| Alias | Equivalent Cron | Description |
|-------|----------------|-------------|
| `@yearly` | `0 0 1 1 *` | Once per year (midnight Jan 1) |
| `@annually` | `0 0 1 1 *` | Same as @yearly |
| `@monthly` | `0 0 1 * *` | Once per month (midnight 1st) |
| `@weekly` | `0 0 * * 0` | Once per week (Sunday midnight) |
| `@daily` | `0 0 * * *` | Once per day (midnight) |
| `@hourly` | `0 * * * *` | Once per hour (top of hour) |

---

## Prompt file vs inline prompt guidance

Switchboard supports two approaches for providing agent prompts: inline prompts in the configuration file, or external prompt files. Each approach has advantages depending on your use case.

### Inline Prompts

Inline prompts are specified directly in the [`switchboard.toml`](../switchboard.toml) configuration:

```toml
[[agent]]
name = "daily-review"
schedule = "0 9 * * *"
prompt = "Review all files changed in the last 24 hours. Flag bugs, security issues, and style violations."
timeout = "30m"
```

#### When to Use Inline Prompts

Inline prompts are best for:

1. **Simple, concise prompts**
   ```toml
   prompt = "Generate a summary of yesterday's activity logs."
   ```

2. **Single-purpose agents** with straightforward instructions
   ```toml
   prompt = "Check for outdated dependencies and open issues."
   ```

3. **Quick prototyping** and iteration
   ```toml
   prompt = "Analyze the latest commit for potential issues."
   ```

4. **Agents with frequently changing prompts** that need quick edits

#### Advantages of Inline Prompts

- **Single file** — Configuration and prompt in one place
- **Quick to edit** — No need to navigate to separate files
- **Self-contained** — Easy to copy-paste full agent definitions
- **Less overhead** — No file path resolution or additional files to manage

#### Disadvantages of Inline Prompts

- **Limited length** — Large prompts make configuration files unwieldy
- **No version history** — Changes to prompts are mixed with config changes
- **Difficult to maintain** — Complex multi-section prompts become hard to read in TOML
- **No reusability** — Cannot share prompts between agents

#### Inline Prompt Examples

**Simple single-line prompt:**
```toml
[[agent]]
name = "uptime-check"
schedule = "*/5 * * * *"
prompt = "Check if the application server is responsive and log the result."
```

**Multi-line prompt using triple quotes:**
```toml
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = """
Review all files changed in the last 6 hours.

Focus on:
- Security vulnerabilities
- Performance issues
- Code style violations

Generate a report and save to reports/code-review.md.
"""
timeout = "1h"
```

### Prompt Files

Prompt files are external Markdown files referenced from the configuration:

```toml
[[agent]]
name = "architect"
schedule = "0 2 * * *"
prompt_file = "prompts/architect.md"
timeout = "2h"
```

The prompt file content is loaded at agent execution time.

#### When to Use Prompt Files

Prompt files are best for:

1. **Complex, multi-section prompts** with structured content
2. **Reused prompts** shared across multiple agents or projects
3. **Agents requiring frequent updates** while keeping config stable
4. **Long-form documentation-style prompts** with examples and instructions
5. **Collaborative environments** where multiple developers edit prompts

#### Advantages of Prompt Files

- **Maintainability** — Large prompts are easier to read and edit in dedicated files
- **Version control friendly** — Prompt changes are tracked separately from config
- **Reusability** — Same prompt file can be referenced by multiple agents
- **Rich formatting** — Markdown syntax for headers, lists, code blocks
- **External tooling** — Can use editors, linters, and review tools
- **Documentation** — Prompt files can serve as both config and documentation

#### Disadvantages of Prompt Files

- **Multiple files** — Need to manage configuration and prompt files separately
- **Path management** — Must ensure prompt files exist at correct paths
- **Context switching** — Need to navigate between config and prompt files
- **Potential for drift** — Prompt file might be modified without updating agent config

#### Prompt File Examples

**Simple prompt file structure:**
```markdown
# prompts/nightly-summary.md

You are a nightly summary agent. Review the day's activity and generate a brief report.

Your task:
1. Check recent commits in the last 24 hours
2. Review any open issues or pull requests
3. Check for any error logs or exceptions
4. Generate a 3-5 bullet point summary

Output format:
Save the summary to `reports/nightly-summary-{YYYY-MM-DD}.md`
```

**Complex prompt file with sections:**
See [`sample_prompts/ARCHITECT.md`](../sample_prompts/ARCHITECT.md) for a complete example with:
- Golden rules
- Session protocols for idempotency
- Ordered task phases
- State file management
- Commit conventions

**Reusable prompt file:**
```markdown
# prompts/code-review.md

# Code Review Agent

You are a code reviewer. Analyze code changes for quality issues.

## Review Criteria
- Security vulnerabilities
- Performance issues
- Code style violations
- Potential bugs
- Documentation gaps

## Output Format
Generate a report in Markdown format at: `reviews/review-{timestamp}.md`

## Severity Levels
- Critical: Must fix immediately
- High: Should fix soon
- Medium: Nice to have
- Low: Minor improvements
```

Used by multiple agents:
```toml
[[agent]]
name = "daily-code-review"
schedule = "0 9 * * *"
prompt_file = "prompts/code-review.md"
timeout = "30m"
env = { REVIEW_SCOPE = "last-24h" }

[[agent]]
name = "weekly-code-review"
schedule = "0 10 * * 1"
prompt_file = "prompts/code-review.md"
timeout = "2h"
env = { REVIEW_SCOPE = "last-7d" }
```

### Path Resolution

Prompt file paths are resolved relative to the project root (the directory containing `switchboard.toml`):

```toml
# Project structure:
# /project-root/
#   switchboard.toml
#   prompts/
#     agent.md

[[agent]]
name = "my-agent"
prompt_file = "prompts/agent.md"  # Resolves to /project-root/prompts/agent.md
```

Absolute paths are also supported:

```toml
[[agent]]
name = "my-agent"
prompt_file = "/absolute/path/to/agent.md"
```

### Comparison Summary

| Aspect | Inline Prompts | Prompt Files |
|--------|----------------|--------------|
| Best for | Simple, short prompts | Complex, long prompts |
| File count | Single config file | Config + prompt file(s) |
| Readability | Good for 1-3 lines | Excellent for multi-section prompts |
| Maintainability | Good for simple prompts | Excellent for complex prompts |
| Version control | Changes mixed with config | Separate commit history |
| Reusability | Cannot be shared | Can be shared across agents |
| Edit speed | Very fast | Fast (but requires file navigation) |
| Rich formatting | Limited (TOML strings) | Full Markdown support |
| External tooling | Not applicable | Can use editors, linters, etc. |
| Prompt length | Practically limited (lines get long) | No practical limit |

### Choosing the Right Approach

**Use inline prompts when:**
- Prompt is 1-3 sentences
- Agent has a single, simple purpose
- You're prototyping or iterating quickly
- Prompt changes frequently alongside config changes

**Use prompt files when:**
- Prompt has multiple sections or structure
- Prompt includes examples, code blocks, or complex formatting
- Prompt is reused by multiple agents
- You want separate version history for prompts
- Multiple developers collaborate on prompt content
- Prompt length exceeds ~10 lines

### Hybrid Approach

You can mix both approaches in the same configuration file, using whichever is appropriate for each agent:

```toml
# Simple inline prompt
[[agent]]
name = "health-check"
schedule = "*/5 * * * *"
prompt = "Check system health and log metrics."

# Complex prompt file
[[agent]]
name = "architect"
schedule = "0 2 * * *"
prompt_file = "prompts/architect.md"

# Reusable prompt file used by multiple agents
[[agent]]
name = "daily-review"
schedule = "0 9 * * *"
prompt_file = "prompts/code-review.md"
env = { REVIEW_SCOPE = "daily" }

[[agent]]
name = "weekly-review"
schedule = "0 10 * * 1"
prompt_file = "prompts/code-review.md"
env = { REVIEW_SCOPE = "weekly" }
```

---

## Agent naming conventions

Agent names are the primary identifier for your automated tasks. Well-chosen names make your configuration readable, maintainable, and self-documenting.

### Naming Rules

Agent names in Switchboard must:

1. **Be unique** — No two agents can have the same name in a single configuration
2. **Be non-empty** — Names cannot be blank
3. **Be valid TOML strings** — Enclosed in double quotes, can contain most characters

### Syntax Requirements

```toml
# Valid agent names
[[agent]]
name = "code-reviewer"        # lowercase with hyphens
schedule = "0 */6 * * *"

[[agent]]
name = "daily_report"        # underscores
schedule = "0 9 * * *"

[[agent]]
name = "DependencyChecker"   # camelCase
schedule = "0 10 * * *"

# Agent names are strings - use quotes
[[agent]]
name = "agent123"            # alphanumeric
schedule = "0 * * * *"
```

### Recommended Conventions

#### 1. Use kebab-case for Readability

Kebab-case (lowercase with hyphens) is the most common convention:

```toml
# Recommended
[[agent]]
name = "code-reviewer"
name = "nightly-security-scan"
name = "daily-log-summary"

# Less common but acceptable
name = "code_reviewer"      # snake_case
name = "CodeReviewer"        # CamelCase
name = "code-reviewer-2"     # With version number
```

#### 2. Make Names Descriptive

Choose names that clearly communicate what the agent does:

```toml
# Good - clear purpose
[[agent]]
name = "dependency-checker"

# Good - includes frequency
[[agent]]
name = "nightly-code-review"

# Good - includes scope
[[agent]]
name = "weekly-security-audit"

# Avoid - too vague
[[agent]]
name = "agent1"
name = "task"
name = "worker"
```

#### 3. Include Frequency When Relevant

For time-based agents, include the schedule frequency in the name:

```toml
# Good - includes frequency
[[agent]]
name = "hourly-metrics-collector"
schedule = "0 * * * *"

[[agent]]
name = "nightly-database-backup"
schedule = "0 2 * * *"

[[agent]]
name = "weekly-report-generator"
schedule = "0 10 * * 1"
```

#### 4. Use Functional Role Names

For agents with specific roles in your workflow, name them by their function:

```toml
[[agent]]
name = "architect"
schedule = "0 2 * * *"

[[agent]]
name = "code-reviewer"
schedule = "0 */4 * * *"

[[agent]]
name = "qa-tester"
schedule = "0 8 * * *"

[[agent]]
name = "documentation-writer"
schedule = "0 16 * * *"
```

### Naming Patterns by Category

#### Code Quality Agents

```toml
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"

[[agent]]
name = "security-scanner"
schedule = "0 2 * * *"

[[agent]]
name = "lint-checker"
schedule = "0 */2 * * *"

[[agent]]
name = "style-enforcer"
schedule = "0 10 * * *"
```

#### Monitoring & Metrics Agents

```toml
[[agent]]
name = "uptime-monitor"
schedule = "* * * * *"

[[agent]]
name = "metrics-collector"
schedule = "*/5 * * * *"

[[agent]]
name = "performance-tracker"
schedule = "*/15 * * * *"

[[agent]]
name = "error-log-analyzer"
schedule = "0 * * * *"
```

#### Maintenance Agents

```toml
[[agent]]
name = "dependency-checker"
schedule = "0 9 * * 1"

[[agent]]
name = "cache-cleaner"
schedule = "0 3 * * *"

[[agent]]
name = "database-optimizer"
schedule = "0 4 * * 0"

[[agent]]
name = "log-rotator"
schedule = "0 5 * * *"
```

#### Documentation Agents

```toml
[[agent]]
name = "doc-generator"
schedule = "0 16 * * *"

[[agent]]
name = "api-doc-sync"
schedule = "0 14 * * *"

[[agent]]
name = "changelog-updater"
schedule = "0 18 * * 5"

[[agent]]
name = "readme-maintainer"
schedule = "0 12 * * 1"
```

#### Orchestration Agents

```toml
[[agent]]
name = "architect"
schedule = "0 2 * * *"

[[agent]]
name = "dev-orchestrator"
schedule = "0 9 * * *"

[[agent]]
name = "qa-runner"
schedule = "0 17 * * *"

[[agent]]
name = "deploy-coordinator"
schedule = "0 13 * * *"
```

### Naming Parallel Agents

When you have multiple agents that perform similar tasks in parallel or sequentially, use version numbers or descriptive suffixes:

```toml
# Version numbers for parallel agents
[[agent]]
name = "code-reviewer-1"
schedule = "0 */4 * * *"
prompt_file = "prompts/code-reviewer-1.md"

[[agent]]
name = "code-reviewer-2"
schedule = "0 */4 * * *"
prompt_file = "prompts/code-reviewer-2.md"

# Descriptive suffixes for sequential agents
[[agent]]
name = "code-reviewer-frontend"
schedule = "0 */4 * * *"
prompt = "Review frontend code changes."

[[agent]]
name = "code-reviewer-backend"
schedule = "1 */4 * * *"
prompt = "Review backend code changes."

[[agent]]
name = "code-reviewer-infrastructure"
schedule = "2 */4 * * *"
prompt = "Review infrastructure changes."
```

### Avoiding Name Collisions

Agent names must be unique. When working with multiple projects or teams:

```toml
# Use project prefixes for team environments
[[agent]]
name = "frontend-dependency-checker"
schedule = "0 9 * * *"

[[agent]]
name = "backend-dependency-checker"
schedule = "0 9 * * *"

# Use environment prefixes for dev/staging/prod
[[agent]]
name = "dev-code-reviewer"
schedule = "0 */2 * * *"

[[agent]]
name = "staging-code-reviewer"
schedule = "0 */4 * * *"

[[agent]]
name = "prod-code-reviewer"
schedule = "0 */6 * * *"
```

### Name Length Considerations

Keep agent names reasonably concise but descriptive:

```toml
# Good - concise and clear
[[agent]]
name = "nightly-backup"
name = "security-scan"
name = "dep-checker"

# Acceptable - longer but clear
[[agent]]
name = "nightly-database-backup"
name = "security-vulnerability-scan"
name = "dependency-vulnerability-checker"

# Avoid - too short
[[agent]]
name = "backup"
name = "scan"
name = "check"

# Avoid - too long
[[agent]]
name = "automated-nightly-database-backup-and-verification"
name = "comprehensive-security-vulnerability-scanning-agent"
```

### Examples from sample_prompts/

The [`sample_prompts/`](../sample_prompts/) directory uses clear, descriptive naming:

- `ARCHITECT.md` — Architect planning agent
- `DEV.md` — Development orchestrator agent
- `QA.md` — Quality assurance/testing agent
- `SUMMARIZER.md` — Log/activity summarization agent
- `FIX.md` — Bug fix agent
- `JANITOR.md` — Cleanup/maintenance agent

### Best Practices Summary

| Practice | Example | Why |
|----------|---------|-----|
| Use kebab-case | `code-reviewer` | Readable, conventional |
| Be descriptive | `nightly-security-scan` | Self-documenting |
| Include frequency | `hourly-metrics` | Indicates schedule |
| Use role names | `architect`, `qa-runner` | Clear purpose |
| Prefix for disambiguation | `frontend-linter` | Avoids collisions |
| Keep reasonable length | `daily-backup` | Not too short/long |
| Version parallel agents | `reviewer-1`, `reviewer-2` | Clear ordering |

### Agent Name Impact

Agent names are used in several places in Switchboard:

1. **Configuration** — Identifies the agent in [`switchboard.toml`](../switchboard.toml)
2. **Logging** — Prefixes all log entries: `[agent-name]`
3. **Metrics** — Groups execution statistics by agent name
4. **CLI commands** — Used to reference specific agents:
   ```bash
   switchboard logs code-reviewer
   switchboard run nightly-backup
   ```
5. **Docker labels** — Container labels include agent name for tracking

Choose names that work well in all these contexts — short enough for CLI use but descriptive enough for logs and metrics.

---

## Additional Resources

- [README.md](../README.md) — Main project documentation
- [PRD.md](../PRD.md) — Product requirements and architecture
- [docs/setup.md](setup.md) — Setup and configuration guide
- [docs/troubleshooting.md](troubleshooting.md) — Common issues and solutions
- [sample_prompts/](../sample_prompts/) — Example agent prompts
- [examples/](../examples/) — Sample configuration files
