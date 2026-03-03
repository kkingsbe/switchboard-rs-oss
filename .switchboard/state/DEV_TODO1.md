# DEV_TODO1 — Development Agent 1

> Sprint: 8
> Focus Area: CLI and Logging
> Last Updated: 2026-03-03
> Total Points: 5

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md` (Discord Gateway section)

## Stories

- [ ] **story-004-08**: CLI `gateway up` Command (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-004-08-gateway-up-cli.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [004-08] CLI gateway up command`

- [ ] **story-007-04**: Proper Logging (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-007-04-gateway-logging.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [007-04] Gateway logging`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_1` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
