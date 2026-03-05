# DEV_TODO1 — Development Agent 1

> Sprint: 20
> Focus Area: Discord Gateway Core Connection
> Last Updated: 2026-03-05
> Total Points: 5

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `src/gateway/server.rs` — Current implementation
- `src/discord/gateway.rs` — Reference implementation

## Stories

- [ ] **{story-004-07}**: Wire up Discord Gateway Connection (5 pts)
  - 📄 Story: `.switchboard/state/stories/story-004-07-discord-gateway.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: High
  - 📝 Commit: `feat(gateway): [story-004-07] wire up Discord gateway connection`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_1` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
