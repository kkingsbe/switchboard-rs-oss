# Epic 04: Discord Gateway - Basic Gateway with Single Project

> Priority: 1
> Depends on: None
> Estimated stories: 8
> New stories: 8
> Already implemented: 0
> Status: not-started

## Description

Implement the basic Discord Gateway Service that connects to Discord and forwards messages to a single project. This is the foundation for multi-project support.

## Stories

### Story 4.1: Create gateway module structure

- **Points:** 1
- **Depends on:** None
- **Risk:** Low
- **Type:** infrastructure
- **Status:** not-started

**As a** developer,
**I want** a basic gateway module structure,
**So that** I can organize the gateway code.

**Acceptance Criteria:**

1. Create `src/gateway/mod.rs` with module declarations
   - Verification: File exists and compiles
2. Add `pub mod gateway` to `src/lib.rs`
   - Verification: `cargo build` succeeds
3. Add feature flag `gateway` to Cargo.toml
   - Verification: `cargo build --features gateway` compiles

**Technical Notes:**

- Files to create: `src/gateway/mod.rs`
- Files to modify: `src/lib.rs`, `Cargo.toml`
- Pattern: Follow existing module patterns in `src/discord/`
- Skills: See `./skills/rust-best-practices/SKILL.md`

---

### Story 4.2: Implement gateway configuration loading

- **Points:** 2
- **Depends on:** 4.1
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** developer,
**I want** to load gateway configuration from a TOML file,
**So that** the gateway can be configured without hardcoding.

**Acceptance Criteria:**

1. Create `GatewayConfig` struct with fields: discord_token, server, logging, channels
   - Verification: Unit tests pass
2. Implement `GatewayConfig::load(path: Option<&str>)` to load from `gateway.toml`
   - Verification: Can load sample config file
3. Support environment variable expansion for discord_token (e.g., `${DISCORD_TOKEN}`)
   - Verification: Token loaded from env var when specified

**Technical Notes:**

- Files to create: `src/gateway/config.rs`
- Pattern: Follow `src/config/mod.rs` patterns
- Dependencies: toml, serde, crate::config::env

---

### Story 4.3: Create HTTP server with health check endpoint

- **Points:** 3
- **Depends on:** 4.2
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** system administrator,
**I want** a health check endpoint on the gateway,
**So that** I can monitor if the gateway is running.

**Acceptance Criteria:**

1. Create HTTP server on configured port (default 9745)
   - Verification: Server starts and binds to port
2. Implement GET `/health` endpoint returning JSON `{"status": "ok"}`
   - Verification: `curl http://localhost:9745/health` returns 200
3. Add graceful shutdown handling
   - Verification: Server stops cleanly on SIGINT

**Technical Notes:**

- Files to create: `src/gateway/server.rs`
- Dependencies: axum, tower
- Pattern: Follow async patterns in `src/discord/gateway.rs`

---

### Story 4.4: Implement WebSocket server for project connections

- **Points:** 3
- **Depends on:** 4.3
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** project developer,
**I want** to connect my project to the gateway via WebSocket,
**So that** I can receive Discord messages.

**Acceptance Criteria:**

1. Create WebSocket endpoint at `/ws`
   - Verification: WebSocket connection accepts upgrade request
2. Handle WebSocket connections and parse incoming messages
   - Verification: Can receive and parse JSON messages
3. Echo received messages back for testing
   - Verification: Simple round-trip test passes

**Technical Notes:**

- Files to create/modify: `src/gateway/server.rs`
- Dependencies: tokio-tungstenite, futures-util

---

### Story 4.5: Define message protocol types

- **Points:** 2
- **Depends on:** 4.1
- **Risk:** Low
- **Type:** feature
- **Status:** not-started

**As a** developer,
**I want** clear message protocol types,
**So that** the gateway and projects can communicate reliably.

**Acceptance Criteria:**

1. Define `GatewayMessage` enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
   - Verification: Types serialize/deserialize correctly
2. Implement serde serialization/deserialization
   - Verification: JSON round-trip tests pass
3. Document protocol in code comments
   - Verification: Doc tests pass

**Technical Notes:**

- Files to create: `src/gateway/protocol.rs`
- Dependencies: serde, serde_json, uuid

---

### Story 4.6: Implement basic registration protocol

- **Points:** 3
- **Depends on:** 4.4, 4.5
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** project developer,
**I want** to register my project with the gateway,
**So that** I can receive Discord messages.

**Acceptance Criteria:**

1. Project sends `{"type": "register", "project_name": "...", "channels": [...]}`
   - Verification: Message parsed correctly
2. Gateway responds with `{"type": "register_ack", "status": "ok", "session_id": "..."}`
   - Verification: Registration completes successfully
3. Invalid registration returns `{"type": "register_error", "error": "..."}`
   - Verification: Error case handled gracefully

**Technical Notes:**

- Files to modify: `src/gateway/server.rs`, `src/gateway/protocol.rs`

---

### Story 4.7: Wire up Discord Gateway connection

- **Points:** 5
- **Depends on:** 4.2, 4.6
- **Risk:** High
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** the gateway to connect to Discord,
**So that** it can receive messages from my channels.

**Acceptance Criteria:**

1. Gateway connects to Discord using twilight-gateway
   - Verification: Connection established, Ready event received
2. Gateway listens for MessageCreate events
   - Verification: Messages logged/forwarded
3. Handle reconnection on disconnect
   - Verification: Auto-reconnect after Discord disconnect

**Technical Notes:**

- Files to modify: `src/gateway/server.rs`
- Existing code: `src/discord/gateway.rs` (DiscordGateway)
- Dependencies: twilight-gateway (already in Cargo.toml)

---

### Story 4.8: Add CLI command `switchboard gateway up`

- **Points:** 3
- **Depends on:** 4.7
- **Risk:** Medium
- **Type:** feature
- **Status:** not-started

**As a** user,
**I want** to start the gateway from the CLI,
**So that** I can easily run the gateway service.

**Acceptance Criteria:**

1. CLI has `gateway` subcommand with `up` action
   - Verification: `switchboard gateway up --help` shows usage
2. Command starts gateway with config from `gateway.toml`
   - Verification: Gateway starts and connects to Discord
3. Support `--config` flag for custom config path
   - Verification: Custom config file loads correctly
4. Support `--detach` flag to run in background (future)
   - Verification: Not required for MVP

**Technical Notes:**

- Files to create: `src/cli/commands/gateway.rs`
- Files to modify: `src/cli/mod.rs`
- Pattern: Follow existing CLI command patterns
