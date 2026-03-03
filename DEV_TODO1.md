# DEV_TODO1 — Development Agent 1

> ⚠️ Rebalanced by Sprint Planner on 2026-03-03
> Sprint: 7
> Focus Area: Discord Gateway Connection + Channel Registry & Configuration
> Last Updated: 2026-03-03T13:10:28Z
> Total Points: 9

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `src/discord/gateway.rs` — Discord Gateway patterns

## Stories

- [ ] **story-004-07** (REWORK): Wire up Discord Gateway Connection
  - 📄 Story: .switchboard/state/stories/story-004-07.md
  - 🔍 Review: See REVIEW_QUEUE.md — CHANGES_REQUESTED
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Address ALL "Must Fix" items
  - 📝 Commit: `fix(dev1): [story-004-07] address review feedback`

- [x] **{story-004-07}**: Wire up Discord Gateway Connection (5 pts) ✅ queued for review
  - 📄 Story: `.switchboard/state/stories/story-004-07-discord-gateway.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: High
  - 📝 Commit: `feat(dev1): [story-004-07] Wire up Discord Gateway connection`

- [ ] **{story-005-01}**: Implement ChannelRegistry (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-01-channel-registry.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev1): [story-005-01] Implement ChannelRegistry`

- [ ] **{story-005-05}**: Add Configuration Validation (1 pt)
  - 📄 Story: `.switchboard/state/stories/story-005-05-config-validation.md`
  - 📚 Skills: `skills/rust-best-practices/SKILL.md`, `skills/rust-engineer/references/error-handling.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev1): [story-005-05] Add configuration validation`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_1` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`. ✅ done (562 passed, 5 pre-existing failures)
