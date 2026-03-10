# Story: story-004-02 - Implement Gateway Configuration Loading

## Metadata

- **Story ID**: story-004-02
- **Title**: Implement Gateway Configuration Loading
- **Epic**: Epic 004 - Gateway Infrastructure
- **Points**: 2
- **Type**: feature
- **Risk Level**: Low
- **Status**: Implemented

---

## User Story

As an operator, I want the gateway to load its configuration from a TOML file so that I can customize gateway behavior without modifying code.

---

## Acceptance Criteria

1. Gateway can load configuration from `gateway.toml` file
2. Configuration supports Discord bot token with environment variable expansion
3. Configuration supports server settings (host, http_port, ws_port)
4. Configuration supports logging settings (level, file)
5. Configuration supports channel mappings (channel_id, project_name, endpoint)
6. Configuration validation ensures required fields are present
7. Defaults are provided for optional fields

**Test Methods**:
- `GatewayConfig::load(Some("gateway.toml"))` successfully loads valid config
- Invalid config files return appropriate errors
- Environment variable expansion works for `${VAR}` and `${VAR:-default}` syntax

---

## Technical Context

### Architecture References

The configuration module follows the pattern established in `src/config/` for environment variable handling.

### Existing Code

- Configuration module: `src/gateway/config.rs`
- Config env resolution: `src/config/env.rs`
- Example config: `gateway.toml`

---

## Implementation Plan

1. Create `GatewayConfig` struct with fields:
   - `discord_token: String`
   - `server: ServerConfig`
   - `logging: LoggingConfig`
   - `channels: Vec<ChannelMapping>`
2. Create `ServerConfig` struct with host, http_port, ws_port
3. Create `LoggingConfig` struct with level, file
4. Create `ChannelMapping` struct with channel_id, project_name, endpoint
5. Implement `GatewayConfig::load(path)` method:
   - Read TOML file
   - Parse into struct
   - Resolve environment variables
   - Validate configuration
6. Implement `GatewayConfig::from_env()` for environment-based config
7. Add comprehensive tests
8. Add defaults using serde attributes

---

## Skills to Read

- [Rust Engineer](../../skills/rust-engineer/SKILL.md)
- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)
- [Traits Reference](../../skills/rust-engineer/references/traits.md)

---

## Dependencies

- `toml` crate for TOML parsing
- `serde` / `serdeDeserialize` for serialization
- `src/config/env.rs` for environment variable resolution

---

## Scope Boundaries

### In Scope
- Loading configuration from TOML files
- Environment variable expansion
- Configuration validation
- Default values for optional fields

### Out of Scope
- Writing configuration (read-only)
- Hot reloading
- Multiple config file formats

---

## Files in Scope

| File | Description |
|------|-------------|
| `src/gateway/config.rs` | Configuration loading implementation |
| `src/config/env.rs` | Environment variable resolution (dependency) |
| `gateway.toml` | Example configuration file |
| `src/gateway/mod.rs` | Module export |

---

## Acceptance Test Examples

```rust
// Load valid config
let config = GatewayConfig::load(Some("gateway.toml")).expect("Failed to load");
assert!(!config.discord_token.is_empty());

// Invalid config returns error
let result = GatewayConfig::load(Some("invalid.toml"));
assert!(result.is_err());

// Environment variable expansion
// Config with: discord_token = "${TEST_TOKEN:-default}"
// Should resolve to value of TEST_TOKEN or "default"
```
