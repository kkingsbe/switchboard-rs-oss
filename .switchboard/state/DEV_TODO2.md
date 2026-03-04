# DEV_TODO2 — Development Agent 2

> Sprint: 11
> Focus Area: Message Routing & Gateway Protocol
> Last Updated: 2026-03-04
> Total Points: 8 pts

## Orientation

Before starting any stories, read these files:

- `.switchboard/planning/project-context.md`
- `.switchboard/planning/architecture.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/async.md`

## Stories

- [ ] **story-005-04**: Runtime Channel Subscribe/Unsubscribe (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-005-04-runtime-channel-subscribe.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [005-04] implement runtime channel subscribe/unsubscribe`

- [ ] **story-006-02**: Heartbeat Protocol (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-006-02-heartbeat-protocol.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [006-02] implement heartbeat protocol`

- [ ] **story-006-04**: Handle Disconnections (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-006-04-handle-disconnections.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`, `./skills/rust-engineer/references/async.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [006-04] handle project disconnections gracefully`

- [ ] **story-007-02**: CLI `gateway down` Command (2 pts)
  - 📄 Story: `.switchboard/state/stories/story-007-02-gateway-down-cli.md`
  - 📚 Skills: `./skills/rust-engineer/SKILL.md`
  - ⚡ Pre-check: Build + tests pass
  - ✅ Post-check: Build + tests pass, acceptance criteria met
  - 🔒 Risk: Low
  - 📝 Commit: `feat(dev2): [007-02] implement gateway down CLI command`

- [ ] AGENT QA: Run full build and test suite. If green, create
  `.switchboard/state/.dev_done_2` with date. If ALL `.dev_done_*`
  files exist for all agents with work, also create
  `.switchboard/state/.sprint_complete`.
