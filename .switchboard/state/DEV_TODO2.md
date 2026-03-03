# DEV_TODO2 — Development Agent 2

> Sprint: 10
> Focus Area: Message Routing
> Last Updated: 2026-03-03
> Total Points: 3 pts

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `./skills/rust-engineer/SKILL.md`

## Stories

- [ ] **story-005-03**: Route Messages by Channel (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-03-route-by-channel.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [005-03] implement channel-based message routing`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
