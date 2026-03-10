# Story 4.3: Create HTTP server with health check endpoint

> Epic: epic-04 — Discord Gateway Phase 1: Basic Gateway
> Points: 3
> Sprint: 2
> Type: feature
> Risk: medium
> Created: 2026-03-02

## User Story

**As a** system administrator,
**I want** a health check endpoint on the gateway,
**So that** I can monitor if the gateway is running.

## Acceptance Criteria

1. Create HTTP server on configured port (default 9745)
   - **Test:** Server starts and binds to port without errors

2. Implement GET `/health` endpoint returning JSON `{"status": "ok"}`
   - **Test:** `curl http://localhost:9745/health` returns 200 with correct JSON

3. Add graceful shutdown handling
   - **Test:** Server stops cleanly on SIGINT without hanging connections

## Technical Context

### Architecture Reference

From architecture.md:
- HTTP server using axum framework
- Default port 9745
- Health check endpoint at /health
- Graceful shutdown with signal handling

### Project Conventions

From project-context.md:
- **Build:** `cargo build --features "discord gateway"`
- **Test:** `cargo test --lib`
- **Lint:** `cargo clippy -- -D warnings`
- **Error Handling:** Use thiserror - never anyhow in library code
- **No unwrap()** in production - use `?` or `.expect()`
- **Async:** tokio with patterns from `src/discord/gateway.rs`
- **Logging:** Use tracing - never println!
- **Tests:** Inline in module files

### Existing Code Context

Pattern to follow from `src/discord/gateway.rs`:
```rust
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

// Async server pattern
pub async fn run_server(config: GatewayConfig) -> Result<(), GatewayError> {
    // Server implementation
}
```

## Implementation Plan

1. Add dependencies to Cargo.toml: axum, tower, hyper
2. Create `src/gateway/server.rs`
3. Implement GatewayServer struct with:
   - bind address from config
   - axum router with /health endpoint
   - graceful shutdown with signal handling
4. Implement health check handler returning JSON
5. Integrate with GatewayConfig for port binding
6. Write integration tests for server startup and health endpoint
7. Run tests: `cargo test --lib`
8. Run lint: `cargo clippy -- -D warnings`
9. Manual test: Start server, curl health endpoint, send SIGINT

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Async patterns and error handling
- `./skills/rust-engineer/SKILL.md` — Tokio async runtime
- `./skills/rust-engineer/references/async.md` — Axum and tower patterns

### Dependencies

- **Story 4.2**: Must complete first (GatewayConfig needed for port configuration)

## Scope Boundaries

### This Story Includes
- Creating HTTP server with axum
- Health check endpoint at /health
- Graceful shutdown on SIGINT
- Integration with GatewayConfig

### This Story Does NOT Include
- WebSocket server (story 4.4)
- Discord Gateway connection (story 4.7)
- CLI commands (story 4.8)
- Authentication/authorization

### Files in Scope
- `src/gateway/server.rs` — create
- `Cargo.toml` — modify (add axum, tower, hyper)

### Files NOT in Scope
- `src/gateway/protocol.rs` — story 4.5
- WebSocket server — story 4.4
- CLI integration
