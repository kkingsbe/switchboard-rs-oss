# DEV_TODO1 — Development Agent 1

> Sprint: 7
> Focus Area: Discord Gateway (blocked - needs rework)
> Last Updated: 2026-03-03
> Total Points: 5
> ⚠️ Rebalanced by Sprint Planner on 2026-03-03

## Orientation

Before starting, read the review feedback in REVIEW_QUEUE.md for story-004-03 and story-004-07.

## Stories

- [x] **{story-004-07}**: Discord Gateway Connection (5 pts) — QUEUED_FOR_REVIEW
  - 📄 Story: `.switchboard/state/stories/story-004-07-*.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: `cargo build --tests 2>&1 | head -50`
  - ✅ Post-check: All tests compile, no warnings, `cargo fmt --check` passes
  - 🔒 Risk: High
  - 📝 Commit: `fix(dev1): [story-004-07] address review feedback - formatting + unwrap_or`

- [x] **{story-004-03}**: HTTP Server with Health Check (3 pts) — QUEUED_FOR_REVIEW
  - 📄 Story: `.switchboard/state/stories/archive/sprint-5/story-004-03-http-server-health-check.md`
  - 📚 Skills: `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build and tests pass
  - ✅ Post-check: `cargo fmt --check` passes
  - 📝 Commit: `fix(dev1): [story-004-03] run cargo fmt to fix formatting`

## Review Feedback Summary (from REVIEW_QUEUE.md)

Both stories share `src/gateway/server.rs`. Fixes needed:

1. **Formatting** — Run `cargo fmt` to fix:
   - Lines 353, 385, 392, 487, 495, 527, 541

2. **unwrap_or in production** (story-004-07 only):
   - Line 511: `channel_id.parse().unwrap_or(0)`
   - Replace with proper error handling (use `?` or match)

## Blocked Stories (moved to Dev 2)

The following stories were moved to DEV_TODO2 for rebalancing:
- story-005-01: ChannelRegistry (3 pts)
- story-005-05: Config Validation (1 pt)
