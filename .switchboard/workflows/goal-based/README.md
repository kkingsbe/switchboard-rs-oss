# Goal-Based Workflows — Implementation Guide

## Quick Start

1. Write your goals in `goals.md` at the project root
2. Copy the prompt files to `.switchboard/prompts/workflows/goal-based/`
3. Copy `switchboard.toml` (or merge the agent entries into your existing config)
4. Run `switchboard up`

The planner will auto-detect your workspace, derive milestones, and begin the loop.

## Directory Layout

```
project-root/
├── goals.md                              # YOUR INPUT (the only thing you provide)
│
├── .switchboard/
│   ├── state/                            # Runtime state (agent coordination)
│   │   ├── MILESTONES.md                 # Planner-derived milestones + success criteria
│   │   ├── CURRENT_TASK.md               # Active task assignment for executor
│   │   ├── EXECUTION_REPORT.md           # Executor's report on completed work
│   │   ├── VERIFIER_FEEDBACK.md          # Verifier's structured assessment
│   │   ├── REFLEXION_MEMORY.md           # Accumulated loop learnings (max 20 entries)
│   │   ├── WORKSPACE_PROFILE.md          # Auto-detected workspace characteristics
│   │   ├── GOALS_CHECKSUM                # SHA-256 of goals.md for change detection
│   │   │
│   │   │  # Signal files (coordination)
│   │   ├── .milestone_ready              # Planner → Executor: work available
│   │   ├── .work_done                    # Executor → Verifier: ready for review
│   │   ├── .verified                     # Verifier → Planner: feedback ready
│   │   └── .goals_complete               # All goals satisfied
│   │
│   ├── prompts/
│   │   └── workflows/
│   │       └── goal-based/
│   │           ├── GOAL_PLANNER.md
│   │           ├── GOAL_EXECUTOR.md
│   │           └── GOAL_VERIFIER.md
│   │
│   └── logs/
│
├── switchboard.toml
└── skills/                               # Optional: reusable skill documents
```

## Signal Protocol

All coordination happens through signal files. No agent reads another agent's
internal state or prompt. The protocol:

```
.milestone_ready   ← Planner creates when task is assigned
.work_done         ← Executor creates when work is complete
.verified          ← Verifier creates when feedback is written
.goals_complete    ← Planner creates when all milestones pass
```

**Guard rails:**
- Executor won't start without `.milestone_ready`
- Verifier won't start without `.work_done`
- Planner waits for `.verified` before processing feedback
- Each agent cleans up the signal it consumed

## Agent Lifecycle

```
┌─────────────────────────────────────────┐
│             COLD START                   │
│                                          │
│  Planner                                 │
│  ├─ Scan workspace → WORKSPACE_PROFILE   │
│  ├─ Read goals.md                        │
│  ├─ Derive milestones → MILESTONES.md    │
│  ├─ Assign first task → CURRENT_TASK.md  │
│  └─ Signal: .milestone_ready             │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│           EXECUTION LOOP                 │
│                                          │
│  Executor                                │
│  ├─ Read CURRENT_TASK.md                 │
│  ├─ Read skills (if referenced)          │
│  ├─ Execute work (code, tests, builds)   │
│  ├─ Write EXECUTION_REPORT.md            │
│  └─ Signal: .work_done                   │
│                                          │
│  Verifier                                │
│  ├─ Read task + report + actual code     │
│  ├─ Inspect work against criteria        │
│  ├─ Run build/tests independently        │
│  ├─ Write VERIFIER_FEEDBACK.md           │
│  ├─ Update REFLEXION_MEMORY.md           │
│  └─ Signal: .verified                    │
│                                          │
│  Planner                                 │
│  ├─ Read verifier feedback               │
│  ├─ Check for goals.md changes           │
│  ├─ Update milestone status              │
│  ├─ Update reflexion memory              │
│  ├─ Decide: next milestone / retry /     │
│  │          decompose / done             │
│  ├─ Write CURRENT_TASK.md                │
│  └─ Signal: .milestone_ready             │
│                                          │
│  [loop until all milestones COMPLETE]    │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│           COMPLETION                     │
│                                          │
│  Planner signals .goals_complete         │
│  All agents detect and STOP              │
└─────────────────────────────────────────┘
```

## Key Mechanisms

### ADaPT Decomposition

When a milestone fails 3 consecutive times, the planner decomposes it into
2-4 smaller sub-milestones. Each sub-milestone is independently verifiable
and completable in a single executor session. Decomposition can recurse up
to depth 3 — beyond that, the milestone is marked BLOCKED for human review.

### Reflexion Memory

After every verification, a structured entry is added to REFLEXION_MEMORY.md
recording what was tried, what happened, and what was learned. The planner
reads this before every task assignment to avoid repeating mistakes. Memory
is capped at 20 entries (oldest pruned first).

### Goal Change Detection

The planner checksums goals.md on every run. When a change is detected, it
reconciles: keeping valid progress, retiring obsolete milestones, and deriving
new ones. Completed work that's still relevant is never thrown away.

## Writing Good Goals

The quality of goals.md directly affects how well the system performs.

**Good goals:**
```markdown
# Goals

## Reduce API response latency
- P95 latency for /api/users should be under 200ms
- Database queries should use indexes, no full table scans
- Add response time logging middleware

## Add user authentication
- JWT-based auth with access and refresh tokens
- Login, logout, and token refresh endpoints
- Protected routes return 401 without valid token
```

**Vague goals (will still work, but milestones will be coarser):**
```markdown
# Goals

Make the API faster and add login.
```

**Tips:**
- Specific success criteria in goals.md become milestone success criteria almost verbatim
- Mentioning specific technologies or approaches constrains the executor helpfully
- Multiple goals are fine — the planner will interleave milestones as needed
- You can update goals.md while the system is running — it will adapt