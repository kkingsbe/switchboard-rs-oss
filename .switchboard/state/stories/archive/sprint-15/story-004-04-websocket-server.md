# Story 004-04: WebSocket Server for Project Connections

> Epic: Epic 04 — Discord Gateway - Basic Gateway
> Points: 3
> Sprint: 15
> Type: feature
> Risk: medium
> Created: 2026-03-04

## User Story

**As a** project developer,  
**I want** to connect my project to the gateway via WebSocket,  
**So that** I can receive Discord messages.

## Acceptance Criteria

1. Create WebSocket endpoint at `/ws`
   - **Test:** WebSocket connection accepts upgrade request

2. Handle WebSocket connections and parse incoming messages
   - **Test:** Can receive and parse JSON messages

3. Echo received messages back for testing
   - **Test:** Simple round-trip test passes

## Technical Context

### Architecture Reference
- Gateway module: `src/gateway/`
- WebSocket handling via tokio-tungstenite
- Dependencies: tokio-tungstenite, futures-util

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Use thiserror for errors, never anyhow
- Use tokio for async
- Use tracing for logging, never println!

## Implementation Plan

1. Add tokio-tungstenite to Cargo.toml dependencies
2. Create WebSocket upgrade handler in `src/gateway/server.rs`
3. Implement message parsing for incoming JSON
4. Add echo functionality for testing
5. Write unit tests for WebSocket handler
6. Run build + tests

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — async Rust patterns
- `./skills/rust-engineer/references/async.md` — async/await patterns

### Dependencies
- story-004-03 (HTTP server with health check) — COMPLETE

## Scope Boundaries

### This Story Includes
- WebSocket endpoint at /ws
- Basic message parsing
- Echo functionality

### This Story Does NOT Include
- Authentication/authorization
- Message routing
- Reconnection logic
- Rate limiting
