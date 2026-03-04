# DEV_TODO1 — Development Agent 1

> Sprint: 10
> Focus Area: CLI Gateway Commands
> Last Updated: 2026-03-04
> Total Points: 5 pts
> ⚠️ Rebalanced by Sprint Planner on 2026-03-04 - removed stale REWORK entry

## 🚨 REWORK QUEUE (from previous sprints)

- [ ] **story-006-01** (REWORK - 2nd review round): Project Connection Management
  - 📄 Story: `.switchboard/state/stories/archive/sprint-6/story-006-01-project-connections.md`
  - 🔍 Review: See `.switchboard/state/review/REVIEW_QUEUE.md` — CHANGES_REQUESTED (scope violations NOT fixed)
  - ⚠️ MUST FIX: Revert out-of-scope changes to:
    - `src/discord/gateway.rs`
    - `src/gateway/ratelimit.rs`
    - `src/gateway/registry.rs`
    - `src/gateway/routing.rs`
    - `src/logging.rs`
  - ✅ Post-check: Only `src/gateway/connections.rs` and `src/gateway/mod.rs` modified
  - 📝 Commit: `fix(dev1): [006-01] REVERT out-of-scope changes (FINAL ATTEMPT)`

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `./skills/rust-engineer/SKILL.md`

## Stories

- [ ] **story-004-08**: CLI `gateway up` Command (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-004-08-gateway-up-cli.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [004-08] implement gateway up CLI command`

- [ ] **story-007-01**: CLI `gateway status` Command (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-007-01-gateway-status.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [007-01] implement gateway status CLI command`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_1` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
