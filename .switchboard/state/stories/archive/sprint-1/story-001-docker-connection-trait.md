# Story 001: Define DockerConnectionTrait

> Epic: epic-testability — Testability Enhancement
> Points: 3
> Sprint: 1
> Type: infrastructure
> Risk: low
> Created: 2026-03-02

## User Story

As a developer,
I want to be able to test Docker connection logic without a running Docker daemon,
So that I can run unit tests in CI/CD pipelines without Docker.

## Acceptance Criteria

1. Define `DockerConnectionTrait` with methods:
   - `get_docker_socket_path() -> Result<String, DockerError>`
   - `connect_to_docker() -> Result<Docker, DockerError>`
   - `check_docker_available() -> Result<bool, DockerError>`
   
2. Create `MockDockerConnection` struct implementing the trait for tests

3. **Test:** Write unit tests using `MockDockerConnection` that verify connection timeout handling
   - Run: `cargo test docker_connection --no-default-features`

4. **Test:** Verify trait object safety (dyn Trait)
   - Run: `cargo test trait_object`

## Technical Context

### Architecture Reference
From architecture.md - Module: docker/client.rs
The Docker client currently directly uses bollard::Docker. This story abstracts the connection layer.

### Project Conventions
- Use thiserror for error types
- No unwrap() in production code
- Run cargo clippy before commit

### Existing Code Context
Current `src/docker/client.rs`:
- `get_docker_socket_path()` - Gets socket path with 5s timeout
- `connect_to_docker()` - Connects to Docker via socket  
- `check_docker_available()` - Checks Docker daemon is running

```
src/docker/
├── client.rs    # Contains connection logic to abstract
├── mod.rs       # Re-exports
└── run/         # Submodules
```

## Implementation Plan

1. Create `src/docker/connection.rs` - Define `DockerConnectionTrait`
2. Update `src/docker/client.rs` - Implement trait for real Docker, add `MockDockerConnection`
3. Update `src/docker/mod.rs` - Export new trait
4. Write tests in `src/docker/`
5. Run build + clippy + tests

### Skills to Read
- `./skills/rust-engineer/SKILL.md` — Rust engineering patterns
- `./skills/rust-engineer/references/traits.md` — Trait design
- `./skills/rust-engineer/references/testing.md` — Testing standards

### Dependencies
None - this is a foundational story

## Scope Boundaries

### This Story Includes
- DockerConnectionTrait definition
- Mock implementation for tests
- Unit tests for connection timeout

### This Story Does NOT Include
- Refactoring existing DockerClient to use the trait
- Integration tests with real Docker daemon
- Changes to container run logic

### Files in Scope
- `src/docker/connection.rs` — create
- `src/docker/client.rs` — modify (add MockDockerConnection)
- `src/docker/mod.rs` — modify (export trait)
- `src/docker/tests/connection_tests.rs` — create

### Files NOT in Scope
- `src/docker/run.rs` 
- `src/docker/build.rs`
