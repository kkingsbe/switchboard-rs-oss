# Dev 1 TODO - Sprint 21

## Assigned Stories

### Story 4.1: Create gateway module structure (1 pt)
**Status:** IN PROGRESS
**File:** `stories/story-004-01-gateway-module-structure.md`

**Tasks:**
- [ ] Create `src/gateway/mod.rs` with module declarations
- [ ] Add `pub mod gateway` to `src/lib.rs`
- [ ] Add feature flag `gateway` to Cargo.toml
- [ ] Verify `cargo build --features gateway` compiles

### Story 5.5: Add configuration validation (1 pt)
**Status:** IN PROGRESS
**File:** `stories/story-005-05-config-validation.md`

**Tasks:**
- [ ] Validate discord_token is not empty
- [ ] Validate http_port and ws_port are valid (1024-65535)
- [ ] Validate channel mappings have required fields
- [ ] Add unit tests for validation

## Dependencies

- Story 4.5 (dev-2) depends on Story 4.1 completion

## Notes

- Follow Rust best practices from `./skills/`
- Use thiserror for validation errors
- Use tracing for logging (no println!)
