# Sprint 2 - dev-2

## Assigned Stories (6 points total)

### Story 3.1: Decompose src/docker/run/run.rs
- **Points:** 5
- **Epic:** Code Quality Improvements
- **Status:** in-progress
- **Reference:** `.switchboard/planning/epics/epic-03-code-quality.md`

### Story 3.4: Clean Up Empty Feature Flags
- **Points:** 1
- **Epic:** Code Quality Improvements
- **Status:** in-progress
- **Reference:** `.switchboard/planning/epics/epic-03-code-quality.md`

## Acceptance Criteria (Story 3.1)
1. `src/docker/run/run.rs` (5,115 lines) decomposed into:
   - `src/docker/run/create.rs` - Container creation logic
   - `src/docker/run/execute.rs` - Container execution logic
   - `src/docker/run/lifecycle.rs` - Container lifecycle management
   - `src/docker/run/run.rs` - Main entry point that coordinates sub-modules
2. All existing functionality preserved (all tests pass)
3. Module exports properly organized in `src/docker/run/mod.rs`
4. No functionality changes - pure refactoring

## Acceptance Criteria (Story 3.4)
1. Cargo.toml feature flags reviewed:
   - `integration` - Either wire to actual conditional compilation OR remove
   - `scheduler` - Either wire to actual conditional compilation OR remove
   - `streams` - Either wire to actual conditional compilation OR remove
2. Only `discord` feature remains (confirmed real)
3. If features are kept, they must have actual conditional compilation
4. If features are removed, Cargo.toml updated accordingly
