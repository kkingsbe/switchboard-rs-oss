# DEV_TODO1 — Development Agent 1

> Sprint: 10
> Focus Area: CLI Gateway Commands
> Last Updated: 2026-03-03
> Total Points: 5 pts

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
