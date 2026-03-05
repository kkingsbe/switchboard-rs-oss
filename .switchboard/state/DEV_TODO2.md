# DEV_TODO2 — Development Agent 2

> Sprint: 20
> Focus Area: Channel Configuration and Runtime Subscriptions (LOW RISK)
> Last Updated: 2026-03-05
> Total Points: 4

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-best-practices/SKILL.md`

## Stories

- [ ] **story-005-02**: Support Channel Mapping in Config (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-02-channel-mapping-config.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: LOW
  - 📝 Commit: `feat(dev2): [story-005-02] Support channel mapping in config`

- [ ] **story-005-04**: Support Runtime Channel Subscribe/Unsubscribe (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-04-runtime-channel-subscribe.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: LOW
  - 📝 Commit: `feat(dev2): [story-005-04] Support runtime channel subscribe/unsubscribe`

## Notes

- Both stories have LOW risk - straightforward implementation
- Dependencies already satisfied (4.2, 5.1 all COMPLETE)
- These two stories can be done in any order since they're independent

---

## AGENT QA

When all stories are complete, run:
```bash
cargo build --features "discord gateway"
cargo test --lib
cargo clippy -- -D warnings
```

If all green, create `.switchboard/state/.dev_done_2` with date.
If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
