# Story 5.5: Add configuration validation

## User Story

**As a** user,
**I want** to get clear error messages when my config is invalid,
**So that** I can fix configuration issues quickly.

## Acceptance Criteria

1. Validate discord_token is not empty
   - Verification: Error if token missing
   
2. Validate http_port and ws_port are valid (1024-65535)
   - Verification: Error if port out of range
   
3. Validate channel mappings have required fields
   - Verification: Error if channel mapping incomplete

## Technical Context

### Architecture
- Validation happens during config loading
- Use thiserror for structured validation errors
- Clear, user-friendly error messages

### Project Conventions
- Follow patterns from `src/config/mod.rs`
- Use serde's validation attributes where possible
- Return descriptive errors that help users fix issues

### Existing Code to Reference
- `src/config/mod.rs` - config loading patterns
- `src/gateway/config.rs` - will be created in story 4.2
- `gateway.toml` - sample config file

## Implementation Plan

1. Add validation to `src/gateway/config.rs`:
   - Create `GatewayConfig::validate()` method
   - Validate token is present and non-empty
   - Validate port ranges (1024-65535)
   - Validate channel mappings have required fields

2. Use thiserror for validation errors:
   - Define `ConfigValidationError` enum
   - Map each validation failure to specific error variant

3. Integrate validation:
   - Call validate() after loading config
   - Return errors with clear messages

4. Add unit tests:
   - Test empty token returns error
   - Test invalid port ranges return errors
   - Test incomplete channel mappings return errors

5. Verify compilation and tests pass

## Skills

- [Rust Best Practices](../../skills/rust-best-practices/SKILL.md)
- [Rust Engineer](../../skills/rust-engineer/SKILL.md)

## Dependencies

- Depends on: Story 5.2 (channel mapping in config)

## Scope Boundaries

**In Scope:**
- Token validation
- Port range validation  
- Channel mapping validation
- Unit tests for validation

**Out of Scope:**
- Network-level validation (ports in use)
- Discord token format validation
- Runtime validation (channel existence)

## Risk Assessment

- **Risk Level:** Low
- **Rationale:** Pure validation logic, no I/O
