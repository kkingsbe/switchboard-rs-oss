# Goal-Based Workflows — Design Document

## Overview

Goal-based workflows are a new class of Switchboard workflow where the human operator provides only a `goals.md` file as input. The system autonomously discovers context, plans work, executes, and verifies — operating in continuous tight loops until the goals are met.

This is fundamentally different from BMAD. BMAD front-loads structure: the human provides a brief, an architect produces a PRD, and work flows downhill through well-defined phases. Goal-based workflows invert this. The agent system figures out what work needs to happen before it can do it. The `goals.md` is a compass heading, not a blueprint.

Goals can range from specific and measurable ("reduce API call latency below 200ms") to broad and open-ended ("create a social media application") to business-outcome-oriented ("increase MRR") when placed in an existing project.

---

## Design Decisions

### Loop Structure — Tight, Continuous Loops

The system operates in tight discover/plan/execute/reassess loops rather than following a linear phase-gated pipeline. Each loop iteration discovers a little, plans a little, executes a little, and reassesses — similar to how a senior developer works on an ambiguous problem.

This is cron-driven using standard Switchboard scheduling, consistent with all other Switchboard workflows.

### Scope Bounding — Milestone-Based

The planner's first action is to derive concrete milestones from `goals.md`. These milestones become the backbone for all subsequent loops. The system works through milestones sequentially, with each loop iteration advancing toward the current milestone.

### Success Criteria — Agent-Derived from goals.md

Success criteria are derived directly from `goals.md` content, not invented independently by agents. They are traceable back to the human's intent. If `goals.md` is updated, the system detects the change and re-derives criteria accordingly.

### Project Detection — Auto-Detect from Workspace

The planner auto-detects whether it's working in an existing codebase or greenfield by scanning workspace contents on first run. An existing project means: read the codebase, understand architecture, identify what's relevant to the goal. Greenfield means: make foundational decisions, scaffold, build incrementally. The discovery phase adapts accordingly.

### Goal Change Handling — Diff and Adapt

The planner checksums `goals.md` on each run. When a change is detected, it performs a reconciliation pass: keeping progress on still-relevant milestones, retiring obsolete ones, and deriving new milestones as needed. This preserves valid work while remaining responsive to goal refinement.

---

## Agent Topology

Three agents operate in sequence within each loop iteration: **Planner**, **Executor**, **Verifier**.

### Methodology Synthesis

The design draws from several established agent architecture patterns:

**ADaPT (As-Needed Decomposition and Planning)** — The planner uses recursive decomposition. Rather than planning everything upfront, it decomposes milestones into sub-tasks only when the executor fails to complete them. This adapts naturally to both task complexity and model capability.

**Reflexion** — The system maintains a memory of what's been tried and what happened. After each loop, the verifier's structured feedback is stored and provided to the planner as context for the next iteration. The planner doesn't just retry — it retries with accumulated wisdom about what worked and what didn't.

**Plan-Validate-Execute (P-V-E)** — The verifier is an independent evaluation agent that uses a different prompt (and ideally a different model) than the executor. This avoids correlated errors — the same blindspot that caused an executor to produce flawed work won't cause the verifier to miss it.

### Planner

The planner is the loop controller. Its responsibilities:

- **First run**: Scan workspace to detect existing project vs. greenfield. Read `goals.md`. Derive milestones with measurable success criteria. Write initial state files.
- **Subsequent runs**: Check for `goals.md` changes (checksum comparison). If changed, perform diff-and-adapt reconciliation on milestones. Read verifier feedback and Reflexion memory. Determine the next milestone or sub-task for the executor. If a milestone was too complex (executor failed), decompose it further using ADaPT-style recursive decomposition.
- **Outputs**: Updated `MILESTONES.md`, current task assignment for executor, signal file `.milestone_ready`.

### Executor

The executor does the actual work. It takes a single milestone or sub-task from the planner's output and executes it — writing code, modifying files, running commands, creating artifacts.

- **Input**: Reads current task from planner's state files after detecting `.milestone_ready` signal.
- **Execution**: Performs the assigned work within the codebase.
- **Outputs**: Updated codebase, execution report in state files, signal file `.work_done`.

### Verifier

The verifier independently evaluates the executor's work against the success criteria derived from `goals.md`. It does not simply pass/fail — it produces structured feedback.

- **Input**: Reads executor's work and execution report after detecting `.work_done` signal. References success criteria from `MILESTONES.md`.
- **Evaluation**: Assesses what succeeded, what failed, why it failed, and what to try differently.
- **Outputs**: `VERIFIER_FEEDBACK.md` with structured assessment, updated `REFLEXION_MEMORY.md` with accumulated learnings, signal file `.verified`.

The verifier must use a different prompt than the executor. Using a different model is recommended to maximize error detection through diversity of perspective.

---

## Coordination

### Signal Files

Standard Switchboard signal-file pattern for sequencing agent execution:

| Signal File | Producer | Consumer | Meaning |
|---|---|---|---|
| `.milestone_ready` | Planner | Executor | New work is available |
| `.work_done` | Executor | Verifier | Executor has completed its task |
| `.verified` | Verifier | Planner | Verification is complete, feedback available |

### State Files

All state files live in `.switchboard/state/`:

| File | Owner | Purpose |
|---|---|---|
| `MILESTONES.md` | Planner | Current milestone list with success criteria, status, and decomposition history |
| `CURRENT_TASK.md` | Planner | The specific task assigned to the executor this iteration |
| `EXECUTION_REPORT.md` | Executor | What the executor did, what succeeded, what failed |
| `VERIFIER_FEEDBACK.md` | Verifier | Structured assessment of the executor's work |
| `REFLEXION_MEMORY.md` | Verifier | Accumulated learnings across loop iterations (append with pruning) |
| `GOALS_CHECKSUM` | Planner | Hash of goals.md for change detection |
| `WORKSPACE_PROFILE.md` | Planner | Result of initial workspace scan (existing project vs. greenfield, tech stack, structure) |

### State File Discipline

Per established Switchboard principles:
- State files are structured reference data with pruning rules, not session journals.
- `REFLEXION_MEMORY.md` should have a size budget — retain the N most recent and most relevant learnings, not an unbounded append log.
- Each agent detects current phase from existing state files and resumes correctly after interruption (idempotent resumption).

---

## Configuration

Goal-based workflows are configured in `switchboard.toml` like any other workflow. Agent instance counts are parameterized through environment variables.

Each role runs as a single agent instance. The planner and verifier are sequential bottlenecks by nature, and with tight loops the executor focuses on one milestone at a time. Parallelism can be revisited later if needed.

### Example switchboard.toml

```toml
[settings]
image_name = "switchboard-agent:latest"
timezone = "America/New_York"

# --- Goal-Based Planner ---
[[agent]]
name = "goal-planner"
schedule = "*/10 * * * *"
prompt_file = "prompts/goal-planner.md"

# --- Goal-Based Executor ---
[[agent]]
name = "goal-executor"
schedule = "*/10 * * * *"
prompt_file = "prompts/goal-executor.md"

# --- Goal-Based Verifier ---
[[agent]]
name = "goal-verifier"
schedule = "*/10 * * * *"
prompt_file = "prompts/goal-verifier.md"
```

---

## Loop Lifecycle

### First Run (Cold Start)

1. **Planner** detects no existing state files. Scans workspace to build `WORKSPACE_PROFILE.md`. Reads `goals.md`. Derives milestones with success criteria. Writes `MILESTONES.md`, `GOALS_CHECKSUM`, and first `CURRENT_TASK.md`. Signals `.milestone_ready`.

2. **Executor** detects `.milestone_ready`. Reads `CURRENT_TASK.md`. Performs the work. Writes `EXECUTION_REPORT.md`. Signals `.work_done`.

3. **Verifier** detects `.work_done`. Reads executor's report and the relevant milestone's success criteria. Evaluates the work. Writes `VERIFIER_FEEDBACK.md` and initializes `REFLEXION_MEMORY.md`. Signals `.verified`.

### Steady State Loop

1. **Planner** detects `.verified`. Reads verifier feedback. Checks `goals.md` checksum — if changed, performs diff-and-adapt on milestones. Consults `REFLEXION_MEMORY.md`. Decides next action:
   - If milestone passed verification → mark complete, assign next milestone.
   - If milestone failed → decompose further (ADaPT) or reassign with adjusted approach based on feedback.
   - If all milestones complete → mark workflow as done.
   Writes updated state files. Signals `.milestone_ready`.

2. **Executor** picks up new task, executes, reports, signals.

3. **Verifier** evaluates, feeds back, updates memory, signals.

### Goal Change Mid-Execution

1. **Planner** detects checksum mismatch on `goals.md`.
2. Reads new goals content. Compares against current `MILESTONES.md`.
3. Milestones still aligned with new goals → preserved with current status.
4. Milestones no longer relevant → retired.
5. New goals not covered by existing milestones → new milestones derived.
6. Writes updated `MILESTONES.md` and `GOALS_CHECKSUM`. Continues normal loop.

---

## Design Principles

These principles are drawn from accumulated learnings across Switchboard workflows:

- **Explicit negative guardrails outperform positive instructions.** Agent prompts need "Do NOT do X" rules to close loopholes.
- **File-based inter-agent contracts.** Repeating JSON schemas or document structures verbatim across dependent agents prevents drift. Signal files are the coordination primitive.
- **The verifier must differ from the executor.** Different prompt, ideally different model. Correlated errors are the enemy of verification.
- **State files need size budgets.** Reflexion memory must be pruned, not unbounded. Structured reference data, not session journals.
- **Task sizing matters.** Milestones and sub-tasks should be completable within agent session time limits. Rich upfront context in task descriptions pays off.
- **Archive/housekeeping at session start, not end.** Agents skip cleanup at end of sessions. Moving housekeeping to the start ensures it runs.
- **Idempotent resumption.** All agents detect current phase from existing state files and resume correctly after interruption.

---

## Open Questions

Items to resolve during implementation:

- **Reflexion memory pruning strategy**: Fixed window (last N entries)? Relevance-based? Size cap with LRU eviction?
- **Maximum decomposition depth**: How deep should ADaPT-style recursive decomposition go before declaring a milestone intractable?
- **Workflow completion signaling**: How does the system signal to the human that all goals are met? Discord notification? State file? Both?
- **Model selection per role**: Which models work best for planner vs. executor vs. verifier? Should this be configurable or hardcoded?