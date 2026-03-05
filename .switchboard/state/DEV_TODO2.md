# DEV_TODO2 — Development Agent 2

> Sprint: 20
> Focus Area: Gateway Configuration & Channel Management
> Last Updated: 2026-03-05
> Total Points: 4

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `src/gateway/config.rs` — Current config implementation
- `src/gateway/registry.rs` — Current registry implementation

## Stories

- [ ] **{story-005-02}**: Support Channel Mapping in Config (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-02-channel-mapping-config.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(gateway): [story-005-02] support channel mapping in config`

- [ ] **{story-005-04}**: Runtime Channel Subscribe/Unsubscribe (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-04-runtime-channel-subscribe.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(gateway): [story-005-04] runtime channel subscribe/unsubscribe`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
