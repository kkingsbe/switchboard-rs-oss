# Story 4.4: WebSocket server for project connections

> Epic: epic-04-discord-gateway-phase1 — Discord Gateway - Basic Gateway with Single Project
> Points: 3
> Sprint: 18
> Type: feature
> Risk: medium
> Created: 2026-03-05T07:11:29Z

## User Story

**As a** project developer,
**I want** to connect my project to the gateway via WebSocket,
**So that** I can receive Discord messages.

## Acceptance Criteria

1. Create WebSocket endpoint at `/ws`
   - Verification: WebSocket connection accepts upgrade request
2. Handle WebSocket connections and parse incoming messages
   - Verification: Can receive and parse JSON messages
3. Echo received messages back for testing
   - Verification: Simple round-trip test passes

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

From `.switchboard/planning/project-context.md`:

- **Technology Stack:**
  - **WebSocket:** tokio-tungstenite 0.24

### Project Conventions

- **Async conventions:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`.
- **Error Handling:** Use `thiserror` for error types.
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages.
- **Logging:** Use `tracing` for logging. Never use `println!` or `eprintln!`.

### Existing Code Context

From `.switchboard/planning/architecture.md` §2 Technology Stack:
```toml
tokio-tungstenite = "0.24"
futures-util = "0.3"
```

## Implementation Plan

1. Add tokio-tungstenite and futures-util dependencies to Cargo.toml
2. Extend `src/gateway/server.rs` with WebSocket endpoint at `/ws`
3. Implement WebSocket upgrade handler
4. Handle incoming messages and echo back for testing
5. Test with WebSocket client

### Skills to Read
- `./skills/rust-best-practices/SKILL.md`
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/async.md` (if applicable)

### Dependencies
- Story 4.3: HTTP server with health check — must complete first

## Scope Boundaries

### This Story Includes
- WebSocket endpoint at `/ws`
- WebSocket upgrade handling
- Basic message echo for testing

### This Story Does NOT Include
- Project registration protocol
- Discord connection
- Message routing

### Files in Scope
- `src/gateway/server.rs` — modify (add WebSocket)
- `Cargo.toml` — modify (add dependencies)

### Files NOT in Scope
- `src/gateway/protocol.rs` — not yet
- `src/gateway/registry.rs` — not yet
- `src/cli/commands/gateway.rs` — not yet
