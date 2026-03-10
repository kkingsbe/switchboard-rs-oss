# Story 001: Docker Connection Trait

> Epic: epic-01 — Docker Module
> Points: 3
> Sprint: 12
> Type: feature
> Risk: medium
> Created: 2026-03-04
> Status: ready
> Assigned To: dev-2

## User Story

As a developer, I want a Docker connection trait, So that I can abstract over different Docker clients.

## Acceptance Criteria

1. Define DockerConnection trait with connect(), disconnect(), execute() methods
   - **Test:** Trait compiles - Verify trait definition compiles without errors

2. Implement trait for existing Docker client
   - **Test:** Implementation works - Verify the trait is implemented for the existing Docker client

3. Use trait in docker/mod.rs
   - **Test:** Code compiles - Verify module uses trait instead of concrete type

## Technical Context

### Architecture Reference

From architecture.md:
- Docker module provides container management
- Currently uses bollard crate for Docker API
- Need abstraction for testing and future client changes

### Project Conventions

From project-context.md:
- Build: `cargo build`
- Test: `cargo test --lib`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt`
- Use `thiserror` for error types
- Use `tracing` for logging
- Never use `unwrap()` in production
- Use traits for abstraction

### Existing Code Context

```
src/docker/
├── mod.rs (docker module entry)
├── client.rs (Docker client implementation)
├── connection.rs (TO BE CREATED - this story)
├── build.rs (Docker build functionality)
├── run.rs (Docker run functionality)
└── skills.rs (Docker skills)
```

**Current state:**
- `src/docker/client.rs` has Docker client implementation using bollard
- `src/docker/connection.rs` does not exist yet - this is the file to create

## Implementation Plan

1. **Create** `src/docker/connection.rs` with:
   - `DockerConnection` trait definition:
     - `connect() -> impl Future<Output = Result<(), DockerError>>`
     - `disconnect() -> impl Future<Output = Result<(), DockerError>>`
     - `execute(cmd: DockerCommand) -> impl Future<Output = Result<DockerResponse, DockerError>>`
   - Error type: `DockerError` using thiserror
   - Command and Response types for execute()

2. **Implement** trait for existing client:
   - Create `DockerClient` struct that wraps existing bollard client
   - Implement `DockerConnection` trait for `DockerClient`
   - Delegate to existing bollard methods

3. **Update** `src/docker/mod.rs`:
   - Export the trait and new types
   - Use trait bounds instead of concrete types where appropriate
   - Update any functions that take DockerClient to use the trait

4. **Test** the implementation:
   - Verify trait definition compiles
   - Verify implementation compiles
   - Verify module compiles with trait usage

### Skills to Read

- `./skills/rust-best-practices/SKILL.md` — Rust best practices
- `./skills/rust-engineer/SKILL.md` — Core Rust patterns
- `./skills/rust-engineer/references/traits.md` — Trait design patterns

### Dependencies

- None (foundational story)

## Scope Boundaries

### This Story Includes
- DockerConnection trait definition
- Trait implementation for existing Docker client
- Module integration with trait usage
- Basic error handling

### This Story Does NOT Include
- Mock implementation for testing (future)
- Connection pooling
- Multiple Docker host support
- Async trait methods (use regular async fn for now)

### Files in Scope

- `src/docker/connection.rs` — create (new file)
- `src/docker/client.rs` — modify (implement trait)
- `src/docker/mod.rs` — modify (use trait)

### Files NOT in Scope

- `src/docker/build.rs` — don't modify
- `src/docker/run.rs` — don't modify
- `src/docker/skills.rs` — don't modify
- Test mocks — defer to future
