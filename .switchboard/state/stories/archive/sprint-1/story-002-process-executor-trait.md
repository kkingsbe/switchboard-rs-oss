# Story 002: Define ProcessExecutorTrait

> Epic: epic-testability — Testability Enhancement
> Points: 2
> Sprint: 1
> Type: infrastructure
> Risk: low
> Created: 2026-03-02

## User Story

As a developer,
I want to mock external process execution,
So that I can test code that spawns subprocesses without running them.

## Acceptance Criteria

1. Define `ProcessExecutorTrait` with methods:
   - `spawn_child(cmd: Command) -> Result<Child, ProcessError>`
   - `wait_child(child: &mut Child) -> Result<ExitStatus, ProcessError>`
   - `kill_child(child: &mut Child) -> Result<(), ProcessError>`

2. Create `RealProcessExecutor` implementing production behavior
3. Create `MockProcessExecutor` for testing

4. **Test:** Verify mock can simulate successful exit
   - Run: `cargo test process_executor`

5. **Test:** Verify mock can simulate error conditions
   - Run: `cargo test process_error`

## Technical Context

### Architecture Reference
From architecture.md - External process execution needed for:
- Running `docker` CLI commands
- Running `npx` for skills

### Project Conventions
- Use thiserror for ProcessError
- No unwrap() in production code

### Existing Code Context
Check `src/cli/process.rs` - may have existing process handling

```
src/cli/
├── process.rs   # Check for existing patterns
└── mod.rs
```

## Implementation Plan

1. Create `src/traits/process.rs` or extend existing trait file
2. Define `ProcessExecutorTrait`
3. Create `RealProcessExecutor` and `MockProcessExecutor`
4. Write unit tests
5. Run build + tests

### Skills to Read
- `./skills/rust-engineer/SKILL.md`
- `./skills/rust-engineer/references/traits.md`
- `./skills/rust-engineer/references/testing.md`

### Dependencies
Story 001 (DockerConnectionTrait) - can be done in parallel

## Scope Boundaries

### This Story Includes
- ProcessExecutorTrait definition
- Mock implementation

### This Story Does NOT Include
- Refactoring existing code to use the trait
- Integration tests

### Files in Scope
- `src/traits/process.rs` — create (or modify existing)
- Tests in `src/traits/`
