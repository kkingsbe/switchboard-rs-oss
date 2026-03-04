# DEV_TODO1 — Development Agent 1

> Sprint: 12
> Focus Area: CLI Infrastructure (PID file, logging, Docker Trait)
> Last Updated: 2026-03-04
> Total Points: 6 (3 + 3 rebalanced from dev-2)

## Orientation

Before starting any stories, read these files:
- `.switchboard/planning/project-context.md`
- `skills/rust-best-practices/SKILL.md`
- `skills/rust-engineer/SKILL.md`

## Stories

- [ ] **story-007-03**: PID File Management (1 pt)
  - 📄 Story: `.switchboard/state/stories/story-007-03-pid-file.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-007-03] PID file management`

- [ ] **story-007-04**: Gateway Logging (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-007-04-logging.md`
  - 📚 Skills: `skills/rust-engineer/SKILL.md`, `skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-007-04] gateway logging`

- [ ] **story-001-docker**: Docker Connection Trait (3 pts) [REBALANCED from dev-2]
  - 📄 Story: `.switchboard/state/stories/story-001-docker-connection-trait.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-001-docker] Docker connection trait`

- [ ] AGENT QA: Run full build and test suite. If green, create `.switchboard/state/.dev_done_1` with date. If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
