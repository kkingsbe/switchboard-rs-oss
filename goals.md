# Switchboard Observability — Design Spec

## Overview

A structured event logging and metrics system built into Switchboard's core binary. Every event is something Switchboard directly observes — container lifecycle and git activity. No agent self-reporting. No workflow-specific concepts.

Metrics are derived from these ground-truth events. The event log is the foundation — consumption (CLI, dashboard, scripts) is a separate concern.

---

## Design Principles

1. **Only measure what Switchboard can observe.** Containers starting, containers exiting, exit codes, duration, git commits. Nothing that requires trusting an agent's self-report.
2. **Workflow-agnostic.** No awareness of signal files, milestones, task types, or any coordination protocol. Works identically for goal-based, BMAD, academic pipeline, or any future workflow.
3. **Zero agent changes required.** Agents don't need to emit anything. Observability is purely a property of the orchestration layer.
4. **Git as ground truth for output.** The only reliable measure of what an agent actually produced is what it committed.

---

## Architecture

```
┌─────────────────────────────────────────────────┐
│                 Switchboard Core                 │
│                                                  │
│  Scheduler ──▶ Event Emitter ──▶ Event Log       │
│  Container     (structured)      (.jsonl file)   │
│  Manager                                         │
│  Git Differ                                      │
└─────────────────────────────────────────────────┘
```

---

## Event Log

### Location

```
.switchboard/events/events.jsonl
```

Append-only. One JSON object per line. No array wrapping.

### Rotation

When `events.jsonl` exceeds **10MB**, rotate:

```
events.jsonl      → events.2025-03-10T09-00-00Z.jsonl
(new empty)       → events.jsonl
```

Rotated files are kept for 30 days, then deleted.

### Event Schema

Every event shares a common envelope:

```json
{
  "ts": "2025-03-10T09:15:32.481Z",
  "event": "container.started",
  "agent": "goal-executor",
  "run_id": "a1b2c3d4",
  "data": { }
}
```

| Field | Type | Description |
|-------|------|-------------|
| `ts` | ISO 8601 UTC | When the event occurred |
| `event` | string | Event type (dot-namespaced) |
| `agent` | string \| null | Agent name from `switchboard.toml` (null for system events) |
| `run_id` | string \| null | Unique ID for this agent invocation (ties start/end events) |
| `data` | object | Event-type-specific payload |

---

## Event Types

### `scheduler.started`

Emitted when `switchboard up` begins.

```json
{
  "event": "scheduler.started",
  "agent": null,
  "run_id": null,
  "data": {
    "agents": ["goal-planner", "goal-executor", "goal-verifier", "skill-distiller"],
    "agent_count": 4,
    "version": "0.5.0",
    "config_file": "switchboard.toml"
  }
}
```

### `scheduler.stopped`

Emitted on graceful shutdown (Ctrl+C or SIGTERM).

```json
{
  "event": "scheduler.stopped",
  "agent": null,
  "run_id": null,
  "data": {
    "reason": "sigint",
    "uptime_seconds": 86400
  }
}
```

### `container.started`

Emitted when Switchboard launches an agent container.

```json
{
  "event": "container.started",
  "agent": "goal-executor",
  "run_id": "a1b2c3d4",
  "data": {
    "image": "kilosynth/prompter:latest",
    "trigger": "cron",
    "schedule": "*/5 * * * *",
    "container_id": "docker-container-hash"
  }
}
```

`trigger` values: `"cron"` (scheduled), `"manual"` (via `switchboard run <name>`).

### `container.exited`

Emitted when a container exits.

```json
{
  "event": "container.exited",
  "agent": "goal-executor",
  "run_id": "a1b2c3d4",
  "data": {
    "exit_code": 0,
    "duration_seconds": 847,
    "timeout_hit": false
  }
}
```

### `container.skipped`

Emitted when a cron trigger fires but the agent is skipped due to `overlap_mode = "skip"`.

```json
{
  "event": "container.skipped",
  "agent": "goal-executor",
  "run_id": null,
  "data": {
    "reason": "overlap_skip",
    "running_run_id": "a1b2c3d4"
  }
}
```

### `container.queued`

Emitted when a cron trigger fires and `overlap_mode = "queue"` is set, so the run is queued rather than skipped.

```json
{
  "event": "container.queued",
  "agent": "goal-executor",
  "run_id": "b2c3d4e5",
  "data": {
    "queue_position": 1,
    "running_run_id": "a1b2c3d4"
  }
}
```

### `git.diff`

Emitted after each agent container exits. Switchboard captures the git state before the container starts and after it exits, then diffs.

```json
{
  "event": "git.diff",
  "agent": "goal-executor",
  "run_id": "a1b2c3d4",
  "data": {
    "commit_count": 3,
    "commits": [
      {
        "hash": "abc1234",
        "message": "feat(executor): [M4] implement connector",
        "files_changed": 4,
        "insertions": 127,
        "deletions": 12
      },
      {
        "hash": "def5678",
        "message": "test(executor): [M4] unit tests for connector",
        "files_changed": 2,
        "insertions": 89,
        "deletions": 0
      },
      {
        "hash": "ghi9012",
        "message": "chore(executor): [M4] task complete — source connector",
        "files_changed": 1,
        "insertions": 3,
        "deletions": 1
      }
    ],
    "total_insertions": 219,
    "total_deletions": 13,
    "total_files_changed": 7
  }
}
```

**Capture method:** Before launching the container, record `HEAD` hash. After exit, run:

```bash
git log {before_hash}..HEAD --format="%H|%s" --numstat --no-merges
```

If `HEAD` hasn't moved (no commits), emit `git.diff` with `commit_count: 0` and empty arrays. This is itself a useful signal — the agent ran but produced no commits.

---

---

## Derived Metrics

These metrics can be computed from the event log by any consumer (CLI tool, script, dashboard). All are derived solely from core events — no agent cooperation required.

### Throughput & Velocity

| Metric | Computation |
|--------|-------------|
| **Agent runs** | Count of `container.started` events in window |
| **Productive runs** | Count of `container.exited` events where the corresponding `git.diff` has `commit_count > 0` |
| **Productive run rate** | Productive runs / agent runs |
| **Commits** | Sum of `git.diff.data.commit_count` across all runs in window |
| **Lines inserted** | Sum of `git.diff.data.total_insertions` |
| **Lines deleted** | Sum of `git.diff.data.total_deletions` |
| **Files changed** | Sum of `git.diff.data.total_files_changed` |
| **Avg run duration** | Mean of `container.exited.data.duration_seconds` |
| **Avg commits per run** | Commits / productive runs (excluding empty runs from denominator) |

### Reliability

| Metric | Computation |
|--------|-------------|
| **Container failures** | Count of `container.exited` where `exit_code != 0` |
| **Failure rate** | Container failures / total `container.exited` |
| **Timeouts** | Count of `container.exited` where `timeout_hit == true` |
| **Skipped runs** | Count of `container.skipped` events |
| **Empty runs** | Count of runs where `git.diff.data.commit_count == 0` AND `exit_code == 0` (agent ran successfully but produced nothing) |
| **Scheduler uptime** | Time from first `scheduler.started` to last event in window, minus any gaps between `scheduler.stopped` and next `scheduler.started` |

### Per-Agent Breakdown

All throughput and reliability metrics are also computed per agent (grouped by the `agent` field). This lets you spot which agent is failing, which is most productive, and which is wasting cycles.

---

## Configuration

```toml
[settings.observability]
enabled = true                          # Default: true
event_log_dir = ".switchboard/events"   # Default
max_log_size = "10MB"                   # Rotate after this size
retention_days = 30                     # Delete rotated files after N days
```

Minimal config surface. No workflow-specific knobs.

---

## Implementation Notes

### Event Emitter (Rust)

Lightweight struct passed to the scheduler and container manager. Owns a file handle to `events.jsonl`. Writes with append + flush-on-write semantics.

```rust
pub struct EventEmitter {
    file: std::fs::File,
    config: ObservabilityConfig,
}

impl EventEmitter {
    pub fn emit(&mut self, event: Event) -> Result<()> {
        let line = serde_json::to_string(&event)?;
        writeln!(self.file, "{}", line)?;
        self.file.flush()?;
        self.check_rotation()?;
        Ok(())
    }
}
```

The `Event` type is a Rust enum with serde tagging:

```rust
#[derive(Serialize)]
#[serde(tag = "event")]
pub enum EventData {
    #[serde(rename = "scheduler.started")]
    SchedulerStarted { agents: Vec<String>, agent_count: usize, version: String },

    #[serde(rename = "container.started")]
    ContainerStarted { image: String, trigger: String, schedule: String, container_id: String },

    #[serde(rename = "container.exited")]
    ContainerExited { exit_code: i32, duration_seconds: u64, timeout_hit: bool },

    #[serde(rename = "container.skipped")]
    ContainerSkipped { reason: String, running_run_id: String },

    #[serde(rename = "container.queued")]
    ContainerQueued { queue_position: u32, running_run_id: String },

    #[serde(rename = "git.diff")]
    GitDiff { commit_count: u32, commits: Vec<CommitInfo>, total_insertions: u32, total_deletions: u32, total_files_changed: u32 },

    #[serde(rename = "scheduler.stopped")]
    SchedulerStopped { reason: String, uptime_seconds: u64 },
}

#[derive(Serialize)]
pub struct Event {
    pub ts: String,
    #[serde(flatten)]
    pub data: EventData,
    pub agent: Option<String>,
    pub run_id: Option<String>,
}
```

### Git Diff Capture

Integrated into the container run lifecycle:

1. **Before container launch:** Record `HEAD` hash → `before_hash`
2. **After container exits:** Record `HEAD` hash → `after_hash`
3. **If `before_hash != after_hash`:** Run `git log {before_hash}..{after_hash} --format="%H|%s" --numstat --no-merges`, parse output
4. **If `before_hash == after_hash`:** Emit `git.diff` with `commit_count: 0`

One git command per agent run. Negligible overhead.

### Run ID Generation

Use a short random hex string (8 chars) generated at container launch. Ties together `container.started`, `container.exited`, and `git.diff` events for the same run.

```rust
fn generate_run_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("{:08x}", rng.gen::<u32>())
}
```

### Log Rotation

Check file size after each write. If over threshold:

1. Rename current file with timestamp suffix
2. Open new file
3. Scan for rotated files older than `retention_days`, delete them