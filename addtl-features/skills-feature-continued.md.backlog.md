# Skills Feature Continued — Backlog
> Feature Doc: ./addtl-features/skills-feature-continued.md
> Created: 2026-02-23
> Last Updated: 2026-02-23

## Remaining Tasks

### QA Verification (Agent 4)
- [ ] Complete QA verification for Container Skill Mounting (3.6)
- [ ] Complete QA verification for `switchboard validate` skill configuration checks (3.7)
- [ ] Create `.agent_done_4` file upon successful QA completion

### Skill Commands Implementation (3.3.2 - 3.3.4)
> Note: These commands appear to be implemented in src/commands/skills.rs but were not explicitly tracked in the sprint TODO files. Verify implementation completeness.

- [ ] Verify `switchboard skills install` (3.3.2) implementation matches feature spec:
  - Accepts `owner/repo@skill-name` format
  - Delegates to `npx skills add`, moves to `./skills/`
  - Updates lockfile, cleans up `.agents/skills/`
  - Handles `--yes` flag for overwrite confirmation

- [ ] Verify `switchboard skills installed` (3.3.3) implementation matches feature spec:
  - Scans `./skills/` for SKILL.md files
  - Parses YAML frontmatter
  - Cross-references lockfile for source info
  - Shows which agents have each skill assigned

- [ ] Verify `switchboard skills remove` (3.3.4) implementation matches feature spec:
  - Removes skill directory
  - Removes lockfile entry
  - Prompts for confirmation (--yes bypass)
  - Warns if skill is referenced in switchboard.toml

### Integration & Full Stack Verification
- [ ] Run full build and test suite to verify all skills commands work together
- [ ] Verify end-to-end flow: list → install → installed → remove
- [ ] Verify per-agent skill scoping works with container mounting
- [ ] Verify `switchboard validate` correctly checks skill configuration

---

## Summary

| Feature Requirement | Status | Notes |
|---------------------|--------|-------|
| 3.3.1 `switchboard skills list` | ✅ Complete | Agent 3 (.agent_done_3 exists) |
| 3.3.2 `switchboard skills install` | 🔍 Verify | In code, needs verification |
| 3.3.3 `switchboard skills installed` | 🔍 Verify | In code, needs verification |
| 3.3.4 `switchboard skills remove` | 🔍 Verify | In code, needs verification |
| 3.3.5 `switchboard skills update` | ✅ Complete | Agent 1 (.agent_done_1 exists) |
| 3.4 Per-agent skill scoping | ✅ Complete | Agent 2 (.agent_done_2 exists) |
| 3.1 Skill Discovery API | ✅ Complete | Agent 3 (.agent_done_3 exists) |
| 3.6 Container skill mounting | 🔄 QA Pending | Agent 4 (tasks complete, no .agent_done_4) |
| 3.7 Validate updates | 🔄 QA Pending | Agent 4 (tasks complete, no .agent_done_4) |

**All feature requirements are either complete or in progress. The remaining work is QA verification and implementation verification.**
