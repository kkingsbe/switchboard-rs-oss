# DEV_TODO2 — Development Agent 2

> Sprint: 16
> Focus Area: Gateway Connection Management
> Last Updated: 2026-03-04
> Total Points: 3

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `src/gateway/mod.rs` (existing)
- `src/gateway/registry.rs` (existing)

## Stories

- [x] **{story-006-01}**: Project Connection Management (3 pts) ✅ queued for review
  - 📄 Story: `.switchboard/state/stories/story-006-01.md`
  - 📚 Skills: `./skills/rust-best-practices/SKILL.md`, `./skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Medium
  - 📝 Commit: `feat(dev2): [006-01] project connection management`

- [ ] **{story-006-01}** (REWORK): Project Connection Management
  - 📄 Story: `.switchboard/state/stories/story-006-01.md`
  - 🔍 Review: See REVIEW_QUEUE.md — CHANGES_REQUESTED
  - ⚠️ **MUST FIX:** Scope violation - revert changes to docker files:
    - `src/docker/build.rs`
    - `src/docker/run/run.rs`
    - `src/docker/skills.rs`
  - ✅ Post-check: Build + tests pass, only gateway files modified
  - 📝 Commit: `fix(dev2): [006-01] address review feedback - revert scope violations`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
