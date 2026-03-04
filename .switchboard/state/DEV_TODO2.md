# DEV_TODO2 — Development Agent 2

> Sprint: 12
> Focus Area: Gateway Logging, Gateway Down CLI, PID File
> Last Updated: 2026-03-04
> Total Points: 13 (2 done, 9 remaining after rebalancing)
> ⚠️ Rebalanced by Sprint Planner on 2026-03-04

## Orientation

Before starting any stories, read these files:
- `.switchboard/planning/project-context.md`
- `skills/rust-best-practices/SKILL.md`
- `skills/rust-engineer/SKILL.md`

## Stories (Sprint 12: 2026-03-04 to 2026-03-18)

### Completed Stories

- [x] **story-006-01**: Project Connections (3 pts) ✅ DONE
  - 📄 Story: `.switchboard/state/stories/story-006-01-project-connections.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [story-006-01] project connections`

- [x] **story-006-02**: Heartbeat Protocol (2 pts) ✅ DONE
  - 📄 Story: `.switchboard/state/stories/story-006-02-heartbeat-protocol.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [story-006-02] heartbeat protocol`

### Remaining Stories

- [ ] **story-007-04**: Gateway Logging (2 pts) [REBALANCED from dev-1]
  - 📄 Story: `.switchboard/state/stories/story-007-04-logging.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [story-007-04] gateway logging`

- [ ] **story-005-04**: Runtime Channel Subscribe (2 pts) [REBALANCED from dev-1]
  - 📄 Story: `.switchboard/state/stories/story-005-04-runtime-channel-subscribe.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [story-005-04] runtime channel subscribe`

- [ ] **story-006-04**: Handle Disconnections (2 pts) [REBALANCED from dev-1]
  - 📄 Story: `.switchboard/state/stories/story-006-04-handle-disconnections.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [story-006-04] handle disconnections`

- [ ] **story-007-02**: Gateway Down CLI (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-007-02-gateway-down-cli.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [story-007-02] gateway down CLI`

- [ ] **story-007-03**: PID File Management (1 pt)
  - 📄 Story: `.switchboard/state/stories/story-007-03-pid-file.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [story-007-03] PID file management`

---

- [ ] AGENT QA: Run full build and test suite. If green, create `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
