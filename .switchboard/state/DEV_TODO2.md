# DEV_TODO2 — Development Agent 2

> Sprint: 3
> Focus Area: Rust refactoring / Error handling
> Last Updated: 2026-03-02T02:28:00Z
> Total Points: 5

## Stories

- [x] **{3.3}** (REWORK): Replace .unwrap() Calls with Proper Error Handling (5 pts)
  - 📄 Story: `.switchboard/state/stories/story-3-3-unwrap-refactor.md`
  - 🔍 Review: See REVIEW_QUEUE.md — CHANGES REQUESTED
  - ⚡ Pre-check: Build + tests pass, cargo clippy -- -D warnings passes
  - ✅ Post-check: Address ALL "Must Fix" items from review:
    1. Fix clippy error: Replace `.map_or()` with `.is_some_and()` at src/discord/conversation.rs:198
    2. Revert scope violations: Restore `.switchboard/heartbeat.json` and `.switchboard/state/SPRINT_REPORT.md` to their pre-commit state
  - 📝 Commit: `fix(dev2): [3.3] address review feedback - clippy fix + scope revert`

- [x] **{3.3}**: Replace .unwrap() Calls with Proper Error Handling (5 pts) — REWORKED
  - 📄 Story: `.switchboard/state/stories/story-3-3-unwrap-refactor.md`
  - 📚 Skills: 
    - `./skills/rust-best-practices/SKILL.md`
    - `./skills/rust-best-practices/references/chapter_04.md`
    - `./skills/rust-engineer/references/error-handling.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, no .unwrap() in production code
  - 🔒 Risk: Medium
  - 📝 Commit: `refactor(dev2): [3.3] replace unwrap with proper error handling`

- [ ] **{2.3}**: Clean Up Committed Artifacts (2 pts)
  - 📄 Story: See sprint-status.yaml for details
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass
  - 🔒 Risk: Medium
  - 📝 Commit: `chore(dev2): [2.3] clean up committed artifacts`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2`. If ALL `.dev_done_*` files exist,
  also create `.switchboard/state/.sprint_complete`.
