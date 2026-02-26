# Resolution: Timeout Termination Behavior

**Original Question:** comms/outbox/q012-timeout-termination-behavior.md
**Resolution Date:** 2026-02-14T15:26:00.000Z
**Status:** ✅ RESOLVED - Architect Decision Created

---

## Resolution Summary

Implementation details resolved via architect decision: **ARCHITECT_DECISION_timeout_termination.md**

### Decisions Made:

1. **Termination Signal:** SIGTERM with 10-second grace period, then SIGKILL
   - Initial signal: SIGTERM for graceful shutdown
   - Grace period: 10 seconds (Docker's default for `docker stop`)
   - Final signal: SIGKILL if container still running
   - Exit code: 137 (SIGKILL) for timed-out runs

2. **Log Preservation:** Save all logs captured up to timeout point
   - Logs are streamed in real-time during container execution
   - All logs captured before timeout are saved to agent log file
   - No special timeout log file - timeout noted in scheduler log

3. **End Time for Metrics:** Record both timestamps
   - `timeout_timestamp`: When timeout fired
   - `exit_timestamp`: When container actually stopped
   - Use `exit_timestamp` for duration calculations
   - Store both in metrics for analysis

4. **File System State:** Warning for read-write agents
   - Read-only agents: No issue (can't modify filesystem)
   - Read-write agents: Log warning about potential partial writes
   - Do not attempt to rollback or cleanup partial writes

---

## Implementation Tasks

These decisions are specified in Sprint 4 tasks:
- [`TODO2.md`](../TODO2.md) - Container timeout enforcement
- Implementation location: `src/docker/run/wait/timeout.rs`

---

## Related Files

- [`ARCHITECT_DECISION_timeout_termination.md`](ARCHITECT_DECISION_timeout_termination.md) - Full decision document
- [`PRD.md`](PRD.md) §9 - Error Handling & Edge Cases
