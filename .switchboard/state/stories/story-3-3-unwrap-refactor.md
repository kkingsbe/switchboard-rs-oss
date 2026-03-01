# Story 3.3: Replace .unwrap() Calls with Proper Error Handling

> Epic: epic-03 — Code Quality Improvements
> Points: 5
> Sprint: 3
> Type: refactor
> Risk: medium
> Created: 2026-03-01T21:22:00Z

## User Story

As a developer, I want to replace unsafe .unwrap() calls with proper error handling so that the codebase is more robust and follows Rust best practices.

## Acceptance Criteria

1. All .unwrap() calls in the codebase are identified and reviewed
   - **Test:** Run `grep -r "\.unwrap()" src/ --include="*.rs" | wc -l` before and after
2. Production code .unwrap() calls are replaced with proper error handling
   - **Test:** Build passes with no warnings about unwrap in non-test code
3. Tests may retain .unwrap() for test-specific scenarios
   - **Test:** cargo test passes
4. Error handling follows thiserror pattern
   - **Test:** Code review shows proper Result types

## Technical Context

### Architecture Reference
This story is part of Epic 3 (Code Quality Improvements). The focus is on improving error handling across all modules.

### Skills to Read
- `./skills/rust-best-practices/SKILL.md` — Best practices reference
- `./skills/rust-best-practices/references/chapter_04.md` — Error handling (thiserror vs anyhow)
- `./skills/rust-engineer/references/error-handling.md` — Error handling patterns

### Dependencies
- None - this story is independent

## Implementation Plan

1. Search for all .unwrap() calls in the codebase
2. Categorize them: test code vs production code
3. For production code .unwrap() calls:
   - Determine appropriate error type (thiserror for libraries)
   - Replace with ? operator or match statements
   - Add custom error types if needed
4. For test code .unwrap(), add comments explaining why they're acceptable
5. Run cargo build to verify changes
6. Run cargo test to ensure no regressions
7. Run cargo clippy to check for any missed unwrap patterns

### Skills to Read
- `./skills/rust-best-practices/SKILL.md` — Rust best practices
- `./skills/rust-best-practices/references/chapter_04.md` — Error handling guidelines

## Scope Boundaries

### This Story Includes
- Finding all .unwrap() calls in src/
- Replacing production .unwrap() with proper error handling
- Adding thiserror types where needed

### This Story Does NOT Include
- Refactoring module structure (that's stories 3.1, 3.2)
- Adding new functionality
- Modifying test files beyond commenting

### Files in Scope
- All .rs files in src/ that contain .unwrap()

### Files NOT in Scope
- Test files in tests/
- Documentation files
- CI configuration
