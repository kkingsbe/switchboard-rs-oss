# Epic 05: Discord Gateway - Channel Routing with Config File

> Priority: 2
> Depends on: Epic 04 (phase 1 complete)
> Estimated stories: 5
> New stories: 5
> Already implemented: 0
> Status: not-started

## Description

Implement channel-based routing so that messages from different Discord channels are forwarded to the appropriate projects based on configuration.

## Stories

### Story 5.1: Implement ChannelRegistry

- **Points:** 3
- **Depends on:** 4.1
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** system,
**I want** to track which projects are subscribed to which channels,
**So that** I can route messages correctly.

**Acceptance Criteria:**

1. Create `ChannelRegistry` struct with thread-safe interior
   - Verification: Can be accessed from multiple tasks
2. Implement `register(project, channels)` method
   - Verification: Project added to channel mapping
3. Implement `unregister(project_id)` method
   - Verification: Project removed from all channels
4. Implement `projects_for_channel(channel_id)` method
   - Verification: Returns correct list of projects

**Technical Notes:**

- Files to create: `src/gateway/registry.rs`
- Dependencies: tokio::sync::RwLock, std::collections::HashMap
- Pattern: Use Arc<RwLock<T>> for shared state

---

### Story 5.2: Support channel mapping in config

- **Points:** 2
- **Depends on:** 4.2
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** to configure channel-to-project mappings in the config file,
**So that** I can control routing without code changes.

**Acceptance Criteria:**

1. Config supports `[[channels]]` array in gateway.toml
   - Verification: Sample config loads correctly
2. Each channel mapping has: channel_id, project_name, endpoint
   - Verification: Fields parse correctly
3. Validate channel IDs are numeric strings
   - Verification: Invalid config returns error

**Technical Notes:**

- Files to modify: `src/gateway/config.rs`
- Pattern: Follow serde deserialize patterns

---

### Story 5.3: Route messages by channel_id

- **Points:** 3
- **Depends on:** 5.1, 5.2, 4.7
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** messages from Discord to be routed to the correct project based on channel,
**So that** each project only receives messages from its subscribed channels.

**Acceptance Criteria:**

1. When MessageCreate event arrives, extract channel_id
   - Verification: Channel ID extracted from event
2. Look up projects subscribed to that channel
   - Verification: Correct projects returned
3. Forward message to those projects via WebSocket
   - Verification: Project receives the message

**Technical Notes:**

- Filessrc/gateway/routing.rs`
- Files to modify: `src/gateway to create: `/server.rs`

---

### Story 5.4: Support runtime channel subscribe/unsubscribe

- **Points:** 2
- **Depends on:** 5.1
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** project developer,
**I want** to change my channel subscriptions at runtime,
**So that** I can add/remove channels without restarting the gateway.

**Acceptance Criteria:**

1. Project can send `channel_subscribe` message
   - Verification: New channels added to subscription
2. Project can send `channel_unsubscribe` message
   - Verification: Channels removed from subscription
3. Changes take effect immediately
   - Verification: Next message uses updated subscription

**Technical Notes:**

- Files to modify: `src/gateway/protocol.rs`, `src/gateway/server.rs`

---

### Story 5.5: Add configuration validation

- **Points:** 1
- **Depends on:** 5.2
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** to get clear error messages when my config is invalid,
**So that** I can fix configuration issues quickly.

**Acceptance Criteria:**

1. Validate discord_token is not empty
   - Verification: Error if token missing
2. Validate http_port and ws_port are valid (1024-65535)
   - Verification: Error if port out of range
3. Validate channel mappings have required fields
   - Verification: Error if channel mapping incomplete

**Technical Notes:**

- Files to modify: `src/gateway/config.rs`
- Pattern: Use thiserror for validation errors
