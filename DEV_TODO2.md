# DEV_TODO2 — Development Agent 2

> Sprint: 7
> Focus Area: Channel Registry & Configuration
> Last Updated: 2026-03-03T12:05:06Z
> Total Points: 4

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `src/gateway/registry.rs` — Channel Registry patterns

## Stories

- [ ] **{story-005-01}**: Implement ChannelRegistry (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-01-channel-registry.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [story-005-01] Implement ChannelRegistry`

- [ ] **{story-005-05}**: Add Configuration Validation (1 pt)
  - 📄 Story: `.switchboard/state/stories/story-005-05-config-validation.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/references/error-handling.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [story-005-05] Add configuration validation`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
