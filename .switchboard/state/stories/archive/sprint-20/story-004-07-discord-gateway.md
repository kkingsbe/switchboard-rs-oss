# Story 4.7: Wire up Discord Gateway Connection

> Epic: Epic 04 — Discord Gateway Phase 1
> Points: 5
> Sprint: 20
> Type: feature
> Risk: High
> Created: 2026-03-05

## User Story

**As a** user,
**I want** the gateway to connect to Discord,
**So that** it can receive messages from my channels.

## Acceptance Criteria

1. Gateway connects to Discord using twilight-gateway
   - **Test:** Connection established, Ready event received. Check logs for "Connected to Discord" message.

2. Gateway listens for MessageCreate events
   - **Test:** Messages logged/forwarded. Send a message in a configured Discord channel and verify it's received.

3. Handle reconnection on disconnect
   - **Test:** Auto-reconnect after Discord disconnect. Simulate disconnect and verify reconnection.

## Technical Context

### Architecture Reference

From `architecture.md` §5.2 (gateway::server):
- **Purpose:** HTTP and WebSocket server for gateway
- **Public API:**
  - `GatewayServer::new(config: GatewayConfig) -> Self`
  - `GatewayServer::run().await`
- **Dependencies:** axum, tokio-tungstenite, tower, twilight-gateway

From `architecture.md` §9 (Non-Functional Requirements):
- Single Discord connection: Gateway maintains one twilight-gateway connection
- Channel routing: ChannelRegistry maps channel_id → projects

### Project Conventions

From `project-context.md`:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Error Handling:** Use `thiserror` for error types. Never use `anyhow` in library code.
- **No unwrap() in production:** Use `?` operator or `.expect()` with descriptive messages.
- **Async conventions:** Use tokio for async. Follow patterns in `src/discord/gateway.rs`.
- **Logging:** Use `tracing` for logging. Never use `println!` or `eprintln!`.

### Existing Code Context

**Current gateway module structure** (`src/gateway/`):
```
src/gateway/
├── mod.rs         (module declarations)
├── config.rs      (GatewayConfig - already implemented)
├── server.rs      (HTTP/WS server - exists, needs Discord integration)
├── protocol.rs    (message types - exists)
├── registry.rs    (ChannelRegistry - exists)
├── connections.rs (connection management - exists)
├── routing.rs     (message routing - exists)
├── client.rs      (client library - exists)
├── ratelimit.rs   (rate limiting - exists)
├── reconnection.rs (reconnection logic - exists)
└── pid.rs         (PID file - exists)
```

**Key existing code - server.rs run() function:**
```rust
pub async fn run(&self) -> Result<(), GatewayServerError> {
    serve(self.server_config.clone()).await
}
```
The current run() function only serves HTTP/WS. It needs to ALSO start the Discord gateway connection.

**Reference implementation:** `src/discord/gateway.rs` - contains the existing DiscordGateway implementation that connects to Discord using twilight-gateway.

### Files in Scope

- `src/gateway/server.rs` — MODIFY: Add Discord gateway connection
- `src/gateway/mod.rs` — MODIFY: Export new types if needed
- `src/gateway/routing.rs` — MODIFY: Wire up message routing from Discord events

### Files NOT in Scope

- `src/discord/gateway.rs` — Do NOT modify (keep existing Discord bot functionality separate)
- `src/cli/commands/gateway.rs` — Not yet (story 4.8)
- Client library (`src/gateway/client.rs`) — Already implemented in story 007-05

## Implementation Plan

1. **Extend GatewayServer** — Add Discord gateway connection fields to GatewayServer struct
   - Add `discord_gateway: Option<DiscordGateway>` or create new gateway task
   - Add channel for forwarding Discord events to project routing

2. **Update run() method** — Modify to run both HTTP server AND Discord gateway concurrently
   - Use `tokio::select!` or spawn tasks to run both
   - Handle graceful shutdown for both

3. **Wire up message routing** — Connect Discord event listener to project message routing
   - When MessageCreate received → route to appropriate projects via registry

4. **Add reconnection handling** — Use existing twilight reconnection or implement in gateway
   - Follow patterns from `src/discord/gateway.rs`

5. **Run build + tests** — Verify everything compiles and tests pass

### Skills to Read

- `./skills/rust-engineer/SKILL.md` — For async patterns and error handling
- `./skills/rust-best-practices/SKILL.md` — For Rust idioms and testing

### Dependencies

- Story 4.2: Gateway config loading — COMPLETE
- Story 4.6: Registration protocol — COMPLETE

## Scope Boundaries

### This Story Includes
- Discord WebSocket connection using twilight-gateway
- MessageCreate event handling
- Reconnection on disconnect
- Integration with existing channel routing

### This Story Does NOT Include
- CLI commands (story 4.8)
- Gateway clustering/high availability
- Persistent message storage

## Verification

```bash
# Build verification
cargo build --features "discord gateway"

# Test verification  
cargo test --lib

# Lint verification
cargo clippy -- -D warnings
```
