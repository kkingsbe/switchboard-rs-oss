# Epic 07: Discord Gateway - CLI Integration & Monitoring

> Priority: 4
> Depends on: Epic 06
> Estimated stories: 5
> New stories: 5
> Already implemented: 0
> Status: not-started

## Description

Complete the CLI experience with status monitoring, shutdown commands, and client library for projects.

## Stories

### Story 7.1: Implement `switchboard gateway status`

- **Points:** 2
- **Depends on:** 6.1
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** to see the gateway status,
**So that** I know if it's running and what's connected.

**Acceptance Criteria:**

1. Show gateway status (running/stopped)
   - Verification: Status displayed correctly
2. Show Discord connection status
   - Verification: Shows connected/disconnected
3. Show connected projects and their channels
   - Verification: List of projects and subscriptions displayed

**Technical Notes:**

- Files to modify: `src/cli/commands/gateway.rs`
- Add HTTP endpoint: GET `/status` returning JSON

---

### Story 7.2: Implement `switchboard gateway down`

- **Points:** 2
- **Depends on:** 4.8
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** to stop the gateway,
**So that** I can shut down the service.

**Acceptance Criteria:**

1. Gateway stops gracefully
   - Verification: Clean shutdown with no errors
2. Connected projects notified of shutdown
   - Verification: Projects receive shutdown message
3. Discord connection closed properly
   - Verification: No stale connections

**Technical Notes:**

- Files to modify: `src/cli/commands/gateway.rs`, `src/gateway/server.rs`

---

### Story 7.3: Add PID file management

- **Points:** 1
- **Depends on:** 4.8
- **Risk:** Low
- **Type:** infrastructure
- **Status:** not-started

**As a** user,
**I want** the gateway to track its PID,
**So that** I can manage the process externally.

**Acceptance Criteria:**

1. Write PID to file on start (default: `.switchboard/gateway.pid`)
   - Verification: File created with correct PID
2. Check for existing PID on startup
   - Verification: Error if gateway already running
3. Clean up PID file on shutdown
   - Verification: File removed on clean exit

**Technical Notes:**

- Files to modify: `src/gateway/server.rs`
- Use std::fs for PID file operations

---

### Story 7.4: Add proper logging

- **Points:** 2
- **Depends on:** 4.1
- **Risk:** Low
- **Type:** infrastructure
- **Status:** not-started

**As a** operator,
**I want** detailed logs from the gateway,
**So that** I can troubleshoot issues.

**Acceptance Criteria:**

1. Log gateway startup with configuration
   - Verification: Startup info logged
2. Log project connections/disconnections
   - Verification: Connection events logged
3. Log Discord events (connection, reconnection, errors)
   - Verification: Discord events visible in logs
4. Log to file in addition to stdout
   - Verification: Log file created

**Technical Notes:**

- Files to modify: `src/gateway/*.rs`
- Use existing logging: crate::logging, tracing
- Log file: `.switchboard/gateway.log`

---

### Story 7.5: Create gateway client library

- **Points:** 3
- **Depends on:** 4.5, 4.6, 6.2
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** project developer,
**I want** a client library to connect to the gateway,
**So that** I don't have to implement the WebSocket protocol manually.

**Acceptance Criteria:**

1. Create `GatewayClient` struct
   - Verification: Can be instantiated
2. Implement `connect()` method
   - Verification: Establishes WebSocket connection
3. Implement `recv()` to receive messages
   - Verification: Receives Discord messages
4. Implement heartbeat automatically
   - Verification: Heartbeat sent in background

**Technical Notes:**

- Files to create: `src/gateway/client.rs`
- Document as public API for projects to use

---

## Summary

Total stories across all epics: 24 stories

| Epic | Phase | Stories | Points |
|------|-------|---------|--------|
| Epic 04 | Phase 1: Basic Gateway | 8 | 22 |
| Epic 05 | Phase 2: Channel Routing | 5 | 11 |
| Epic 06 | Phase 3: Multi-Project | 6 | 14 |
| Epic 07 | Phase 4: CLI Integration | 5 | 10 |

**Total: 24 stories, 57 points**
