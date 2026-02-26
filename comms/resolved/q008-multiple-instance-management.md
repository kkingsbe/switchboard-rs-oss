# Resolution: Multiple Instance Management and Daemon Mode

**Original Question:** comms/outbox/q008-multiple-instance-management.md
**Resolution Date:** 2026-02-14T15:28:00.000Z
**Status:** ✅ RESOLVED - Architect Decision Created

---

## Resolution Summary

Implementation details resolved via architect decision: **ARCHITECT_DECISION_multiple_instance_management.md**

### Decisions Made:

1. **Instance Identification:** Option B - PID file in `.switchboard/switchboard.pid`
   - Store PID file when running `switchboard up --detach`
   - `switchboard down` reads PID and stops that specific process
   - PID file contains process ID and config path for verification

2. **Multiple Instances Support:** Option B - Multiple instances from different directories
   - Each project directory can have its own scheduler instance
   - Each instance uses its own `.switchboard/` directory
   - Multiple instances from different directories allowed
   - Multiple instances from same directory NOT allowed (PID file check)

3. **Monitoring Detached Instances:** Option C - Rely on `switchboard logs` and log files
   - No separate `switchboard status` command for v0.1
   - Users view logs via `switchboard logs [<agent-name>] [--follow] [--tail <n>]`
   - Log files in `.switchboard/logs/` directory

4. **Config Path with Detach:** Option A - PID file stores config path used
   - PID file format contains both PID and config path
   - `switchboard down` verifies config path matches current directory
   - Prevents accidentally stopping wrong instance

---

## Implementation Tasks

These decisions will be implemented as part of Sprint 4 tasks in:
- [`TODO2.md`](../TODO2.md) - Error handling (daemon management)
- [`TODO4.md`](../TODO4.md) - Drift cleanup (PID file implementation)

---

## Related Files

- [`ARCHITECT_DECISION_multiple_instance_management.md`](ARCHITECT_DECISION_multiple_instance_management.md) - Full decision document
- [`PRD.md`](PRD.md) §5.1 - CLI Commands
