# DEV_TODO2 — Development Agent 2

> Sprint: 2
> Focus Area: Rust refactoring
> Last Updated: 2026-03-01T20:55:00Z
> Total Points: 11 (5 existing + 5 rebalanced + 1)

## Stories

- [x] **{5.1}**: Clean Up Commit History (1 pt) ✅ COMPLETED

- [ ] **{3.1}**: Decompose src/docker/run/run.rs (5 pts)
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass
  - 🔒 Risk: Medium
  - 📝 Commit: `refactor(dev2): [3.1] decompose docker run module`

- [ ] **{3.2}**: Decompose src/config/mod.rs (5 pts) [REBALANCED from dev-1]
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass
  - 🔒 Risk: Medium
  - 📝 Commit: `refactor(dev2): [3.2] decompose config module`

- [ ] **{3.4}**: Clean Up Empty Feature Flags (1 pt)
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass
  - 🔒 Risk: Low
  - 📝 Commit: `chore(dev2): [3.4] clean up empty feature flags`

> ⚠️ Rebalanced by Sprint Planner on 2026-03-01

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2`. If ALL `.dev_done_*` files exist,
  also create `.switchboard/state/.sprint_complete`.
