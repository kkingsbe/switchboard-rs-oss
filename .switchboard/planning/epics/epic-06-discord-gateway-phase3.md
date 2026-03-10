# Epic 06: Discord Gateway - Multi-Project Support & Reconnection

> Priority: 3
> Depends on: Epic 05
> Estimated stories: 6
> New stories: 6
> Already implemented: 0
> Status: not-started

## Description

Implement robust multi-project support with reconnection handling, heartbeat protocol, and rate limiting.

## Stories

### Story 6.1: Implement project connection management

- **Points:** 3
- **Depends on:** 4.6
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** system,
**I want** to track all connected projects,
**So that** I can manage their state and route messages correctly.

**Acceptance Criteria:**

1. Track active connections with project_id, session_id, subscription info
   - Verification: Connection list accurate
2. Handle multiple simultaneous project connections
   - Verification: Can connect 3+ projects
3. Detect and clean up stale connections
   - Verification: Dead connections removed after timeout

**Technical Notes:**

- Files to create: `src/gateway/connections.rs`
- Dependencies: tokio::sync::mpsc, std::collections::HashMap

---

### Story 6.2: Add heartbeat protocol

- **Points:** 2
- **Depends on:** 6.1
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** system,
**I want** to detect when projects disconnect unexpectedly,
**So that** I can stop routing messages to them.

**Acceptance Criteria:**

1. Projects send heartbeat every 30 seconds
   - Verification: Heartbeat received and processed
2. Gateway responds with heartbeat_ack
   - Verification: Ack sent with server timestamp
3. Mark project as disconnected if no heartbeat for 90 seconds
   - Verification: Project removed after timeout

**Technical Notes:**

- Files to create: `src/gateway/heartbeat.rs`
- Files to modify: `src/gateway/protocol.rs`

---

### Story 6.3: Implement reconnection logic

- **Points:** 3
- **Depends on:** 6.2
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** projects to automatically reconnect if they drop,
**So that** message delivery resumes without manual intervention.

**Acceptance Criteria:**

1. Project can reconnect with same session_id
   - Verification: Reconnection preserves subscription
2. Implement exponential backoff (1s, 2s, 4s... max 60s)
   - Verification: Backoff increases correctly
3. After max retries, mark project as failed
   - Verification: Failure status reported

**Technical Notes:**

- Dependencies: tokio::time, tokio::sync::watch

---

### Story 6.4: Handle project disconnections gracefully

- **Points:** 2
- **Depends on:** 6.1
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** system,
**I want** to handle project WebSocket disconnections cleanly,
**So that** the gateway doesn't crash or leak resources.

**Acceptance Criteria:**

1. Detect WebSocket close events
   - Verification: Disconnection logged
2. Remove project from routing
   - Verification: Messages not sent to disconnected project
3. Allow project to re-register
   - Verification: Same project can reconnect

**Technical Notes:**

- Files to modify: `src/gateway/server.rs`, `src/gateway/connections.rs`

---

### Story 6.5: Implement fan-out message delivery

- **Points:** 2
- **Depends on:** 5.3
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** multiple projects to receive messages from the same channel,
**So that** different projects can process the same Discord messages.

**Acceptance Criteria:**

1. When a message arrives on a channel with multiple subscribers
   - Verification: All subscribed projects receive the message
2. Failure to one project doesn't affect others
   - Verification: Other projects still receive message
3. Messages delivered in Discord event order
   - Verification: Order preserved per project

**Technical Notes:**

- Files to modify: `src/gateway/routing.rs`

---

### Story 6.6: Implement Discord rate limit handling

- **Points:** 2
- **Depends on:** 4.7
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** system,
**I want** to respect Discord's rate limits,
**So that** I don't get the bot suspended.

**Acceptance Criteria:**

1. Track requests per channel
   - Verification: Rate limit tracked correctly
2. Handle 429 responses with Retry-After header
   - Verification: Wait time respected
3. Implement exponential backoff on repeated rate limits
   - Verification: Backoff increases on continued 429s

**Technical Notes:**

- Files to create: `src/gateway/ratelimit.rs`
- Dependencies: std::collections::HashMap, tokio::time
