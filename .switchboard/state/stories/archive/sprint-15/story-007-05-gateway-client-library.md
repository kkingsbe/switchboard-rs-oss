# Story 007-05: Gateway Client Library

> Epic: Epic 07 — CLI Integration
> Points: 3
> Sprint: 15
> Type: feature
> Risk: medium
> Created: 2026-03-04

## User Story

**As a** project developer,  
**I want** a client library to connect to the gateway,  
**So that** I don't have to implement the WebSocket protocol manually.

## Acceptance Criteria

1. Create GatewayClient struct
   - **Test:** Can be instantiated

2. Implement connect() method
   - **Test:** Establishes WebSocket connection

3. Implement recv() to receive messages
   - **Test:** Receives Discord messages

4. Implement heartbeat automatically
   - **Test:** Heartbeat sent in background

## Technical Context

### Architecture Reference
- Create: `src/gateway/client.rs`
- Public API for external projects
- Uses tokio-tungstenite for WebSocket

### Project Conventions
- Build: `cargo build --features "discord gateway"`
- Test: `cargo test --lib`
- Use thiserror for errors
- Use tokio for async
- Document public API

## Implementation Plan

1. Create `src/gateway/client.rs` with GatewayClient struct
2. Implement connect() method for WebSocket connection
3. Implement recv() for receiving messages
4. Add background heartbeat task
5. Write unit tests
6. Run build + tests

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — async Rust patterns
- `./skills/rust-engineer/references/async.md` — async patterns
- `./skills/rust-best-practices/SKILL.md` — best practices

### Dependencies
- story-004-05 (Message protocol types) — COMPLETE
- story-004-06 (Basic registration protocol) — COMPLETE
- story-006-02 (Heartbeat protocol) — COMPLETE

## Scope Boundaries

### This Story Includes
- GatewayClient struct
- connect(), recv() methods
- Automatic heartbeat

### This Story Does NOT Include
- CLI commands
- PID file management
- Logging configuration
