# DEV_TODO2 — Development Agent 2

> Sprint: 8
> Focus Area: Gateway Routing and Rate Limiting
> Last Updated: 2026-03-03
> Total Points: 5

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md` (Discord Gateway section)

## Stories

- [ ] **story-005-03**: Route Messages by Channel (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-03-route-by-channel.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`, `./skills/DISCLI.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [005-03] Route messages by channel`

- [ ] **story-006-06**: Rate Limiting (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-006-06-rate-limiting.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [006-06] Rate limiting`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
