# TODO2 - Agent 2
<<<<<<< HEAD
> Sprint: 1
> Feature Doc: addtl-features/discord-agent.md
> Focus Area: Conversation Configuration
> Last Updated: 2026-02-25T08:00:00Z

## Orientation

Before starting any tasks, read these files to understand the current codebase state:
- `Cargo.toml` — dependencies and feature flags
- `src/lib.rs` — module structure and public API
- `src/config/mod.rs` — existing config parsing patterns

## Tasks

- [x] Create ConversationConfig struct (max_history, ttl_minutes)
  - 📚 SKILLS: Config parsing pattern from existing src/config/
  - 🎯 Goal: A `ConversationConfig` struct with fields: `max_history: usize`, `ttl_minutes: u64`
  - 📂 Files: `src/discord/config.rs` (update)
  - 🧭 Context: max_history controls how many message pairs to keep in conversation context. ttl_minutes is time-to-live for conversation state. Defaults should be max_history=20, ttl_minutes=60.
  - ✅ Acceptance: Struct defined with sensible defaults, serde deserialize works, unit tests pass

- [x] Parse `[discord]` section from switchboard.toml
  - 📚 SKILLS: Config parsing pattern from existing src/config/
  - 🎯 Goal: Integrate all Discord-related config (discord, discord.llm, discord.conversation) into the switchboard config system
  - 📂 Files: `src/config/mod.rs` (update to add Discord section), create integration test
  - 🧭 Context: This task depends on the config structs being defined. It adds the top-level [discord] section parsing. Look at how other top-level sections like [docker] or [scheduler] are parsed in src/config/.
  - ✅ Acceptance: Config parses correctly from test TOML, unit tests pass

- [x] AGENT QA: Run full build and test suite. Fix ALL errors. If green, create '.agent_done_2' with the current date. If ALL '.agent_done_*' files exist for agents that had work this sprint, also create '.sprint_complete'.
=======
> Sprint: 6
> Feature Doc: addtl-features/skills-feature-continued.md
> Focus Area: Verify switchboard skills remove (3.3.4) and E2E flow
> Last Updated: 2026-02-24T01:10:00Z

## Tasks

- [ ] Verify `switchboard skills remove` implementation matches feature spec:
  - Removes skill directory
  - Removes lockfile entry
  - Prompts for confirmation (--yes bypass)
  - Warns if skill is referenced in switchboard.toml
- [ ] Verify end-to-end flow: list → install → installed → remove
- [ ] AGENT QA: Run full build and test suite. Fix ALL errors. If green, create '.agent_done_2' with the current date. If ALL '.agent_done_*' files exist for agents that had work this sprint, also create '.sprint_complete'.
>>>>>>> skills-improvements
