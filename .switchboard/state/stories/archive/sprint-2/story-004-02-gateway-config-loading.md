# Story 4.2: Implement gateway configuration loading

> Epic: epic-04 — Discord Gateway Phase 1: Basic Gateway
> Points: 2
> Sprint: 2
> Type: feature
> Risk: low
> Created: 2026-03-02

## User Story

**As a** developer,
**I want** to load gateway configuration from a TOML file,
**So that** the gateway can be configured without hardcoding.

## Acceptance Criteria

1. Create `GatewayConfig` struct with fields: discord_token, server, logging, channels
   - **Test:** Unit tests pass for struct creation

2. Implement `GatewayConfig::load(path: Option<&str>)` to load from `gateway.toml`
   - **Test:** Can load sample config file without errors

3. Support environment variable expansion for discord_token (e.g., `${DISCORD_TOKEN}`)
   - **Test:** Token loaded from env var when specified in config as `${DISCORD_TOKEN}`

## Technical Context

### Architecture Reference

From architecture.md:
- GatewayConfig struct with fields: discord_token, server, logging, channels
- Configuration loading from TOML files
- Environment variable expansion for sensitive data

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Error Handling:** Use thiserror - never anyhow in library code
- **No unwrap()** in production - use `?` or `.expect()`
- **Logging:** Use tracing - never println!
- **Tests:** Inline in module files, naming: `test_name_should_do_something()`

### Existing Code Context

Pattern to follow from `src/config/mod.rs`:
```rust
// Example configuration loading pattern
pub struct Config {
    pub discord: DiscordConfig,
    pub docker: DockerConfig,
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        // ...
    }
}
```

## Implementation Plan

1. Create `src/gateway/config.rs`
2. Define GatewayConfig struct with:
   - `discord_token: String`
   - `server: ServerConfig` (port, host)
   - `logging: LoggingConfig` (level, output)
   - `channels: HashMap<String, ChannelConfig>`
3. Implement GatewayConfig::load(path: Option<&str>)
4. Implement environment variable expansion for `${VAR}` patterns
5. Add required dependencies to Cargo.toml: toml, serde
6. Write unit tests
7. Run tests: `cargo test --lib`
8. Run lint: `cargo clippy -- -D warnings`

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Error handling with thiserror
- `./skills/rust-engineer/SKILL.md` — Async Rust patterns
- `./skills/rust-engineer/references/testing.md` — Testing standards

### Dependencies

- **Story 4.1**: Must complete first (module structure must exist)

## Scope Boundaries

### This Story Includes
- Creating `src/gateway/config.rs`
- Defining GatewayConfig struct
- Implementing load() method with TOML parsing
- Environment variable expansion
- Unit tests

### This Story Does NOT Include
- HTTP server implementation (story 4.3)
- WebSocket server
- Discord connection logic
- CLI integration

### Files in Scope
- `src/gateway/config.rs` — create
- `Cargo.toml` — modify (add toml, serde dependencies)

### Files NOT in Scope
- `src/gateway/server.rs` — story 4.3
- `src/gateway/protocol.rs` — story 4.5
- CLI commands
