# Story 5.2: Support Channel Mapping in Config

> Epic: Epic 05 — Discord Gateway Phase 2
> Points: 2
> Sprint: 20
> Type: feature
> Risk: Low
> Created: 2026-03-05

## User Story

**As a** user,
**I want** to configure channel-to-project mappings in the config file,
**So that** I can control routing without code changes.

## Acceptance Criteria

1. Config supports `[[channels]]` array in gateway.toml
   - **Test:** Sample config loads correctly. Create a gateway.toml with channel mappings and verify it parses.

2. Each channel mapping has: channel_id, project_name, endpoint
   - **Test:** Fields parse correctly. Verify all three fields are captured in the struct.

3. Validate channel IDs are numeric strings
   - **Test:** Invalid config returns error. Test with non-numeric channel_id.

## Technical Context

### Architecture Reference

From `architecture.md` §5.1 (gateway::config):
- **Purpose:** Load and validate gateway configuration
- **Public API:**
  - `GatewayConfig::load(path: Option<&str>) -> Result<Self, ConfigError>`
  - `GatewayConfig::from_env() -> Result<Self, ConfigError>`
- **Dependencies:** toml, serde
- **Data flow:** TOML file → parse → validated config struct

From `architecture.md` §6 (Data Model):
```rust
struct ChannelMapping {
    channel_id: String,
    project_name: String,
    endpoint: String,  // Project's WebSocket endpoint
}
```

### Project Conventions

From `project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Config:** Use TOML config files with serde. Follow patterns in `src/config/`.
- **Error Handling:** Use `thiserror` for validation errors.
- **Validation:** Validate channel IDs are numeric strings.

### Existing Code Context

**Current GatewayConfig structure** (`src/gateway/config.rs`):
```rust
pub struct GatewayConfig {
    pub discord_token: String,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    // Note: channels field needs to be added
}
```

The GatewayConfig already has discord_token, server, and logging fields. The channels field needs to be added.

**Files to modify:**
- `src/gateway/config.rs` — Add channels field and deserialize

### Files in Scope

- `src/gateway/config.rs` — MODIFY: Add ChannelMapping struct and channels field

### Files NOT in Scope

- `src/gateway/registry.rs` — Not yet (story 5.1 already done)
- `src/gateway/routing.rs` — Not yet (story 5.3)
- `src/cli/commands/gateway.rs` — Not yet

## Implementation Plan

1. **Add ChannelMapping struct** — Define in config.rs with channel_id, project_name, endpoint fields
   - Use serde for deserialization

2. **Add channels field to GatewayConfig** — Add `pub channels: Vec<ChannelMapping>` field

3. **Update load() method** — Parse the `[[channels]]` array from TOML

4. **Add validation** — Ensure channel_id is numeric string

5. **Run build + tests** — Verify everything compiles

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — For serialization patterns
- `./skills/rust-best-practices/SKILL.md` — For Rust idioms

### Dependencies

- Story 4.2: Gateway config loading — COMPLETE

## Scope Boundaries

### This Story Includes
- Channel mapping config structure
- TOML parsing for [[channels]] array
- Validation of channel_id format

### This Story Does NOT Include
- Runtime channel routing (story 5.3)
- Runtime subscribe/unsubscribe (story 5.4)
- CLI integration

## Verification

```bash
# Test config parsing
# Create test config with channel mappings and verify parse

# Build verification
cargo build --features "discord gateway"

# Test verification
cargo test --lib gateway::config

# Lint verification
cargo clippy -- -D warnings
```
