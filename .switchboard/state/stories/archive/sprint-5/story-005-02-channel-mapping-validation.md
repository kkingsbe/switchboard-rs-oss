# Story 5.2: Configuration Validation

> Epic: Epic 05 — Discord Gateway - Channel Routing with Config File
> Points: 2
> Sprint: 5
> Type: feature
> Risk: Low
> Created: 2026-03-03

## User Story

As a user,
I want to get clear error messages when my config is invalid,
So that I can fix configuration issues quickly.

## Acceptance Criteria

1. Validate discord_token is not empty
   - **Test:** Config with empty token returns error

2. Validate http_port and ws_port are valid (1024-65535)
   - **Test:** Config with port=80 returns error
   - **Test:** Config with port=70000 returns error

3. Validate channel mappings have required fields
   - **Test:** Config with missing channel_id returns error
   - **Test:** Config with missing project_name returns error

## Technical Context

### Architecture Reference

Per architecture.md §5.2 - gateway::config:
- Already implemented: `GatewayConfig::load(path: Option<&str>)`
- Already implemented: Channel mapping in config

This story adds validation to the existing config loading.

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Error Handling:** Use `thiserror` for error types
- **Testing:** Place unit tests in the same file as the code

### Existing Code Context

Current `src/gateway/config.rs` already has:
- `GatewayConfigError` enum with variants
- `GatewayConfig::load()` method
- Channel mapping struct with fields

Existing error variants:
```rust
#[derive(Debug, Error)]
pub enum GatewayConfigError {
    #[error("Failed to read configuration file: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse configuration file: {0}")]
    ParseError(#[from] toml::de::Error),
    #[error("Missing required environment variable: {0}")]
    EnvVarError(String),
}
```

Existing config structure:
```rust
pub struct ChannelMapping {
    pub channel_id: String,
    pub project_name: String,
    pub endpoint: String,
}
```

## Implementation Plan

1. **Add validation error variant to `GatewayConfigError`**
   - Add `ValidationError(String)` variant

2. **Implement validation in `GatewayConfig::load()`**
   - Validate discord_token is not empty
   - Validate http_port is in valid range (1024-65535)
   - Validate ws_port is in valid range
   - Validate each channel mapping has required fields

3. **Add unit tests**
   - Test empty token returns error
   - Test invalid port returns error
   - Test missing channel fields return error

4. **Run build + tests**

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Error handling with thiserror
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Validation patterns

### Dependencies

- Story 4.2 (gateway config loading) — Complete ✓

## Scope Boundaries

### This Story Includes
- Config validation for required fields
- Port range validation
- Error messages

### This Story Does NOT Include
- Runtime channel subscription (Story 5.4)
- Message routing (Story 5.3)

### Files in Scope
- `src/gateway/config.rs` — modify (add validation)

### Files NOT in Scope
- `src/gateway/server.rs` — Story 4.3
- `src/gateway/registry.rs` — Already exists
