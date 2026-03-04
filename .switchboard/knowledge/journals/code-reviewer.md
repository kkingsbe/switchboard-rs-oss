# Code Reviewer Journal

## 2026-03-03T16:50:00Z — Sprint 8 Reviews

### Review Session Notes

- **Reviewed story:** story-005-03 (Route Messages by Channel)
- **Status:** ❌ CHANGES_REQUESTED
- **Implementation quality:** Functionally complete but scope violation found

#### Issue Found

- **MUST FIX:** Out-of-scope file modification - `.switchboard/knowledge/journals/sprint-planner.md` was modified but is NOT in the story's "Files in Scope" list
- This is a scope violation - the story scope explicitly lists only `routing.rs`, `server.rs`, and `registry.rs`
- The sprint planner knowledge file is outside the story's scope

#### What Was Implemented Well

1. All acceptance criteria are met:
   - Channel ID extraction: `extract_channel_id()` function with 4 unit tests
   - Project lookup: Uses `registry.projects_for_channel()` - 4 tests
   - Message forwarding: Uses `ws_sender.send()` - verified with live receiver test

2. Follows project conventions:
   - Uses `thiserror` for error types
   - Uses `tracing` for logging
   - Comprehensive doc comments
   - Tests use descriptive names per project convention
   - No `unwrap()` in production code

3. Good test coverage: 8+ routing tests plus 4+ registry channel subscription tests

#### Build/Test Results

- Build: ✅ Passes with 1 warning (unrelated to this story - ratelimit.rs)
- Tests: 660 passed, 7 failed (all pre-existing failures in docker/ and ratelimit modules)

#### Pattern Observed

- Pre-existing test failures continue in docker/ and gateway ratelimit modules
- These are not related to new story implementations

---

## 2026-03-03 — Sprint 4 Reviews

### Review Session Notes

- **Reviewed story:** story-004-02 (Implement Gateway Configuration Loading)
- **Status:** ✅ APPROVED
- **Implementation quality:** Excellent - follows all Rust best practices

#### Common Patterns Observed

- The implementation uses `thiserror` for error handling as required by project conventions
- Tests are properly placed in the same file as the code (module tests)
- Doc comments are present on public API methods
- No `unwrap()` in production code (only in tests, which is allowed)

#### What Made This Implementation Good

1. Comprehensive unit tests covering:
   - Default config values
   - Config loading from TOML file
   - Environment variable expansion (both `${VAR}` and `${VAR:-default}` syntax)
   - Multiple channel mappings
   - Error handling for missing/invalid files

2. Follows project conventions:
   - Uses tracing for logging (not println!)
   - Uses serde for TOML deserialization
   - Uses thiserror for error types
   - Proper module organization

#### Pre-existing Issues Noted

- 17 docker::run::run tests are failing - unrelated to this story
- These failures existed before this story was implemented

#### Build/Test Results

- Build: ✅ Passes with `cargo build --features "discord gateway"`
- Clippy: ✅ No warnings with `cargo clippy -- -D warnings`
- Tests: 550 passed, 17 failed (pre-existing docker test failures)

### 2026-03-03T12:50:04Z — Sprint 7 Reviews

- Story `story-004-07` (Wire up Discord Gateway Connection) required CHANGES_REQUESTED
- Critical issue: Test compilation failures due to missing `discord_gateway` field in AppState - 8 test files affected
- Skill violations found: unused variable `_gateway_token`, use of `unwrap_or(0)` in production
- Acceptance criteria: 2/3 fully met, 1/3 partial (reconnection relies on twilight-gateway but no explicit loop)
- Pattern observed: New features that add fields to shared state (like AppState) need comprehensive test updates

### 2026-03-03 — Sprint 8 Reviews

- story-005-03 (Route Messages by Channel): Re-review requested out-of-scope changes be reverted. Developer did NOT revert the changes to `.switchboard/knowledge/journals/sprint-planner.md` - still 33 lines of Sprint 8 planning notes in the commit. Implementation is correct (all 3 acceptance criteria MET, tests pass) but scope violation persists.
- Common violation: Out-of-scope file modifications - this is the second review round for this issue
- The implementation itself is solid - routing logic, tests, error handling all correct

---

## 2026-03-04T03:29:00Z — Sprint 10 Reviews

### Review Session Notes

Reviewed 3 stories from Sprint 10:

#### story-004-08: CLI `gateway up` Command
- **Status:** ✅ APPROVED
- **Implementation quality:** Excellent
- All 4 acceptance criteria met:
  - CLI has `gateway` subcommand with `up` action ✅
  - Command starts gateway with config from `gateway.toml` ✅
  - Support `--config` flag for custom config path ✅
  - Support `--detach` flag (placeholder) ✅
- Build: ✅ PASSED
- Tests: 703 passed, 5 failed (pre-existing docker tests)
- Skills compliance: Uses thiserror, tracing, no unwrap() in production

#### story-006-01: Project Connection Management (RE-REVIEW - THIRD ROUND)
- **Status:** ❌ CHANGES_REQUESTED (THIRD ATTEMPT)
- **Critical Issue:** Scope violations STILL NOT FIXED after TWO previous reviews
- Files in scope: `src/gateway/connections.rs`, `src/gateway/mod.rs`
- Files modified in commit: 12 files (only 2 in scope)
- Must fix: Revert ALL out-of-scope changes before re-review

#### story-007-01: CLI `gateway status` Command
- **Status:** ❌ CHANGES_REQUESTED
- **Critical Issue:** Missing implementation - CLI does NOT call HTTP `/status` endpoint
- Acceptance criteria status:
  - Show gateway running/stopped: PARTIAL (only checks PID file)
  - Show Discord connection status: NOT IMPLEMENTED
  - Show connected projects/channels: NOT IMPLEMENTED
- Server has `/status` endpoint that returns all this data, but CLI doesn't call it
- Must fix: Add HTTP client call to GET `/status` and display results
