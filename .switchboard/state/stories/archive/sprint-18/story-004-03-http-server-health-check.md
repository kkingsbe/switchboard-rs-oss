# Story 4.3: HTTP server with health check endpoint

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 3
> Sprint: 18
> Type: feature
> Risk: medium
> Created: 2026-03-05T07:11:29Z

## User Story

**As a** system administrator,
**I want** a health check endpoint on the gateway,
**So that** I can monitor if the gateway is running.

## Acceptance Criteria

1. Create HTTP server on configured port (default 9745)
   - Verification: Server starts and binds to port
2. Implement GET `/health` endpoint returning JSON `{"status": "ok"}`
   - Verification: `curl http://localhost:9745/health` returns 200
3. Add graceful shutdown handling
   - Verification: Server stops cleanly on SIGINT

## Technical Context

### Architecture Reference

From `.switchboard/planning/architecture.md` §5.2:

```
### 5.2 gateway::server

- **Purpose:** HTTP and WebSocket server for gateway
- **Public API:**
  - `GatewayServer::new(config: GatewayConfig) -> Self`
  - `GatewayServer::run().await`
- **Dependencies:** axum, tokio-tungstenite, tower
- **Data flow:** 
  - HTTP requests → route handlers
  - WebSocket connections → project session management
```

### Project Conventions

From `.switchboard/planning/project-context.md`:

- **Technology Stack:**
  - **Language:** Rust 2021 edition
  - **Async Runtime:** tokio 1.40 (full features)
  - **HTTP:** axum 0.7
- **Async conventions:** Use tokio for async. Follow patterns in `src/discord/gateway.rs` - `async fn` with `tokio::main`.
- **Error Handling:** Use `thiserror` for error types.
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages.
- **Logging:** Use `tracing` for logging. Never use `println!` or `eprintln!`.

### Existing Code Context

From `src/discord/gateway.rs` - async patterns:
```rust
// Example: async fn with tokio::main
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Server setup
}
```

From `.switchboard/planning/architecture.md` §2 Technology Stack:
```toml
axum = "0.7"
tower = "0.4"
```

## Implementation Plan

1. Add axum and tower dependencies to Cargo.toml
2. Create `src/gateway/server.rs` with basic HTTP server
3. Implement `/health` endpoint
4. Add graceful shutdown with signal handling
5. Verify with `curl http://localhost:9745/health`

### Skills to Read
- `./skills/rust-best-practices/SKILL.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/async.md` (if applicable)
- `./skills/rust-engineer/references/testing.md` (if applicable)

### Dependencies
- Story 4.2: Gateway config loading — must complete first

## Scope Boundaries

### This Story Includes
- Basic HTTP server with axum
- Health check endpoint
- Graceful shutdown
- Port configuration from GatewayConfig

### This Story Does NOT Include
- WebSocket endpoint
- Discord connection
- Project registration protocol

### Files in Scope
- `src/gateway/server.rs` — create
- `Cargo.toml` — modify (add dependencies)

### Files NOT in Scope
- `src/gateway/protocol.rs` — not yet
- `src/gateway/connections.rs` — not yet
- `src/cli/commands/gateway.rs` — not yet
