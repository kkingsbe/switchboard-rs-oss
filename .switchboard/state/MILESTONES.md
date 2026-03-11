# Milestones

**Goals checksum:** 4ccc8d59696825c863f60e37950643c1915b8cc8819fbf0cd4cc40b508f2afc5
**Workspace type:** existing
**Derived:** 2026-03-10
**Last updated:** 2026-03-11

---

## Milestone 1: Event Core Infrastructure

**Status:** COMPLETE
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [x] Event struct and EventData enum defined with serde serialization
- [x] EventEmitter struct implemented with file writing capability
- [x] Unit tests for JSON serialization/deserialization pass
- [x] Event schema validation works correctly

**Decomposition history:**
<!-- Empty initially -->

---

## Milestone 2: Scheduler Events Integration

**Status:** COMPLETE
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [x] scheduler.started event emitted on switchboard up
- [x] scheduler.stopped event emitted on graceful shutdown
- [x] Uptime calculation tracked correctly
- [x] Integration tests for scheduler lifecycle events pass

**Decomposition history:**
<!-- Empty initially -->

---

## Milestone 3: Container Events Integration

**Status:** PENDING
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [ ] container.started event emitted when launching containers
- [ ] container.exited event emitted on container completion
- [ ] Exit code, duration_seconds, timeout_hit captured
- [ ] container.skipped and container.queued events implemented
- [ ] Integration tests for container lifecycle pass

**Decomposition history:**
<!-- Empty initially -->

---

## Milestone 4: Git Diff Capture

**Status:** PENDING
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [ ] HEAD hash recorded before container launch
- [ ] HEAD hash captured after container exits
- [ ] git.log output parsed into structured commit data
- [ ] Edge case handled: no commits made
- [ ] Unit tests for git diff parsing pass

**Decomposition history:**
<!-- Empty initially -->

---

## Milestone 5: Log Rotation

**Status:** PENDING
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [ ] 10MB size check implemented after each write
- [ ] Log rotation with timestamp suffix works
- [ ] 30-day retention cleanup implemented
- [ ] Tests for rotation logic pass

**Decomposition history:**
<!-- Empty initially -->

---

## Milestone 6: Configuration Integration

**Status:** PENDING
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [ ] [settings.observability] TOML parsing works
- [ ] Wired into main app initialization
- [ ] Config loading tests pass

**Decomposition history:**
<!-- Empty initially -->

---

## Milestone 7: Derived Metrics (Consumer Layer)

**Status:** PENDING
**Task type:** code
**Goal reference:** Implement ./observability_design_spec.md using STRICT test-driven development
**Success criteria:**
- [ ] Throughput metrics computed: agent runs, productive runs, commits, lines inserted/deleted
- [ ] Reliability metrics computed: failure rate, timeouts, skipped runs, empty runs
- [ ] Per-agent breakdown grouping works
- [ ] Metrics computation tests pass

**Decomposition history:**
<!-- Empty initially -->

---
