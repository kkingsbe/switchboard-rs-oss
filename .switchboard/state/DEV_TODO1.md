# Sprint 2 - dev-1

## Assigned Stories (7 points total)

### Story 2.3: Clean Up Committed Artifacts
- **Points:** 2
- **Epic:** OSS Release Preparation
- **Status:** in-progress
- **Reference:** `.switchboard/planning/epics/epic-02-oss-release-prep.md`

### Story 3.2: Decompose src/config/mod.rs
- **Points:** 5
- **Epic:** Code Quality Improvements
- **Status:** in-progress
- **Reference:** `.switchboard/planning/epics/epic-03-code-quality.md`

## Acceptance Criteria (Story 2.3)
1. Following added to `.gitignore`:
   - `coverage/html/`
   - `comms/outbox/`
   - `logs/`
   - `plans/`
   - `skills-lock.json`
   - `BLOCKERS.md`
   - `FRONTEND_PRD.md`
2. Root-level test TOML files moved to `tests/fixtures/`:
   - `invalid.toml` → `tests/fixtures/`
   - `test-*.toml` → `tests/fixtures/`
3. `switchboard.toml` (author's working config) removed from root

## Acceptance Criteria (Story 3.2)
1. `src/config/mod.rs` (3,511 lines) decomposed into:
   - `src/config/types.rs` - Type definitions and structs
   - `src/config/parsing.rs` - TOML parsing logic
   - `src/config/validation.rs` - Configuration validation
   - `src/config/mod.rs` - Main entry point that re-exports
2. All tests pass after refactoring
3. Module exports properly organized
4. No functionality changes - pure refactoring
