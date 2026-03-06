# Story 4.3: HTTP Server with Health Check Endpoint

> Epic: Epic 04 — Discord Gateway - Basic Gateway with Single Project
> Points: 3
> Sprint: 5
> Type: feature
> Risk: Medium
> Created: 2026-03-03

## User Story

As a system administrator,
I want a health check endpoint on the gateway,
So that I can monitor if the gateway is running.

## Acceptance Criteria

1. Create HTTP server on configured port (default 8080)
   - **Test:** Server starts and binds to port 8080

2. Implement GET `/health` endpoint returning JSON `{"status": "ok"}`
   - **Test:** `curl http://localhost:8080/health` returns 200 with `{"status": "ok"}`

3. Add graceful shutdown handling
   - **Test:** Server stops cleanly on SIGINT (Ctrl+C)

## Technical Context

### Architecture Reference

Per architecture.md §5.3 - gateway::server:
- **Purpose:** HTTP and WebSocket server for gateway
- **Public API:**
  - `GatewayServer::new(config: GatewayConfig) -> Self`
  - `GatewayServer::run().await`
- **Dependencies:** axum, tokio-tungstenite, tower
- **Data flow:** HTTP requests → route handlers

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Async:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`
- **Error Handling:** Use `thiserror` for error types. Never use `unwrap()` in production
- **Logging:** Use `tracing` for logging

### Existing Code Context

Current gateway module structure:
```
src/gateway/
├── mod.rs        # Already exists - module exports
├── config.rs     # Already exists - GatewayConfig
├── protocol.rs   # Already exists - GatewayMessage
└── registry.rs   # Already exists - ChannelRegistry
```

Key existing code patterns to follow:
- Use `#[tokio::main]` for async main
- Use `tracing::info!`, `tracing::debug!` for logging
- Use `thiserror` for error types (see config.rs)

```rust
// Example from src/discord/gateway.rs for async patterns
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... implementation
}
```

## Implementation Plan

1. **Create `src/gateway/server.rs`** — Main HTTP server implementation
   - Add `pub mod server;` to `src/gateway/mod.rs`
   - Implement `GatewayServer` struct with config
   - Implement `run()` method that starts axum server
   - Add `/health` route handler
   - Add graceful shutdown with signal handling

2. **Add dependencies to Cargo.toml** if needed
   - Ensure axum is in dependencies (check existing Cargo.toml)

3. **Write tests** — Unit tests for server startup and health endpoint

4. **Run build + tests** — Verify everything compiles

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices (error handling, async)
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Async patterns with tokio

### Dependencies

- Story 4.2 (gateway config loading) — Complete ✓

## Scope Boundaries

### This Story Includes
- Basic HTTP server with axum
- Health check endpoint
- Graceful shutdown

### This Story Does NOT Include
- WebSocket server (Story 4.4)
- Discord gateway connection (Story 4.7)
- CLI commands (Story 4.8)

### Files in Scope
- `src/gateway/server.rs` — create
- `src/gateway/mod.rs` — modify (add `pub mod server;`)

### Files NOT in Scope
- `src/cli/commands/gateway.rs` — Story 4.8
- `src/discord/gateway.rs` — Already exists, don't modify
