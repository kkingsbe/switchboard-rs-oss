# Story TEST-FIX-01: Fix Pre-existing Test Failures

> Epic: test-infrastructure
> Points: 3
> Sprint: 2 (fix)
> Type: test
> Risk: medium
> Created: 2026-03-01T21:10:00Z

## User Story
As a developer, I need the test suite to pass so that refactoring work can proceed safely.

## Acceptance Criteria

1. All pre-existing test failures are fixed
   - **Test:** `cargo test` passes with 0 failures
2. No new test failures are introduced
   - **Test:** Run tests before and after, count failures

## Technical Context

### The Blocker
The BLOCKERS.md file documents 24 pre-existing test failures that block all refactoring work in Sprint 2.

### Test Files to Investigate
- Look at the test output to identify failing test files

## Implementation Plan

1. Run `cargo test 2>&1 | head -100` to see failure details
2. Categorize failures (compilation errors vs runtime failures)
3. Fix each category appropriately
4. Run tests to verify fixes
5. If tests pass, create `.switchboard/state/.tests_fixed` marker

### Skills to Read
- `./skills/rust-engineer/references/testing.md` — Testing patterns
- `./skills/rust-best-practices/references/chapter_04.md` — Error handling

### Dependencies
None - this is a prerequisite story

## Scope Boundaries

### This Story Includes
- Fixing pre-existing test failures
- Ensuring test suite passes

### This Story Does NOT Include
- Refactoring of source code (that's stories 3.1, 3.2)
- New feature development
- CI/CD changes

### Files in Scope
- Test files in `src/` and `tests/`

### Files NOT in Scope
- Source code in `src/` (except test files)
- CI configuration
