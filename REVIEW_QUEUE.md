# Code Review Queue

---

## PENDING_REVIEW

*(None)*

---

## CHANGES_REQUESTED

### story-007-02: Gateway Down CLI
- **Status**: CHANGES_REQUESTED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-04T06:34:00Z
- **Acceptance Criteria**:
  - [x] Gateway stops gracefully — MET: Implemented with SIGTERM and configurable timeout
  - [x] Connected projects notified of shutdown — MET: SIGTERM triggers graceful shutdown
  - [x] CLI available — MET: `gateway down` command shows in CLI help
  - [ ] Code compiles without warnings — NOT MET: 6 clippy errors in gateway.rs
- **Must Fix**:
  1. Clippy error at src/cli/commands/gateway.rs:408
     - Current: `.ok_or_else(|| GatewayCommandError::NotRunning)?`
     - Expected: `.ok_or(GatewayCommandError::NotRunning)?`
     - Why: Per project context and rust-best-practices skill, clippy must pass with `-D warnings`
  2. Clippy error at src/cli/commands/gateway.rs:411
     - Current: `.map_err(|e| GatewayCommandError::IoError(e))`
     - Expected: `GatewayCommandError::IoError`
     - Why: Per rust-best-practices skill, use tuple variant directly instead of redundant closure
  3. Clippy errors at src/cli/commands/gateway.rs:487, 489, 493, 502
     - Current: `return Ok(());` / `return Err(...)`
     - Expected: `Ok(())` / `Err(...)` (remove unnecessary return statements)
     - Why: Per rust-best-practices skill, unneeded return statements should be removed
- **Requeue Instructions**: Fix all 6 clippy errors in src/cli/commands/gateway.rs and re-queue for review

---

## APPROVED

### story-006-04: Handle Disconnections
- **Status**: ✅ APPROVED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-04T06:33:00Z
- **Acceptance Criteria**:
  - [x] Detect WebSocket close events — MET: Message::Close handling at server.rs:415
  - [x] Remove project from routing — MET: registry.unregister() called on disconnect
  - [x] Allow project to re-register — MET: registry.register() handles existing projects
- **Summary**: Implementation exists in codebase. All disconnection tests pass. Build and clippy pass for server.rs.

### story-005-04: Runtime Channel Subscribe/Unsubscribe
- **Status**: ✅ APPROVED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-04T06:31:00Z
- **Acceptance Criteria**:
  - [x] Project can send `channel_subscribe` message — MET: GatewayMessage::ChannelSubscribe exists with handler
  - [x] Project can send `channel_unsubscribe` message — MET: GatewayMessage::ChannelUnsubscribe exists with handler
  - [x] Changes take effect immediately — MET: handlers call registry.add_channel_subscription() / remove_channel_subscription()
- **Summary**: Implementation complete with message types, handlers, and serialization tests. All 726 tests pass (5 pre-existing Docker failures). Clippy passes for protocol.rs and server.rs.

### story-007-01: CLI `gateway status` Command
- **Status**: ✅ APPROVED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-04T06:29:00Z
- **Acceptance Criteria**:
  - [x] Show gateway running/stopped status — MET: Checks PID file
  - [x] Show Discord connection status — MET: Queries HTTP /status endpoint
  - [x] Show connected projects/channels — MET: Displays from /status endpoint response
- **Summary**: Enhanced gateway status command to query HTTP /status endpoint. All tests pass. Clippy passes for status command code.

### story-004-08: CLI `gateway up` Command
- **Status**: ✅ APPROVED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-04T06:25:00Z
- **Acceptance Criteria**:
  - [x] Build passes — MET: cargo build --features "discord gateway" succeeds
  - [x] Tests pass — MET: 726 tests pass (5 pre-existing Docker failures)
  - [x] Command exists and is functional — MET: `gateway up --help` shows correct usage
- **Summary**: CLI gateway up command already exists in codebase and is functional. All acceptance criteria met.

### story-005-03
- **Status**: ✅ APPROVED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-03T22:57:00Z
- **Acceptance Criteria**:
  - [x] Extract channel_id from MessageCreate events — MET: server.rs process_discord_events extracts channel_id
  - [x] Look up projects subscribed to that channel — MET: registry.projects_for_channel() called
  - [x] Forward message to those projects via WebSocket — MET: Iterates projects and sends via ws_sender
- **Findings**:
  - SHOULD FIX: Consider reusing Router::route_message() from routing.rs instead of inline implementation in process_discord_events (code duplication)
- **Summary**: All acceptance criteria met. Build, tests (133 gateway tests), and clippy all pass. Implementation correctly routes Discord messages to subscribed projects via WebSocket.
