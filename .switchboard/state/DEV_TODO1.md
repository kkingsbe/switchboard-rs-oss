# DEV_TODO1 — Development Agent 1

> Sprint: 2
> Focus Area: File cleanup + Test fixes
> Last Updated: 2026-03-01T21:19:00Z
> Total Points: 5 (2 + 3)

## Stories

- [ ] **{TEST-FIX-01}**: Fix Pre-existing Test Failures (3 pts)
  - 📄 Story: `.switchboard/state/stories/story-test-fix-01.md`
  - ⚡ Pre-check: Build passes
  - ✅ Post-check: `cargo test` passes with 0 failures
  - 🔒 Risk: Medium
  - 📝 Commit: `test(dev1): [TEST-FIX-01] fix pre-existing test failures`
  - ⚠️ CRITICAL: This unblocks stories 3.1, 3.2 which require passing tests

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_1`.
