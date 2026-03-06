# Story 5.5: Add Configuration Validation

> Epic: Epic 05 — Discord Gateway - Phase 2: Channel Routing
> Points: 1
> Sprint: 7
> Type: feature
> Risk: low
> Created: 2026-03-03
> Status: not-started

## User Story

As a user,
I want to get clear error messages when my config is invalid,
So that I can fix configuration issues quickly.

## Acceptance Criteria

1. Validate discord_token is not empty
   - **Test:** Error if token missing
   - Verification: Load config with empty token returns error

2. Validate http_port and ws_port are valid (1024-65535)
   - **Test:** Error if port out of range
   - Verification: Load config with port=80 returns error

3. Validate channel mappings have required fields
   - **Test:** Error if channel mapping incomplete
   - Verification: Load config with missing channel_id returns error

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md`:

- **§5.1 gateway::config:** Load and validate gateway configuration
- **§7 Error Handling Strategy:** Use `thiserror` for error types

### Project Conventions

From `.switchboard/planning/project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Error Handling:** Use `thiserror` for error types. Never use `unwrap()` in production

### Existing Code Context

**GatewayConfig in `src/gateway/config.rs`:**
```rust
// Lines 93-110 - GatewayConfig already exists
pub struct GatewayConfig {
    pub discord_token: String,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub channels: Vec<ChannelMapping>,
}

pub struct ChannelMapping {
    pub channel_id: String,
    pub project_name: String,
    pub endpoint: String,
}
```

**ServerConfig:**
```rust
// Lines 33-57
pub struct ServerConfig {
    pub host: String,
    pub http_port: u32,
    pub ws_port: u32,
}
```

**Existing error types:**
```rust
// Lines 14-31 - GatewayConfigError already has validation support
pub enum GatewayConfigError {
    // ... existing variants ...
    #[error("Configuration validation error: {0}")]
    ValidationError(String),
}
```

### Files in src/gateway/
```
src/gateway/
├── mod.rs        # Module exports (exists)
├── config.rs     # Config loading - ADD VALIDATION HERE
├── protocol.rs   # Message protocol types (exists)
├── registry.rs   # Channel registry (exists)
└── server.rs     # HTTP/WS server (exists)
```

## Implementation Plan

1. **Add validation to GatewayConfig**
   - Create private `validate()` method on GatewayConfig
   - Call validate() in `GatewayConfig::load()`
   - Return ValidationError with clear messages

2. **Validate discord_token**
   - Check not empty after env var expansion
   - Return helpful error if missing

3. **Validate ports**
   - Check http_port in range 1024-65535
   - Check ws_port in range 1024-65535
   - Check they don't conflict (optional)

4. **Validate channel mappings**
   - Each ChannelMapping must have:
     - non-empty channel_id (numeric string)
     - non-empty project_name
     - valid endpoint URL (optional, but recommended)

5. **Write unit tests**
   - Test empty token returns error
   - Test invalid port returns error
   - Test invalid channel mapping returns error
   - Test valid config passes validation

6. **Run build + tests**
   - `cargo build --features "discord gateway"`
   - `cargo test --lib`

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices
- [`skills/rust-engineer/references/error-handling.md`](skills/rust-engineer/references/error-handling.md) — Error handling patterns

### Dependencies

- Story 5.2 (Channel mapping in config) — Complete ✓

## Scope Boundaries

### This Story Includes
- Validation of discord_token
- Validation of http_port and ws_port ranges
- Validation of channel mapping fields
- Clear error messages for each validation failure

### This Story Does NOT Include
- Runtime validation of channel existence in Discord
- Network connectivity checks
- Config file watcher for hot-reload
- Validation of project endpoints (Story 5.2 covers this partially)

### Files in Scope
- `src/gateway/config.rs` — modify (add validation)

### Files NOT in Scope
- `src/gateway/server.rs` — Don't modify server logic
- `src/gateway/registry.rs` — Different module
- `src/discord/gateway.rs` — Different module
