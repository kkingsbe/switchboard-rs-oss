# Story 003: Refactor docker/mod.rs to Use Traits

> Epic: epic-testability — Testability Enhancement
> Points: 5
> Sprint: 1
> Type: infrastructure
> Risk: medium
> Created: 2026-03-02

## User Story

As a developer,
I want DockerClient to use the traits created in Stories 001-002,
So that the Docker module can be tested without a Docker daemon.

## Acceptance Criteria

1. Refactor `DockerClient` to accept `Arc<dyn DockerConnectionTrait>` 
2. Add trait bounds to existing `DockerClient` methods
3. Ensure existing functionality preserved

4. **Test:** Run existing docker tests with mock
   - Run: `cargo test docker --no-default-features`

5. **Test:** Verify code compiles without Docker feature
   - Run: `cargo check --no-default-features`

## Technical Context

### Architecture Reference
From architecture.md - The docker module needs testability via traits

### Existing Code Context
Current `src/docker/client.rs`:
```rust
pub struct DockerClient {
    client: Arc<dyn DockerClientTrait>,
    docker: Option<Docker>,
    _image_name: String,
    _image_tag: String,
}
```

## Implementation Plan

1. Refactor `DockerClient::new()` to accept connection trait
2. Add builder method `DockerClient::with_connection()`
3. Update tests to use mocks
4. Run full test suite

### Skills to Read
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/traits.md`
- `./skills/rust-best-practices/SKILL.md`

### Dependencies
- Story 001: DockerConnectionTrait
- Story 002: ProcessExecutorTrait

## Scope Boundaries

### This Story Includes
- Refactoring DockerClient to use traits
- Updating constructors

### This Story Does NOT Include
- Changes to DockerClientTrait (already exists)
- Container run logic changes
- Integration tests
