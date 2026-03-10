# Story 4.1: Create gateway module structure

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 1
> Sprint: 18
> Type: infrastructure
> Risk: low
> Created: 2026-03-05T07:11:29Z

## User Story

**As a** developer,
**I want** a basic gateway module structure,
**So that** I can organize the gateway code.

## Acceptance Criteria

1. Create `src/gateway/mod.rs` with module declarations
   - Verification: File exists and compiles
2. Add `pub mod gateway` to `src/lib.rs`
   - Verification: `cargo build` succeeds
3. Add feature flag `gateway` to Cargo.toml
   - Verification: `cargo build --features gateway` compiles

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md` §3:

```
src/
├── gateway/                    ← NEW MODULE
│   ├── mod.rs                  # Module exports
│   ├── config.rs               # Gateway configuration
│   ├── server.rs               # HTTP/WS server
│   ├── protocol.rs             # Message protocol types
│   ├── routing.rs              # Message routing logic
│   ├── registry.rs             # Channel/project registry
│   ├── connections.rs          # Connection management
│   ├── heartbeat.rs            # Heartbeat protocol
│   ├── ratelimit.rs           # Discord rate limiting
│   └── client.rs               # Client library for projects
```

### Project Conventions

From `.switchboard/planning/project-context.md`:

- **Build:** `cargo build --features "discord gateway"` (include gateway feature flag)
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Format:** `cargo fmt`
- **Error Handling:** Use `thiserror` for error types. Never use `anyhow` in library code.
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages.
- **Module organization:** New gateway code goes in `src/gateway/`.

### Existing Code Context

From `src/lib.rs` - module exports pattern:
```rust
pub mod discord;
pub mod gateway;  // Add this
```

From `Cargo.toml` - feature flag pattern:
```toml
[features]
default = ["discord"]
discord = []
gateway = []  // Add this
```

## Implementation Plan

1. Create `src/gateway/mod.rs` with basic module declarations
2. Add `pub mod gateway` to `src/lib.rs`
3. Add `gateway` feature flag to `Cargo.toml`
4. Verify build with `cargo build --features "discord gateway"`

### Skills to Read
- `./skills/rust-best-practices/SKILL.md`
- `./skills/rust-engineer/SKILL.md`

### Dependencies
- None - this is the foundation story

## Scope Boundaries

### This Story Includes
- Basic module structure in `src/gateway/mod.rs`
- Feature flag in Cargo.toml
- Module export in src/lib.rs

### This Story Does NOT Include
- Any implementation code (config, server, protocol, etc.)
- CLI commands

### Files in Scope
- `src/gateway/mod.rs` — create
- `src/lib.rs` — modify
- `Cargo.toml` — modify

### Files NOT in Scope
- `src/gateway/config.rs` — not yet
- `src/gateway/server.rs` — not yet
- `src/gateway/protocol.rs` — not yet
