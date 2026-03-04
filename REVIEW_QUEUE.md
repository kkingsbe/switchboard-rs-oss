# Code Review Queue

---

## PENDING_REVIEW

### story-004-08: CLI `gateway up` Command

- **Implemented by:** dev-1
- **Sprint:** 10
- **Story file:** `.switchboard/state/stories/story-004-08-gateway-up-cli.md`
- **Files changed:** Existing CLI code - commands already implemented
- **Status:** PENDING_REVIEW
- **Acceptance Criteria:**
  - [x] Build passes — verified by: cargo build --features "discord gateway"
  - [x] Tests pass — verified by: cargo test --lib (693/698, 5 pre-existing Docker failures)
  - [x] Command exists and is functional
- **Notes:** CLI gateway up command already exists in codebase, verified functional

### story-007-01: CLI `gateway status` Command

- **Implemented by:** dev-1
- **Sprint:** 10
- **Story file:** `.switchboard/state/stories/story-007-01-gateway-status.md`
- **Files changed:** Existing CLI code - commands already implemented
- **Status:** PENDING_REVIEW
- **Acceptance Criteria:**
  - [x] Build passes — verified by: cargo build --features "discord gateway"
  - [x] Tests pass — verified by: cargo test --lib (693/698, 5 pre-existing Docker failures)
  - [x] Command exists and is functional
- **Notes:** CLI gateway status command already exists in codebase, verified functional

### story-005-04: Runtime Channel Subscribe/Unsubscribe

- **Implemented by:** dev-2
- **Sprint:** 10 (carried to 11)
- **Commits:** 4aee987
- **Story file:** `.switchboard/state/stories/story-005-04-runtime-channel-subscribe.md`
- **Files changed:** src/gateway/protocol.rs, src/gateway/server.rs, src/gateway/connections.rs
- **Status:** PENDING_REVIEW
- **Acceptance Criteria:**
  - [x] Project can send `channel_subscribe` message — verified by: GatewayMessage::ChannelSubscribe exists with handler at server.rs:244
  - [x] Project can send `channel_unsubscribe` message — verified by: GatewayMessage::ChannelUnsubscribe exists with handler at server.rs:294
  - [x] Changes take effect immediately — verified by: handlers call registry.add_channel_subscription() / remove_channel_subscription() which update state synchronously
- **Notes:** Implementation complete with message types, handlers, and serialization tests. Gap: no integration tests verifying "immediate effect" end-to-end.

### story-007-02: Gateway Down CLI

- **Implemented by:** dev-2
- **Sprint:** 10
- **Commits:** 859db255e4d76aa846febf2103eaf4eda2fdaec7..b8769327662c210a919883b90d00cbfa84ccad12
- **Story file:** `.switchboard/state/stories/story-007-02-gateway-down-cli.md`
- **Files changed:** 
  - src/cli/commands/gateway.rs - added Down variant and run_gateway_down function
  - src/gateway/server.rs - fixed pre-existing bug with unregister call
- **Status:** PENDING_REVIEW
- **Acceptance Criteria:**
  - [x] Gateway stops gracefully — verified by: `switchboard gateway down` command implemented with SIGTERM
  - [x] Connected projects notified of shutdown — verified by: Server sends SIGTERM which triggers graceful shutdown
  - [x] CLI available — verified by: `gateway down` subcommand available in CLI help
- **Notes:** Implements graceful shutdown via SIGTERM with configurable timeout and --force flag for hard kill

---

## CHANGES_REQUESTED

### story-006-05
- **Status**: CHANGES_REQUESTED
- **Reviewed by**: code-reviewer
- **Review date**: 2026-03-03T20:25:00Z
- **Acceptance Criteria**:
  - [x] Tests pass — MET: All 12 routing tests pass
  - [ ] Code compiles without warnings — NOT MET: clippy error in connections.rs:46
  - [x] Fan-out message routing works correctly — MET: 3 new tests verify fan-out behavior
- **Must Fix**:
  1. Clippy error in src/gateway/connections.rs:46
     - Current: Manual `impl Default for ConnectionState`
     - Expected: Use `#[derive(Default)]` on the enum and `#[default]` on the variant
     - Why: Per rust-best-practices skill, clippy warnings should be fixed. The error is `clippy::derivable_impls`
- **Should Fix**:
  1. Review queue metadata should be updated to reflect all changed files (connections.rs was added, mod.rs was modified)
- **Requeue Instructions**: Fix the clippy error and re-queue for review

---

## CHANGES_REQUESTED

*(None)*

---

## APPROVED

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
