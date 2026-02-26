# TODO1 - Agent 1
<<<<<<< HEAD
> Sprint: 1
> Feature Doc: addtl-features/discord-agent.md
> Focus Area: Discord Configuration
> Last Updated: 2026-02-25T08:00:00Z

## Orientation

Before starting any tasks, read these files to understand the current codebase state:
- `Cargo.toml` — dependencies and feature flags
- `src/lib.rs` — module structure and public API
- `src/config/mod.rs` — existing config parsing patterns

## Tasks

- [x] Create DiscordConfig struct (enabled, token_env, channel_id)
  - 📚 SKILLS: Config parsing pattern from existing src/config/
  - 🎯 Goal: A `DiscordConfig` struct with fields: `enabled: bool`, `token_env: String`, `channel_id: u64`
  - 📂 Files: `src/discord/config.rs` (create), `src/discord/mod.rs` (update)
  - 🧭 Context: The Discord Bot token should be loaded from an environment variable (name specified in token_env). Follow the pattern used for other config sections in src/config/. The feature doc specifies this goes in src/discord/config.rs
  - ✅ Acceptance: Struct defined, serde deserialize works, unit tests pass

- [x] Create LlmConfig struct (provider, api_key_env, model, max_tokens, system_prompt_file)
  - 📚 SKILLS: Config parsing pattern from existing src/config/
  - 🎯 Goal: A `LlmConfig` struct with fields: `provider: String`, `api_key_env: String`, `model: String`, `max_tokens: u32`, `system_prompt_file: Option<String>`
  - 📂 Files: `src/discord/config.rs` (update)
  - 🧭 Context: The LLM API key should also come from environment variable (api_key_env). Default provider should be "openrouter". Follow the same pattern as DiscordConfig.
  - ✅ Acceptance: Struct defined, serde deserialize works, unit tests pass

- [x] AGENT QA: Run full build and test suite. Fix ALL errors. If green, create '.agent_done_1' with the current date. If ALL '.agent_done_*' files exist for agents that had work this sprint, also create '.sprint_complete'.
=======
> Sprint: 6
> Feature Doc: addtl-features/skills-feature-continued.md
> Focus Area: Verify switchboard skills install (3.3.2) implementation
> Last Updated: 2026-02-24T01:10:00Z

## Tasks

- [ ] [BUILDFIX] Fix failing tests to make build healthy
  - 📚 SKILLS: (use your judgment based on what's failing)
  - 🎯 Goal: Fix all 18 failing tests so `cargo test` exits 0
  - 📂 Files: Look at the test files mentioned in the errors:
    - src/discord/tools.rs (discord tools tests)
    - src/discord/config.rs (discord config test)  
    - src/docker/run/run.rs (docker integration tests)
    - src/config.rs (switchboard toml parsing test)
  - 🧭 Context: The tests are failing due to:
    1. Missing skills fixtures - integration tests look for ./skills/ directory with specific skill names (repo, repo1, etc.)
    2. File system issues in discord tools tests - tests expect files at certain paths
    3. Environment/config issues in discord config tests
  - ✅ Acceptance: `cargo test` exits with code 0 (all tests pass)

- [ ] Verify `switchboard skills install` implementation matches feature spec:
  - Accepts `owner/repo@skill-name` format
  - Delegates to `npx skills add`, moves to `./skills/`
  - Updates lockfile, cleans up `.agents/skills/`
  - Handles `--yes` flag for overwrite confirmation
- [ ] Run full build and test suite to verify implementation
- [ ] AGENT QA: Run full build and test suite. Fix ALL errors. If green, create '.agent_done_1' with the current date. If ALL '.agent_done_*' files exist for agents that had work this sprint, also create '.sprint_complete'.
>>>>>>> skills-improvements
