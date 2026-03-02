# DEV_TODO2 — Development Agent 2

> Sprint: 4
> Focus Area: OSS Release & Code Quality (expanded to include Test Infrastructure)
> Last Updated: 2026-03-02T15:35:00Z
> Total Points: 8 (5 completed + 3 rebalanced)

> ⚠️ Rebalanced by Sprint Planner on 2026-03-02 — TEST-FIX-01 moved from DEV_TODO1

## Orientation

Before starting any stories, read these files:
- `.switchboard/planning/PRD.md` — Product requirements
- `./skills/rust-engineer/SKILL.md` — Rust engineering guidelines

## Stories

- [x] **{story-2.3}**: Clean Up Committed Artifacts (2 pts)
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Check for build artifacts in git
  - ✅ Post-check: No build artifacts in version control
  - 🔒 Risk: Medium
  - 📝 Commit: `chore(oss-release): [2.3] clean up committed artifacts`

- [x] **{story-3.4}**: Clean Up Empty Feature Flags (1 pt)
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Check Cargo.toml for empty features
  - ✅ Post-check: No empty feature flags remain
  - 🔒 Risk: Low
  - 📝 Commit: `refactor(code-quality): [3.4] clean up empty feature flags`

- [x] **{story-4.1}**: Add Clippy and Formatting to CI (2 pts)
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Check .github/workflows for existing CI
  - ✅ Post-check: Clippy and fmt checks pass in CI
  - 🔒 Risk: Low
  - 📝 Commit: `ci(qol): [4.1] add clippy and formatting to CI`
  - ⚠️ Note: Previous attempt had scope violation - only modify CI files, not source code

- [ ] **{TEST-FIX-01}**: Fix Pre-existing Test Failures (3 pts)
  - 📄 Story: `.switchboard/state/stories/archive/sprint-3/story-test-fix-01.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/testing.md`
  - ⚡ Pre-check: Run `cargo test 2>&1 | head -100` to see current failures
  - ✅ Post-check: `cargo test` passes with 0 failures
  - 🔒 Risk: Medium
  - 📝 Commit: `fix(tests): [TEST-FIX-01] resolve pre-existing test failures`
  - 🎯 Note: CRITICAL - blocks stories 3.1 and 3.2. Rebalanced from DEV_TODO1.

- [ ] AGENT QA: Run `cargo test` and ensure all 547+ tests pass.
  - If green, create `.switchboard/state/.dev_done_2` with date.
  - If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
