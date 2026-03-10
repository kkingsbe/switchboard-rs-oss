# Story 4.2: Gateway config loading

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 2
> Sprint: 18
> Type: feature
> Risk: low
> Created: 2026-03-05T07:11:29Z

## User Story

**As a** developer,
**I want** to load gateway configuration from a TOML file,
**So that** the gateway can be configured without hardcoding.

## Acceptance Criteria

1. Create `GatewayConfig` struct with fields: discord_token, server, logging, channels
   - Verification: Unit tests pass
2. Implement `GatewayConfig::load(path: Option<&str>)` to load from `gateway.toml`
   - Verification: Can load sample config file
3. Support environment variable expansion for discord_token (e.g., `${DISCORD_TOKEN}`)
   - Verification: Token loaded from env var when specified

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md` §5.1:

```
### 5.1 gateway::config

- **Purpose:** Load and validate gateway configuration
- **Public API:**
  - `GatewayConfig::load(path: Option<&str>) -> Result<Self, ConfigError>`
  - `GatewayConfig::from_env() -> Result<Self, ConfigError>`
- **Dependencies:** toml, serde
- **Data flow:** TOML file → parse → validated config struct
```

From §6 Data Model:

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

- **Configuration:** Use TOML config files with serde. Follow patterns in `src/config/`.
- **Error Handling:** Use `thiserror` for error types. Never use `anyhow` in library code.
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages.
- **Logging:** Use `tracing` for logging. Never use `println!` or `eprintln!`.

### Existing Code Context

From `src/config/mod.rs` - config loading patterns:
```rust
// Example pattern to follow
impl Config {
    pub fn load(path: Option<&str>) -> Result<Self, ConfigError> {
        let path = path.unwrap_or("switchboard.toml");
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        // Environment variable expansion
        Ok(config)
    }
}
```

From `src/config/env.rs` - environment variable expansion:
```rust
// Example: expand ${VAR_NAME} patterns
pub fn expand_env_vars(s: &str) -> String { ... }
```

## Implementation Plan

1. Create `src/gateway/config.rs` with `GatewayConfig` struct
2. Implement `GatewayConfig::load()` method
3. Add environment variable expansion support
4. Create sample `gateway.toml` file
5. Write unit tests for config loading

### Skills to Read
- `./skills/rust-best-practices/SKILL.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/traits.md` (if applicable)
- `./skills/rust-engineer/references/testing.md` (if applicable)

### Dependencies
- Story 4.1: Create gateway module structure — must complete first

## Scope Boundaries

### This Story Includes
- GatewayConfig struct with all required fields
- Config loading from TOML file
- Environment variable expansion
- Sample gateway.toml

### This Story Does NOT Include
- HTTP server implementation
- WebSocket implementation
- Discord connection

### Files in Scope
- `src/gateway/config.rs` — create
- `gateway.toml` — create (sample config)

### Files NOT in Scope
- `src/gateway/server.rs` — not yet
- `src/gateway/protocol.rs` — not yet
- `src/cli/commands/gateway.rs` — not yet
