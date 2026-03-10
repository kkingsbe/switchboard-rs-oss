# Story: story-001-docker-connection-trait - Docker Connection Trait

## Metadata

- **Story ID**: story-001-docker-connection-trait
- **Title**: Docker Connection Trait
- **Epic**: Epic 001 - Docker Integration
- **Points**: 3
- **Type**: feature
- **Risk Level**: Medium
- **Status**: Implemented

---

## User Story

As a developer, I want a Docker connection trait so that I can easily mock Docker operations in tests and maintain a clean abstraction over Docker functionality.

---

## Acceptance Criteria

1. `DockerConnectionTrait` trait is defined with essential methods
2. Trait is object-safe with `Send + Sync` bounds
3. `RealDockerConnection` provides production implementation
4. `MockDockerConnection` provides test implementation with builder pattern
5. Trait supports: connect, disconnect, check availability, execute commands
6. Mock builder allows configuring all behaviors for tests
7. Error types are comprehensive and informative

**Test Methods**:
- `MockDockerConnection` can be configured with custom responses
- Mock correctly simulates connection success/failure
- Mock correctly simulates Docker availability
- Trait object can be used interchangeably

---

## Technical Context

### Architecture References

The Docker connection trait provides an abstraction layer over Docker operations, following the trait-based design pattern for testability.

### Existing Code

- Docker client: `src/docker/client.rs`
- Docker module: `src/docker/mod.rs`
- Docker errors: `src/docker/mod.rs` (DockerError enum)

---

## Implementation Plan

1. Define `DockerConnectionTrait` with methods:
   - `get_docker_socket_path() -> Result<String, DockerError>`
   - `connect() -> Result<Docker, DockerError>`
   - `disconnect() -> Pin<Box<dyn Future<Output = Result<(), DockerError>> + Send>>`
   - `check_docker_available() -> Result<bool, DockerError>`
   - `execute(cmd: DockerCommand) -> Pin<Box<dyn Future<Output = Result<DockerResponse, DockerError>> + Send>>`
2. Define `DockerCommand` enum for executable commands
3. Define `DockerResponse` enum for command responses
4. Implement `RealDockerConnection` delegating to existing client.rs functions
5. Implement `MockDockerConnection` with builder pattern:
   - Configure socket path
   - Configure connect success/failure
   - Configure availability
   - Configure custom responses
6. Add comprehensive tests

---

## Skills to Read

- [Rust Engineer](../../skills/rust-engineer/SKILL.md)
- [Traits Reference](../../skills/rust-engineer/references/traits.md)
- [Testing Reference](../../skills/rust-engineer/references/testing.md)
- [Async Reference](../../skills/rust-engineer/references/async.md)

---

## Dependencies

- `bollard` for Docker client
- `tokio` for async runtime
- `futures-util` for async trait support

---

## Scope Boundaries

### In Scope
- Trait definition with essential Docker operations
- Production implementation
- Mock implementation for testing
- Builder pattern for mock configuration

### Out of Scope
- All Docker commands (execute method can be extended)
- Connection pooling optimization
- Docker context management

---

## Files in Scope

| File | Description |
|------|-------------|
| `src/docker/connection.rs` | Trait and implementations |
| `src/docker/client.rs` | Existing Docker client (dependency) |
| `src/docker/mod.rs` | Module exports and error types |

---

## Trait Definition

```rust
pub trait DockerConnectionTrait: Send + Sync {
    fn get_docker_socket_path(&self) -> Result<String, DockerError>;
    fn connect(&self) -> Result<Docker, DockerError>;
    fn disconnect(&self) -> Pin<Box<dyn Future<Output = Result<(), DockerError>> + Send>>;
    fn check_docker_available(&self) -> Result<bool, DockerError>;
    fn execute(&self, cmd: DockerCommand) -> Pin<Box<dyn Future<Output = Result<DockerResponse, DockerError>> + Send>>;
}
```

---

## Mock Builder Usage

```rust
let mock = MockDockerConnectionBuilder::new()
    .with_socket_path(Some("/custom/socket".to_string()))
    .with_connect_success(true)
    .with_available(true)
    .with_execute_response(Some(DockerResponse::Ping { result: "OK".to_string() }))
    .build();

let trait_ref: &dyn DockerConnectionTrait = &mock;
```

---

## Test Benefits

- Unit tests don't require Docker daemon
- Tests can simulate various failure scenarios
- Tests can provide predictable responses
- Integration points are clearly defined
