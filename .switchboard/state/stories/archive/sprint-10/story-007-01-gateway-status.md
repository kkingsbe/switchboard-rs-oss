# Story 007-01: CLI `gateway status` Command

> Epic: epic-07 — Discord Gateway Phase 4 (CLI Integration & Monitoring)
> Points: 2
> Sprint: 10
> Type: feature
> Risk: low
> Created: 2026-03-03
> Status: in-progress
> Assigned To: dev-1

## User Story

As a user, I want to see the gateway status, So that I know if it's running and what's connected.

## Acceptance Criteria

1. Show gateway status (running/stopped)
   - **Test:** Run `switchboard gateway status` and verify status displayed
   - **Test:** When gateway not running, show "stopped" status

2. Show Discord connection status
   - **Test:** When gateway connected to Discord, show "connected"
   - **Test:** When gateway disconnected, show "disconnected"

3. Show connected projects and their channels
   - **Test:** With projects connected, list project names and subscribed channels
   - **Test:** With no projects, show "no connected projects"

## Technical Context

### Architecture Reference

From architecture.md:
- Gateway maintains HTTP server for monitoring
- Gateway tracks connected projects in ConnectionManager
- Discord connection status tracked in GatewayServer
- HTTP endpoint: GET `/status` returning JSON

### Project Conventions

From project-context.md:
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Use `tracing` for logging
- Never use `unwrap()` in production

### Existing Code Context

```
src/gateway/
├── mod.rs
├── config.rs       (EXISTS)
├── protocol.rs     (EXISTS)
├── registry.rs    (EXISTS - tracks channel subscriptions)
├── server.rs      (EXISTS - main server)
├── connections.rs (EXISTS - ConnectionManager)
├── routing.rs     (EXISTS)
└── pid.rs         (EXISTS - PID file management)
```

**Existing gateway server:**
- `src/gateway/server.rs` has `GatewayServer` struct
- Tracks Discord connection state
- Has `connections` field with project info

**Existing connections.rs:**
- `ConnectionManager` tracks active project connections
- Has method to list connected projects

**Existing cli/commands/gateway.rs:**
- Partial implementation exists
- Has `up` subcommand

## Implementation Plan

1. **Examine** `src/gateway/server.rs` - Understand current state tracking
2. **Examine** `src/gateway/connections.rs` - Understand connection tracking
3. **Add** HTTP status endpoint in `src/gateway/server.rs`:
   - GET `/status` returning JSON with gateway state
4. **Implement** `status` subcommand in `src/cli/commands/gateway.rs`:
   - Call HTTP endpoint or read shared state
   - Display formatted status output
5. **Handle** gateway not running case (check PID file or try to connect)
6. **Write tests** for status command
7. **Run** `cargo build --features "discord gateway"` and verify
8. **Run** `cargo test --lib` and verify all tests pass

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-best-practices/SKILL.md` — Testing patterns

### Dependencies
- story-006-01 (Project connection management) — complete (connections.rs exists)

## Scope Boundaries

### This Story Includes
- `switchboard gateway status` command
- HTTP GET `/status` endpoint
- Display gateway running status
- Display Discord connection status
- Display connected projects and channels

### This Story Does NOT Include
- Real-time updates (polling or WebSocket)
- Historical metrics
- Project disconnect button
- Gateway restart functionality

### Files in Scope
- `src/cli/commands/gateway.rs` — modify (add status subcommand)
- `src/gateway/server.rs` — modify (add /status endpoint)
- `src/gateway/connections.rs` — use (read connection state)

### Files NOT in Scope
- `src/gateway/pid.rs` — don't modify (use existing)
- `src/discord/gateway.rs` — don't modify
