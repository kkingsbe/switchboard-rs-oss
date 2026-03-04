# DEV_TODO1 — Development Agent 1

> Sprint: 12
> Focus Area: CLI Infrastructure (Docker Trait, Logging)
> Last Updated: 2026-03-04
> Total Points: 13 (5 done, 8 remaining after rebalancing)
> ⚠️ Rebalanced by Sprint Planner on 2026-03-04

## Orientation

Before starting any stories, read these files:
- `.switchboard/planning/project-context.md`
- `skills/rust-best-practices/SKILL.md`
- `skills/rust-engineer/SKILL.md`

## Stories (Sprint 12: 2026-03-04 to 2026-03-18)

### Completed Stories

- [x] **story-004-08**: Gateway Up CLI (3 pts) ✅ DONE
  - 📄 Story: `.switchboard/state/stories/story-004-08-gateway-up-cli.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-004-08] gateway up CLI`

- [x] **story-007-01**: Gateway Status (2 pts) ✅ DONE
  - 📄 Story: `.switchboard/state/stories/story-007-01-gateway-status.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-007-01] gateway status`

### Remaining Stories

- [ ] **story-001-docker-connection-trait**: Docker Connection Trait (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-001-docker-connection-trait.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-001-docker-connection-trait] Docker connection trait`

- [ ] **story-006-04**: Handle Disconnections (2 pts) [REBALANCED from dev-2]
  - 📄 Story: `.switchboard/state/stories/story-006-04-handle-disconnections.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev1): [story-006-04] handle disconnections`

- [ ] **story-006-03**: Reconnection Logic (3 pts) [REBALANCED from dev-2]
  - 📄 Story: `.switchboard/state/stories/story-006-03-reconnection-logic.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`, `skills/rust-engineer/references/error-handling.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev1): [story-006-03] reconnection logic`

---

- [ ] AGENT QA: Run full build and test suite. If green, create `.switchboard/state/.dev_done_1` with date. If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
