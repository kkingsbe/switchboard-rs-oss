# Story 003: Refactor docker/mod.rs

> Epic: epic-testability — Testability Enhancement
> Points: 5
> Sprint: 17
> Type: infrastructure
> Risk: medium
> Created: 2026-03-05T00:50:00Z

## User Story

As a developer, I want DockerClient to use the traits created in Stories 001-002, so that the Docker module can be tested without a Docker daemon.

## Acceptance Criteria

1. Refactor `DockerClient` to accept `Arc<dyn DockerConnectionTrait>`
   - **Test:** Verify DockerClient can be constructed with MockDockerConnection

2. Add trait bounds to existing `DockerClient` methods
   - **Test:** All DockerClient methods work with mock trait

3. Ensure existing functionality preserved
   - **Test:** Run `cargo build --features "discord gateway"` - must compile

4. Run existing docker tests with mock
   - **Test:** `cargo test --lib docker` - tests pass without Docker daemon

5. Verify code compiles without Docker feature
   - **Test:** `cargo check --no-default-features` - must compile

## Technical Context

### Architecture Reference

The Docker module uses trait-based dependency injection for testability:
- `DockerConnectionTrait` in `src/docker/connection.rs` defines the interface
- `RealDockerConnection` is the production implementation
- `MockDockerConnection` is for testing

### Current DockerClient Structure

From `src/docker/client.rs` (lines 391-400):
```rust
pub struct DockerClient {
    client: Arc<dyn DockerClientTrait>,  // Already has trait object!
    docker: Option<Docker>,               // Direct bollard client
    _image_name: String,
    _image_tag: String,
}
```

### Files in Scope
- `src/docker/client.rs` — modify (refactor to use trait consistently)
- `src/docker/mod.rs` — modify (update exports if needed)

### Files NOT in Scope
- `src/docker/run.rs` — do NOT modify
- `src/docker/build.rs` — do NOT modify  
- `src/docker/connection.rs` — already complete from story-001

## Implementation Plan

1. **Audit current trait usage** — Review how DockerClient currently uses both `client` (trait) and `docker` (direct) fields

2. **Refactor to use trait consistently** — Modify DockerClient to delegate all Docker operations through the trait, removing direct `docker` field dependency where possible

3. **Update mod.rs exports** — Ensure proper re-exports

4. **Run build verification** — `cargo build --features "discord gateway"`

5. **Run tests** — `cargo test --lib docker`

6. **Verify no-Docker compilation** — `cargo check --no-default-features`

### Skills to Read
- `./skills/rust-best-practices/SKILL.md` — Rust code quality
- `./skills/rust-engineer/SKILL.md` — Rust systems programming
- `./skills/rust-engineer/references/traits.md` — Trait patterns
- `./skills/rust-engineer/references/testing.md` — Testing patterns

### Dependencies
- story-001-docker-connection-trait — COMPLETE
- story-002-process-executor-trait — COMPLETE

## Scope Boundaries

### This Story Includes
- Refactoring DockerClient to consistently use DockerConnectionTrait
- Ensuring tests can run without Docker daemon
- Verifying compilation in various configurations

### This Story Does NOT Include
- Modifying run.rs or build.rs
- Adding new Docker functionality
- Integration tests with actual Docker daemon
