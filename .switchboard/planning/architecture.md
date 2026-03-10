# Architecture — Discord Gateway Service

> Generated: 2026-03-02
> Source: .switchboard/input/FEATURES.md
> Project Type: brownfield (incremental)
> Scale: medium

## 1. System Overview

This document describes the architecture for a **Discord Gateway Service** - a standalone service that allows multiple switchboard projects to share a single Discord token while handling different channels independently.

The existing switchboard codebase already includes a `DiscordGateway` implementation in `src/discord/gateway.rs` that connects a single bot to Discord. This feature extends that capability to support **multi-project channel routing**.

### Key Capabilities
1. **Single Discord Connection** - One WebSocket connection to Discord, shared by all projects
2. **Channel-based Routing** - Messages routed to projects based on channel configuration
3. **Project Isolation** - Each project connects via WebSocket, receives only its subscribed channels
4. **Fan-out Support** - Multiple projects can subscribe to the same channel

## 2. Technology Stack

| Layer | Choice | Rationale | Status |
|-------|--------|-----------|--------|
| Language | Rust 2021 edition | Existing codebase standard | existing |
| Framework | tokio + twilight-gateway | Already in use | existing |
| WebSocket | tokio-tungstenite | For project connections | **new** |
| HTTP Server | axum | Lightweight, async-native | **new** |
| Serialization | serde_json | Already in use | existing |
| Config | toml + serde | Already in use | existing |
| Error Handling | thiserror | Already in use | existing |
| Logging | tracing | Already in use | existing |

### New Dependencies Required
```toml
tokio-tungstenite = "0.24"
futures-util = "0.3"
axum = "0.7"
tower = "0.4"
```

## 3. Project Structure

```
src/
├── gateway/                    ← NEW MODULE
│   ├── mod.rs                  # Module exports
│   ├── config.rs               # Gateway configuration
│   ├── server.rs               # HTTP/WS server
│   ├── protocol.rs             # Message protocol types
│   ├── routing.rs              # Message routing logic
│   ├── registry.rs             # Channel/project registry
│   ├── connections.rs          # Connection management
│   ├── heartbeat.rs            # Heartbeat protocol
│   ├── ratelimit.rs           # Discord rate limiting
│   └── client.rs               # Client library for projects
│
├── cli/
│   └── commands/
│       └── gateway.rs          # Gateway CLI commands (ADD)
│
└── discord/
    └── gateway.rs              # Existing - used by gateway service
```

## 4. Key Design Decisions

### ADR-001: Gateway as Standalone Service

- **Status:** Accepted
- **Context:** The FEATURES.md specifies a standalone gateway that can serve multiple switchboard projects. This is different from the current inline Discord bot integration.
- **Decision:** Create a new `gateway` module that runs as a separate service. Projects connect to it via WebSocket.
- **Consequences:** 
  - Positive: Clear separation of concerns, projects can be on different machines
  - Negative: Additional deployment complexity, need to manage gateway process

### ADR-002: HTTP + WebSocket for Project Communication

- **Status:** Accepted
- **Context:** Projects need to receive Discord events and send commands
- **Decision:** Use HTTP for management/monitoring, WebSocket for event streaming
- **Alternatives Considered:** 
  - Pure WebSocket: No good for HTTP-based health checks
  - gRPC: Overkill for this use case

### ADR-003: Protocol Buffers Not Required

- **Status:** Accepted
- **Context:** FEATURES.md specifies JSON for messages
- **Decision:** Use JSON serialization (already available via serde_json)
- **Alternatives Considered:** Protobuf - would add build complexity, JSON is sufficient

## 5. Module Specifications

### 5.1 gateway::config

- **Purpose:** Load and validate gateway configuration
- **Public API:**
  - `GatewayConfig::load(path: Option<&str>) -> Result<Self, ConfigError>`
  - `GatewayConfig::from_env() -> Result<Self, ConfigError>`
- **Dependencies:** toml, serde
- **Data flow:** TOML file → parse → validated config struct

### 5.2 gateway::server

- **Purpose:** HTTP and WebSocket server for gateway
- **Public API:**
  - `GatewayServer::new(config: GatewayConfig) -> Self`
  - `GatewayServer::run().await`
- **Dependencies:** axum, tokio-tungstenite, tower
- **Data flow:** 
  - HTTP requests → route handlers
  - WebSocket connections → project session management

### 5.3 gateway::registry

- **Purpose:** Track channel-to-project mappings
- **Public API:**
  - `ChannelRegistry::register(project: ProjectConnection, channels: Vec<String>)`
  - `ChannelRegistry::unregister(project_id: &ProjectId)`
  - `ChannelRegistry::projects_for_channel(channel_id: &str) -> &[ProjectId]`
- **Dependencies:** tokio::sync::RwLock
- **Data flow:** Maintained in memory, updated on project connect/disconnect

### 5.4 gateway::protocol

- **Purpose:** Define message types for gateway<->project communication
- **Public API:** Enums and structs for register, message, heartbeat
- **Dependencies:** serde, serde_json

### 5.5 gateway::routing

- **Purpose:** Route Discord events to appropriate projects
- **Public API:**
  - `MessageRouter::route(event: DiscordEvent) -> Vec<ProjectMessage>`
- **Dependencies:** registry, protocol

### 5.6 gateway::client

- **Purpose:** Client library for projects to connect to gateway
- **Public API:**
  - `GatewayClient::connect(gateway_url: &str, project_name: String, channels: Vec<String>) -> Result<Self>`
  - `GatewayClient::recv() -> Result<GatewayMessage>`
  - `GatewayClient::heartbeat().await`
- **Dependencies:** tokio-tungstenite, futures-util

## 6. Data Model

### Entities

#### GatewayConfig
```rust
struct GatewayConfig {
    discord_token: String,      // Bot token
    server: ServerConfig,        // HTTP/WS ports
    logging: LoggingConfig,     // Logging settings
    channels: Vec<ChannelMapping>,
}

struct ChannelMapping {
    channel_id: String,
    project_name: String,
    endpoint: String,            // Project's WebSocket endpoint
}
```

#### ProjectConnection
```rust
struct ProjectConnection {
    project_id: ProjectId,
    project_name: String,
    ws_sender: mpsc::Sender<String>,
    session_id: Uuid,
    subscribed_channels: Vec<String>,
    registered_at: DateTime<Utc>,
}
```

#### GatewayMessage (protocol)
```rust
enum GatewayMessage {
    Register { project_name: String, channels: Vec<String> },
    RegisterAck { status: String, session_id: Uuid },
    Message { channel_id: String, content: String, ... },
    Heartbeat { session_id: Uuid },
    HeartbeatAck { server_time: i64 },
}
```

## 7. Error Handling Strategy

- **Pattern:** Use `thiserror` for error types (following existing codebase pattern)
- **Error Types:**
  - `GatewayError` - general errors
  - `ConfigError` - configuration loading errors
  - `ProtocolError` - message parsing errors
  - `ConnectionError` - WebSocket errors
- **User-facing errors:** Return structured errors with clear messages
- **Internal errors:** Log with tracing, propagate as Result

Reference: `./skills/rust-engineer/references/error-handling.md`

## 8. Testing Strategy

- **Unit tests:** Config loading, message protocol parsing, registry operations
- **Integration tests:** Gateway to Discord connection, project registration flow
- **Test commands:** 
  - `cargo test --lib`
  - `cargo test --test integration` (if added)

Reference: `./skills/rust-best-practices/SKILL.md` §Testing

## 9. Non-Functional Requirements

| NFR | Architectural Response |
|-----|----------------------|
| Single Discord connection | Gateway maintains one twilight-gateway connection |
| Channel routing | ChannelRegistry maps channel_id → projects |
| Connection resilience | Reconnection with exponential backoff |
| Fan-out support | Route message to all subscribed projects |

## 10. Scope Boundaries

### In Scope
- Gateway HTTP server with health check
- WebSocket server for project connections
- Channel registry for routing
- Project registration protocol
- CLI commands: `gateway up`, `gateway status`, `gateway down`
- Client library for projects

### Out of Scope
- Gateway clustering/high availability
- Persistent message storage
- Authentication beyond project names
- Gateway-to-gateway communication

### Future Considerations
- Project authentication with shared secrets
- TLS (wss://) for production
- Gateway metrics/observability
- Multiple gateway instances

---

## References

- Skill: [Rust Best Practices](./skills/rust-best-practices/SKILL.md)
- Skill: [Rust Engineer](./skills/rust-engineer/SKILL.md)
- Existing Code: [src/discord/gateway.rs](../src/discord/gateway.rs) - Twilight gateway usage
- Existing Code: [src/cli/mod.rs](../src/cli/mod.rs) - CLI patterns
