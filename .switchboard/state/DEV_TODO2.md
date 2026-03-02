# DEV_TODO2 — Development Agent 2

> Sprint: 4
> Focus Area: OSS Release & Code Quality
> Last Updated: 2026-03-02T09:55:00Z
> Total Points: 5

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

- [ ] AGENT QA: Run `cargo build` and `cargo test`.
  - If green, create `.switchboard/state/.dev_done_2` with date.
  - If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
