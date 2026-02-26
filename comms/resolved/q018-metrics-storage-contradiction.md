# Question: Metrics Storage Contradiction (JSON vs SQLite)

**Section:** PRD §11.2 vs ARCHITECT_DECISION_metrics_display.md
**Status:** OPEN
**Date:** 2026-02-14

## Issue

There is a critical contradiction in how metrics should be stored:

1. **PRD §11.2 (lines 325-352)** specifies JSON file storage:
   - "Metrics are persisted to a JSON file at `<log_dir>/metrics.json`"
   - Complete JSON schema is provided with examples

2. **ARCHITECT_DECISION_metrics_display.md (line 77)** states:
   - "Metrics are stored in SQLite as per PRD §11.1"

3. **PRD §11.1 (lines 301-315)** does NOT specify SQLite:
   - Only lists metrics to track
   - No storage format is mentioned in this section

## Context

**PRD §11.2 JSON Format Specification:**
```json
{
  "agents": {
    "code-reviewer": {
      "run_count": 42,
      "success_count": 40,
      "failure_count": 2,
      "total_runtime_seconds": 3780.5,
      "first_run_timestamp": "2026-02-11T08:00:00Z",
      "last_run_timestamp": "2026-02-13T02:00:00Z",
      "last_run_duration_seconds": 85.3,
      "average_run_duration_seconds": 90.0
    }
  }
}
```

## Questions

### Q1: Which storage format should be used?

**Options:**
- **A:** JSON file (as explicitly specified in PRD §11.2)
- **B:** SQLite database (as stated in architect decision)
- **C:** Both - support JSON for portability, SQLite for performance
- **D:** Architect decision was incorrect - revert to PRD specification

### Q2: If SQLite is chosen, how does this affect atomicity requirements?

**Options:**
- **A:** SQLite transactions provide atomicity (no extra mechanism needed)
- **B:** Still need atomic file writes for SQLite database file
- **C:** Both approaches are acceptable (JSON via temp+rename, SQLite via transactions)

### Q3: If JSON is chosen, how should concurrent access be handled?

**Options:**
- **A:** File locking (flock) to prevent concurrent writes
- **B:** Single scheduler instance per project (no concurrent access)
- **C:** Atomic write (temp file + rename) - may lose data on concurrent write

## Impact

Without resolution:
- **Implementation conflict:** Code may follow one spec while tests expect another
- **User experience:** Unclear what file format to expect in `<log_dir>/`
- **Testing confusion:** Test coverage may validate wrong format

## Recommendation

**A (JSON file)** - Follow PRD specification since:
- PRD is the source of truth for requirements
- JSON is simpler and doesn't require additional dependencies
- Already specified in PRD with complete schema
- SQLite adds complexity without clear benefit for v0.1 scale

If SQLite is preferred, PRD §11.2 should be updated to reflect this decision.
