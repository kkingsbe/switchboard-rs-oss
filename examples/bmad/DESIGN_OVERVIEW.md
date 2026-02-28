# BMAD Method for Switchboard — Design Overview

## What This Is

An adaptation of the [BMAD Method](https://github.com/bmad-code-org/BMAD-METHOD) (Breakthrough Method for Agile AI-Driven Development) for Switchboard's autonomous, cron-scheduled agent system.

BMAD was designed for **interactive** AI development — a human loads an agent in an IDE, runs a workflow, reviews output, and iterates. Switchboard agents run **autonomously** on cron schedules inside Docker containers with no human in the loop during execution. This adaptation bridges that gap.

## Key Differences from Stock BMAD

| Aspect | Stock BMAD | Switchboard BMAD |
|--------|-----------|-----------------|
| **Execution** | Interactive IDE sessions | Autonomous cron + Docker |
| **Human role** | In-the-loop every workflow | Sets up planning artifacts, reviews async |
| **Agent communication** | Same chat context | File-based protocols (TODO lists, signals, state files) |
| **Workflow invocation** | Slash commands | Cron schedules with phase detection |
| **Context passing** | Conversation history | Planning artifacts on disk |
| **Coordination** | Human orchestrates | Signal files + sprint gates |

## Architecture

### BMAD Phases → Switchboard Agents

BMAD's 4 phases map to Switchboard agent roles:

```
Phase 1-3: PLANNING (Human-driven, done before `switchboard up`)
  └── Human uses BMAD interactively in their IDE to produce:
      - PRD.md, architecture.md, epics/, project-context.md

Phase 4: IMPLEMENTATION (Autonomous, Switchboard-scheduled)
  ├── sb-architect        — Sprint planning + story creation
  ├── sb-dev-1..N         — Story implementation
  ├── sb-code-reviewer    — Post-implementation quality gate
  ├── sb-scrum-master     — Sprint coordination + retrospectives
  └── sb-auditor          — Codebase health (existing, unchanged)
      sb-planner          — Improvement planning (existing, unchanged)
      sb-refactor-1..N    — Refactoring (existing, unchanged)
```

### Directory Layout

```
.switchboard/
├── prompts/
│   └── workflows/
│       ├── bmad/                          # BMAD implementation workflow
│       │   ├── ARCHITECT.md               # Sprint planner + story creator
│       │   ├── DEV_PARALLEL.md            # Story implementer
│       │   ├── CODE_REVIEWER.md           # Quality gate
│       │   └── SCRUM_MASTER.md            # Sprint coordinator
│       ├── codebase-maintenance/          # Existing (renamed from maintinance)
│       │   ├── AUDITOR.md
│       │   ├── IMPROVEMENT_PLANNER.md
│       │   └── REFACTOR_DEV.md
│       └── documentation/
│           └── SUMMARIZER.md
├── state/                                 # Runtime state (gitignored except planning)
│   ├── sprint-status.yaml                 # Current sprint state
│   ├── DEV_TODO1.md .. DEV_TODON.md       # Per-agent work queues
│   ├── .dev_done_1 .. .dev_done_N         # Agent completion signals
│   ├── .sprint_complete                   # Sprint gate
│   ├── .stories_ready                     # Architect → Dev handoff signal
│   ├── BLOCKERS.md                        # Cross-agent blocker log
│   └── review/                            # Code review state
│       ├── REVIEW_QUEUE.md
│       └── .review_done
└── logs/

_bmad-output/                              # BMAD planning artifacts (committed)
├── planning-artifacts/
│   ├── PRD.md
│   ├── architecture.md
│   ├── epics/
│   │   ├── epic-1.md
│   │   ├── epic-2.md
│   │   └── ...
│   └── project-context.md
└── implementation-artifacts/
    └── sprint-status.yaml                 # Canonical sprint status
```

### Workflow Lifecycle

```
Human (one-time setup):
  1. Install BMAD: npx bmad-method install
  2. Run Phase 1-3 interactively in IDE
  3. Copy planning artifacts to _bmad-output/
  4. Configure switchboard.toml
  5. switchboard up

Switchboard (autonomous loop):
  ┌─────────────────────────────────────────┐
  │  sb-scrum-master (hourly)               │
  │  └─ Checks sprint state                 │
  │  └─ Creates sprint-status.yaml if none  │
  │  └─ Detects sprint completion           │
  │  └─ Advances to next sprint             │
  ├─────────────────────────────────────────┤
  │  sb-architect (every 2 hours)           │
  │  └─ Reads sprint-status.yaml            │
  │  └─ Creates story files from epics      │
  │  └─ Distributes to DEV_TODO*.md         │
  │  └─ Signals .stories_ready              │
  ├─────────────────────────────────────────┤
  │  sb-dev-1..N (every 30 min)             │
  │  └─ Reads DEV_TODO{N}.md               │
  │  └─ Implements next story               │
  │  └─ Runs tests, commits                 │
  │  └─ Queues for review                   │
  │  └─ Signals .dev_done_{N} when empty    │
  ├─────────────────────────────────────────┤
  │  sb-code-reviewer (every 45 min)        │
  │  └─ Reads REVIEW_QUEUE.md              │
  │  └─ Reviews each implementation         │
  │  └─ Approves or rejects with notes      │
  │  └─ Rejected → back to DEV_TODO         │
  └─────────────────────────────────────────┘
```

## The Prompt Files

Each prompt file follows the proven patterns from your existing codebase-maintenance workflow:

1. **Role declaration** — Who this agent is
2. **Configuration** — File paths, env vars, state locations
3. **Golden Rule** — The one thing this agent must never violate
4. **Session Protocol** — Idempotency, resume from interruption
5. **Phase Detection** — What to do based on current state
6. **Phased Execution** — Step-by-step with time budgets
7. **Signal Protocol** — How to communicate with other agents
8. **Commit Convention** — Consistent git messages

## Scaling

The system scales by adjusting `AGENT_COUNT` and agent schedules in `switchboard.toml`. The file-based protocol means agents don't need to know about each other directly — they communicate through state files.