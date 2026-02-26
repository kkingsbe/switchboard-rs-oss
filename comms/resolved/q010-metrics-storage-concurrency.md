# Resolution: Metrics Storage Concurrency and Atomicity

**Original Question:** comms/outbox/q010-metrics-storage-concurrency.md
**Resolution Date:** 2026-02-14T15:25:00.000Z
**Status:** ✅ RESOLVED - Architect Decision Created

---

## Resolution Summary

Implementation details resolved via architect decision: **ARCHITECT_DECISION_metrics_concurrency.md**

### Decisions Made:

1. **Atomic Writes:** Write to temp file, then atomic rename (interrupt-safe)
   - Pattern: `metrics.json.tmp` → `metrics.json`
   - Uses `std::fs::rename()` which is atomic on POSIX systems
   - Prevents corruption on power failure or crash

2. **File Locking:** Optional file lock for concurrent safety (if multiple instances allowed)
   - Use `flock` on Unix platforms
   - Fail gracefully if lock acquisition times out
   - Log warning if concurrent write is detected

3. **Concurrency Model:** One scheduler per project (Option A)
   - Only one `switchboard up` process allowed per project directory
   - Metrics file is per-project in `<log_dir>/metrics.json`
   - PID file in `.switchboard/scheduler.pid` to prevent multiple instances
   - Fail with clear error if user tries to run second instance

4. **Corruption Recovery:** Option A (backup + fresh start)
   - On startup, validate JSON structure
   - If corrupted: Backup to `metrics.json.backup.<timestamp>`, start fresh
   - Log warning with backup location
   - User can restore from backup if needed

5. **Write Frequency:** Option A (after every agent run)
   - Write metrics immediately after each agent run completes
   - Ensures data is persisted even if scheduler crashes

---

## Implementation Status

| Component | Status | Notes |
|------------|--------|--------|
| Metrics data structures | ✅ Implemented | src/metrics/mod.rs |
| Metrics storage (atomic writes) | ✅ Implemented | src/metrics/store.rs |
| Corruption recovery | ✅ Implemented | src/metrics/store.rs (load()) |
| File locking | ❌ Not implemented | Optional enhancement, not required for v0.1 |
| Single instance enforcement | ❌ Not implemented | PID file tracking needed |

---

## Related Files

- [`ARCHITECT_DECISION_metrics_concurrency.md`](ARCHITECT_DECISION_metrics_concurrency.md) - Full decision document
- [`src/metrics/store.rs`](src/metrics/store.rs) - Implementation
- [`PRD.md`](PRD.md) §11.4 - Metrics Storage and Integrity
