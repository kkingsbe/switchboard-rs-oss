# DEV_TODO1 — Development Agent 1

> Sprint: 20
> Focus Area: Discord Gateway Connection (HIGH RISK)
> Last Updated: 2026-03-05
> Total Points: 5

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-best-practices/SKILL.md`

## Stories

- [x] **story-004-07**: Wire up Discord Gateway Connection (5 pts) ✅ queued for review
  - 📄 Story: `.switchboard/state/stories/story-004-07-discord-gateway.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-best-practices/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: HIGH
  - 📝 Commit: `feat(dev1): [story-004-07] Wire up Discord Gateway connection`

## Notes

- This story has HIGH risk - prioritize careful implementation
- Depends on: Story 4.2 (COMPLETE), Story 4.6 (COMPLETE)
- Review: `./skills/rust-engineer/references/async.md` for async patterns

---

## AGENT QA

When story is complete, run:
```bash
cargo build --features "discord gateway"
cargo test --lib
cargo clippy -- -D warnings
```

If all green, create `.switchboard/state/.dev_done_1` with date.
If ALL `.dev_done_*` files exist for all agents with work, also create `.switchboard/state/.sprint_complete`.
