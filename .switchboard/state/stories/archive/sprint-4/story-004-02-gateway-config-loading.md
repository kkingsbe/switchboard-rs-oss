# Story 004-02: Implement Gateway Configuration Loading

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 2
> Sprint: 4
> Type: feature
> Risk: Low
> Created: 2026-03-02T23:05:00Z

## User Story

**As a** developer,
**I want** to load gateway configuration from a TOML file,
**So that** the gateway can be configured without hardcoding.

## Acceptance Criteria

1. Create `GatewayConfig` struct with fields: discord_token, server, logging, channels
   - **Test:** Unit tests pass
2. Implement `GatewayConfig::load(path: Option<&str>)` to load from `gateway.toml`
   - **Test:** Can load sample config file
3. Support environment variable expansion for discord_token (e.g., `${DISCORD_TOKEN}`)
   - **Test:** Token loaded from env var when specified

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md` Section 5.1 (gateway::config):

- **Purpose:** Load and validate gateway configuration
- **Public API:**
  - `GatewayConfig::load(path: Option<&str>) -> Result<Self, ConfigError>`
  - `GatewayConfig::from_env() -> Result<Self, ConfigError>`
- **Dependencies:** toml, serde
- **Data flow:** TOML file → parse → validated config struct

**Data Model:**
```rust
struct GatewayConfig {
    discord_token: String,      // Bot token
    server: ServerConfig,        // HTTP/WS ports
    logging: LoggingConfig,     // Logging settings
    channels: Vec<ChannelMapping>,
}

struct ChannelMapping {
    channel_id: String,
    project_name: String,
    endpoint: String,            // Project's WebSocket endpoint
}
```

### Project Conventions

From `.switchboard/planning/project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Format:** `cargo fmt`
- **Language:** Rust 2021 edition
- **Error Handling:** Use `thiserror` for error types - follow patterns in `src/discord/gateway.rs`
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages
- **Module organization:** New gateway code goes in `src/gateway/`
- **Configuration:** Use TOML config files with serde. Follow patterns in `src/config/`
- **Testing:** Place unit tests in the same file as the code. Use descriptive test names
- **Logging:** Use `tracing` for logging. Never use `println!` or `eprintln!`

### Existing Code Context

**Config module pattern** (from `src/config/mod.rs`):
- Uses `thiserror` for ConfigError enum
- Implements `FromStr` trait for config parsing
- Supports TOML file loading with `std::fs` and `toml::from_str`
- Environment variable expansion is handled by `crate::config::env` module

**Gateway module** (from `src/gateway/mod.rs`):
```rust
//! Gateway module for Discord Gateway Service
//!
//! This module provides the core functionality for the Discord Gateway Service
//! that allows multiple switchboard projects to share a single Discord token.

pub mod config;
pub mod protocol;
```

### Current Directory Structure

```
src/gateway/
├── mod.rs      # Module declarations (exists)
├── protocol.rs # Protocol types (exists, story 4.5)
└── config.rs   # TO BE CREATED by this story
```

## Implementation Plan

1. Create `src/gateway/config.rs` with:
   - `GatewayConfig` struct with all required fields
   - `ServerConfig` struct for HTTP/WS server settings
   - `LoggingConfig` struct for logging settings  
   - `ChannelMapping` struct for channel-to-project mapping
   - `GatewayConfigError` using `thiserror`

2. Implement `GatewayConfig::load(path: Option<&str>)`:
   - Default path: `gateway.toml` in current directory
   - Use `std::fs::read_to_string` to read file
   - Use `toml::from_str` to parse
   - Return `ConfigError` on failure

3. Implement environment variable expansion:
   - Support `${DISCORD_TOKEN}` syntax in config
   - Use `crate::config::env::expand_env_vars` or similar pattern
   - Check environment variables when loading

4. Add unit tests:
   - Test basic config parsing
   - Test env var expansion
   - Test error handling for invalid config

5. Verify:
   - Run `cargo build --features gateway`
   - Run `cargo test --lib`
   - Run `cargo clippy -- -D warnings`

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Error handling with thiserror, serde patterns
- `./skills/rust-engineer/SKILL.md` — Rust engineering patterns
- `./skills/rust-engineer/references/error-handling.md` — Result/Option patterns

### Dependencies

- Story 4.1 (gateway module structure) — **COMPLETE**
  - `src/gateway/mod.rs` exists
  - Feature flag `gateway` is in Cargo.toml
  - Module is declared in `src/lib.rs`

## Scope Boundaries

### This Story Includes
- GatewayConfig struct with all fields
- ServerConfig for port settings
- LoggingConfig for logging settings
- ChannelMapping for channel routing
- Config loading from TOML file
- Environment variable expansion
- Unit tests

### This Story Does NOT Include
- HTTP server implementation (belongs to Story 4.3)
- WebSocket server implementation (belongs to Story 4.4)
- Channel registry implementation (belongs to Epic 05)
- CLI integration (belongs to Story 4.8)

### Files in Scope
- `src/gateway/config.rs` — CREATE
- `gateway.toml` (sample config file) — CREATE for testing

### Files NOT in Scope
- `src/gateway/server.rs` — belongs to Story 4.3
- `src/gateway/registry.rs` — belongs to Epic 05
- `src/cli/commands/gateway.rs` — belongs to Story 4.8
