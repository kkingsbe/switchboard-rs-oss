# Dev 2 TODO - Sprint 21

## Assigned Stories

### Story 4.5: Define message protocol types (2 pts)
**Status:** IN PROGRESS (WAITING FOR 4.1)
**File:** `stories/story-004-05-message-protocol-types.md`

**Tasks:**
- [ ] Define `GatewayMessage` enum with variants: Register, RegisterAck, Message, Heartbeat, HeartbeatAck
- [ ] Implement serde serialization/deserialization
- [ ] Document protocol in code comments
- [ ] Verify JSON round-trip tests pass

## Dependencies

- **Depends on:** Story 4.1 (gateway module structure) - must complete first
- Dev-1 is working on Story 4.1

## Notes

- Wait for Story 4.1 to complete before starting (module structure needed)
- Follow Rust best practices from `./skills/`
- Use serde for JSON serialization
- Reference `src/discord/tools/definitions.rs` for enum patterns
