# Story 4.4: WebSocket Server for Project Connections

> Epic: Epic 04 — Discord Gateway - Basic Gateway with Single Project
> Points: 3
> Sprint: 6
> Type: feature
> Risk: Medium
> Created: 2026-03-03

## User Story

As a project developer,
I want to connect my project to the gateway via WebSocket,
So that I can receive Discord messages.

## Acceptance Criteria

1. Create WebSocket endpoint at `/ws`
   - **Test:** WebSocket connection accepts upgrade request

2. Handle WebSocket connections and parse incoming messages
   - **Test:** Can receive and parse JSON messages

3. Echo received messages back for testing
   - **Test:** Simple round-trip test passes

## Technical Context

### Architecture Reference

Per architecture.md §5.3 - gateway::server:
- **Purpose:** WebSocket server for project<->gateway communication
- **Public API:**
  - `GatewayServer::new(config: GatewayConfig) -> Self`
  - `GatewayServer::run().await`
- **Dependencies:** tokio-tungstenite, futures-util, axum (WebSocket support)
- **Data flow:** WebSocket messages → parse → process → response

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
├── registry.rs   # Already exists - ChannelRegistry
└── server.rs     # Already exists - HTTP server (needs WebSocket upgrade)
```

Existing HTTP server in server.rs:
- Already handles HTTP requests on port 8080
- Already has /health endpoint
- Needs WebSocket upgrade support added

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

1. **Add WebSocket support to `src/gateway/server.rs`**
   - Import tokio-tungstenite and axum::extract::ws
   - Add WebSocket upgrade route at `/ws`
   - Implement WebSocket handler that:
     - Accepts upgrade request
     - Creates bidirectional channel for messages
     - Parses incoming JSON messages using GatewayMessage
     - Echoes messages back for testing

2. **Add dependencies to Cargo.toml** if needed
   - Ensure tokio-tungstenite is in dependencies (check existing Cargo.toml)
   - Ensure futures-util is available

3. **Write tests** — Unit tests for WebSocket handler

4. **Run build + tests** — Verify everything compiles

### Skills to Read

- [`skills/rust-best-practices/SKILL.md`](skills/rust-best-practices/SKILL.md) — Rust best practices (error handling, async)
- [`skills/rust-engineer/SKILL.md`](skills/rust-engineer/SKILL.md) — Async patterns with tokio

### Dependencies

- Story 4.3 (HTTP server with health check) — Complete ✓

## Scope Boundaries

### This Story Includes
- WebSocket endpoint at /ws
- Message parsing using GatewayMessage
- Echo functionality for testing

### This Story Does NOT Include
- Registration protocol (Story 4.6)
- Discord gateway connection (Story 4.7)
- Message routing by channel (Story 5.3)

### Files in Scope
- `src/gateway/server.rs` — modify (add WebSocket support)

### Files NOT in Scope
- `src/gateway/protocol.rs` — Already exists
- `src/gateway/registry.rs` — Already exists
- `src/cli/commands/gateway.rs` — Story 4.8
