# TODO3 - Agent 3
<<<<<<< HEAD
> Sprint: 1
> Feature Doc: addtl-features/discord-agent.md
> Focus Area: Environment Variable Loading
> Last Updated: 2026-02-25T08:00:00Z

## Orientation

Before starting any tasks, read these files to understand the current codebase state:
- `Cargo.toml` — dependencies and feature flags
- `src/lib.rs` — module structure and public API
- `src/config/mod.rs` — existing config patterns

## Tasks

- [ ] Add env var loading for Discord token and LLM API key
  - 📚 SKILLS: Environment variable handling pattern
  - 🎯 Goal: Create a helper that loads environment variables based on config struct field names (token_env, api_key_env)
  - 📂 Files: `src/discord/config.rs` (add helper function)
  - 🧭 Context: The config structs store env var names (e.g., "DISCORD_TOKEN"), and we need a helper to load the actual values at runtime. This should return a Result with a clear error if the env var is missing. Follow Rust best practices for env var handling (std::env::var).
  - ✅ Acceptance: Helper function works, returns Err if env var missing, unit tests verify missing env vars produce helpful errors

=======
> Sprint: 6
> Feature Doc: addtl-features/skills-feature-continued.md
> Focus Area: Verify switchboard skills installed (3.3.3) and validate
> Last Updated: 2026-02-24T01:10:00Z

## Tasks

- [ ] Verify `switchboard skills installed` implementation matches feature spec:
  - Scans `./skills/` for SKILL.md files
  - Parses YAML frontmatter
  - Cross-references lockfile for source info
  - Shows which agents have each skill assigned
- [ ] Verify per-agent skill scoping with container mounting
>>>>>>> skills-improvements
- [ ] AGENT QA: Run full build and test suite. Fix ALL errors. If green, create '.agent_done_3' with the current date. If ALL '.agent_done_*' files exist for agents that had work this sprint, also create '.sprint_complete'.
