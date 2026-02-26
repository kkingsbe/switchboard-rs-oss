# ARCHITECT GAP ANALYSIS — Skills Feature
> Generated: 2026-02-20T10:47:00Z
> Sprint: 3
> Feature: Skills Management CLI
> Feature Doc: addtl-features/skills-feature.md

---

## Executive Summary

The skills feature is approximately **75% complete** (10/12 Acceptance Criteria). Sprint 3 is in progress with 3 of 4 agents complete. The backlog is well-structured and covers most requirements. A few minor gaps and vague items have been identified and should be addressed.

---

## 1. Feature Requirements Analysis

### 1.1 Acceptance Criteria Status

| AC ID | Description | Status | Implementation Location |
|-------|-------------|--------|------------------------|
| AC-01 | `switchboard skills list` invokes `npx skills find` | ✅ COMPLETE | src/commands/skills.rs (Sprint 1, Agent 3) |
| AC-02 | `switchboard skills list --search <query>` invokes `npx skills find <query>` | ✅ COMPLETE | src/commands/skills.rs (Sprint 1, Agent 3) |
| AC-03 | `switchboard skills install <source>` invokes `npx skills add <source> -a kilo -y` | ✅ COMPLETE | src/commands/skills.rs (Sprint 1, Agent 3) |
| AC-04 | `switchboard skills installed` lists installed skills | ✅ COMPLETE | src/commands/skills.rs (Sprint 2, Agent 2) |
| AC-05 | `switchboard skills remove <name>` removes an installed skill | ✅ COMPLETE | src/commands/skills.rs (Sprint 2, Agent 3) |
| AC-06 | `switchboard skills update` invokes `npx skills update` | ✅ COMPLETE | src/commands/skills.rs (Sprint 2, Agent 4) |
| AC-07 | Per-agent `skills = [...]` in `[[agent]]` declares skills | ✅ COMPLETE | src/config/mod.rs (Sprint 1, Agent 4) |
| AC-08 | Skills are installed inside the container at startup | 🔄 IN PROGRESS | src/docker/skills.rs, src/docker/run/mod.rs (Sprint 3, Agents 1-2) |
| AC-09 | Failed skill install aborts run, surfaced in logs and metrics | 🔄 IN PROGRESS | src/docker/run/mod.rs, src/commands/metrics.rs (Sprint 3, Agent 3) |
| AC-10 | `switchboard validate` checks skill references | ✅ COMPLETE | src/commands/validate.rs (Sprint 3, Agent 4) |
| AC-11 | Commands requiring `npx` fail fast with prerequisite error | ✅ COMPLETE | src/skills/error.rs (Sprint 1, Agent 2) |
| AC-12 | Exit codes from all `npx skills` invocations are forwarded | ✅ COMPLETE | src/commands/skills.rs (Sprint 1-2, Agents 3-4) |

**Summary:** 10/12 ACs complete (83%)
- Complete: AC-01, AC-02, AC-03, AC-04, AC-05, AC-06, AC-07, AC-10, AC-11, AC-12
- In Progress: AC-08, AC-09

---

## 2. Gap Analysis

### 2.1 Requirements NOT Covered by Backlog

**Status:** ✅ All requirements are covered by planned work

All 12 acceptance criteria and their sub-requirements are covered by tasks in the backlog or have been completed. No gaps identified in core feature requirements.

### 2.2 Vague Backlog Items Requiring Breakdown

The following items in the backlog should be broken down into more atomic, implementable tasks:

#### 2.2.1 Documentation Tasks (Sprint 4 - Lines 207-221 of backlog)

**Current Items:**
```
- [ ] Update `README.md` with skills feature overview
- [ ] Add `skills` subcommand section to CLI documentation
- [ ] Document `switchboard skills list --help` output
- [ ] Document `switchboard skills install --help` output
- [ ] Document `switchboard skills installed --help` output
- [ ] Document `switchboard skills remove --help` output
- [ ] Document `switchboard skills update --help` output
- [ ] Add example `switchboard.toml` with per-agent skill declarations
- [ ] Document the `skills` field in `[[agent]]` configuration reference
- [ ] Document skill source formats (owner/repo, owner/repo@skill-name, URLs)
- [ ] Document behavior when npx is not available
- [ ] Document container skill installation behavior
- [ ] Document skill installation failure handling in logs
- [ ] Add troubleshooting section for common skill-related issues
```

**Recommended Breakdown:**
```
### README.md Updates
- [ ] Add Skills feature section to README.md overview
- [ ] Document `switchboard skills` command family with examples
- [ ] Add quick start guide for installing first skill
- [ ] Link to detailed documentation sections

### CLI Documentation
- [ ] Create docs/skills_commands.md with command reference
- [ ] Document each command: list, install, installed, remove, update
- [ ] Include usage examples and common workflows
- [ ] Document flags and options for each command

### Configuration Documentation
- [ ] Update docs/configuration.md with skills field documentation
- [ ] Document per-agent skill declaration syntax
- [ ] Add example switchboard.toml with skills configuration
- [ ] Document skill source formats and validation rules

### Runtime Behavior Documentation
- [ ] Document container skill installation lifecycle
- [ ] Document skill installation failure handling
- [ ] Document npx availability requirements
- [ ] Document skill directory locations (.kilocode/skills/)

### Troubleshooting Documentation
- [ ] Create docs/skills_troubleshooting.md
- [ ] Document common error messages and solutions
- [ ] Document network connectivity issues
- [ ] Document skill version conflicts
```

#### 2.2.2 Open Questions Documentation Tasks (Sprint 4 - Lines 223-241 of backlog)

**Current Items:**
```
- [ ] Document decision on skill install latency and agent timeouts (OQ-1)
- [ ] Document current approach: users must account for skill install time in timeout values manually
- [ ] Create GitHub issue or RFC for skill install latency auto-adjustment feature (OQ-1)
- [ ] Document decision on skill version pinning support (OQ-2)
- [ ] Note that skill version pinning is deferred to future iteration (OQ-2)
- [ ] Create GitHub issue for tracking skill version pinning requirements (OQ-2)
- [ ] Document decision on skill caching across runs (OQ-3)
- [ ] Note that skill caching is deferred to future iteration (OQ-3)
- [ ] Document rationale for fresh install each run (ensures up-to-date skills) (OQ-3)
- [ ] Create GitHub issue for skill caching feature request (OQ-3)
- [ ] Document decision on `npx skills` version pinning (OQ-4)
- [ ] Note that using latest `npx skills` version is current approach (OQ-4)
- [ ] Document trade-off: automatic improvements vs potential breaking changes (OQ-4)
- [ ] Create GitHub issue for npx skills version pinning discussion (OQ-4)
- [ ] Document decision on skill install failure policy (OQ-5)
- [ ] Document current behavior: failed install always aborts agent run (OQ-5)
- [ ] Note that `skills_optional` flag is deferred to future iteration (OQ-5)
- [ ] Create GitHub issue for optional skills feature request (OQ-5)
```

**Recommended Breakdown:**
```
### Documentation: Design Decisions
- [ ] Create docs/skills_decisions.md documenting all architectural decisions
- [ ] Document OQ-1: Skill install latency handling (manual timeout adjustment)
- [ ] Document OQ-2: Skill version pinning (deferred to future)
- [ ] Document OQ-3: Skill caching strategy (no caching, fresh install each run)
- [ ] Document OQ-4: npx skills version policy (use latest version)
- [ ] Document OQ-5: Failure policy (hard abort on failure)

### GitHub Issue Creation
- [ ] Create GitHub issue for skill install latency auto-adjustment (OQ-1)
- [ ] Create GitHub issue for skill version pinning (OQ-2)
- [ ] Create GitHub issue for skill caching across runs (OQ-3)
- [ ] Create GitHub issue for npx skills version pinning (OQ-4)
- [ ] Create GitHub issue for optional skills flag (OQ-5)
```

#### 2.2.3 Testing Tasks (Sprint 3-4 - Lines 175-205 of backlog)

**Current Items:**
```
- [ ] Add unit tests for entrypoint script generation
- [ ] Add integration test for npx not found error message
- [ ] Add integration test for invalid skill source format in config
- [ ] Add integration test for duplicate skill detection in config
- [ ] Add integration test for container skill installation
- [ ] Add integration test for container skill installation failure handling
- [ ] Add test for skill installation failure in container (abort with non-zero exit)
```

**Recommended Breakdown:**
```
### Unit Tests: Entrypoint Script Generation
- [ ] Add test for script generation with multiple skills
- [ ] Add test for script generation with single skill
- [ ] Add test for script generation with empty skills list
- [ ] Add test for script structure (shebang, set -e, exec)
- [ ] Add test for skill command formatting (owner/repo vs owner/repo@skill-name)

### Integration Tests: Config Validation
- [ ] Add integration test for `switchboard validate` with npx not found
- [ ] Add integration test for `switchboard validate` with invalid skill source format
- [ ] Add integration test for `switchboard validate` with duplicate skills
- [ ] Add integration test for `switchboard validate` with empty skills field
- [ ] Add integration test for `switchboard validate` with multiple agents

### Integration Tests: Container Integration
- [ ] Add integration test for successful skill installation in container
- [ ] Add integration test for skill installation failure in container
- [ ] Add integration test for partial skill installation (mixed success/failure)
- [ ] Add integration test for skill installation failure logging
- [ ] Add integration test for skill installation failure metrics tracking
```

#### 2.2.4 Performance and Reliability Tasks (Sprint 4 - Lines 253-259 of backlog)

**Current Items:**
```
- [ ] Add performance test for `switchboard skills list` (should return within 3 seconds)
- [ ] Add performance test for single skill installation in container (should complete within 15 seconds)
- [ ] Ensure skill installation time is reflected in `switchboard metrics`
- [ ] Test graceful degradation when network is unavailable
- [ ] Verify distinct log prefixes for skill install failures vs agent execution failures
```

**Recommended Breakdown:**
```
### Performance Testing
- [ ] Add benchmark test for `switchboard skills list` command (3 second SLA)
- [ ] Add benchmark test for container skill installation (15 second SLA per skill)
- [ ] Add performance regression test for skills list command
- [ ] Document performance baselines in test suite

### Metrics Integration
- [ ] Verify skills_install_time_seconds is populated in metrics
- [ ] Verify skills_installed count is accurate in metrics
- [ ] Verify skills_failed count is accurate in metrics
- [ ] Add integration test for metrics output with skills data

### Reliability Testing
- [ ] Add integration test for network unavailability during skills list
- [ ] Add integration test for network unavailability during skills install
- [ ] Add integration test for graceful error messages when offline
- [ ] Verify log prefix distinction in logs output
```

#### 2.2.5 Backwards Compatibility Tasks (Sprint 4 - Lines 260-263 of backlog)

**Current Items:**
```
- [ ] Ensure existing projects without skills field continue to work
- [ ] Ensure manually managed skills in `.kilocode/skills/` still work
- [ ] Add integration test for backwards compatibility with existing configs
```

**Recommended Breakdown:**
```
### Backwards Compatibility Tests
- [ ] Add integration test for existing config without skills field
- [ ] Add integration test for manually managed skills in .kilocode/skills/
- [ ] Add integration test for mixed config (some agents with skills, some without)
- [ ] Add integration test for existing agents container execution (no skills field)
- [ ] Verify `switchboard skills installed` correctly lists manually managed skills
```

### 2.3 Already Implemented Requirements

The following requirements from the feature document are already complete:

✅ **CLI Commands** (AC-01 through AC-06)
- All five CLI commands implemented and tested
- All commands forward npx exit codes correctly
- All commands check for npx availability before execution

✅ **Config Schema** (AC-07)
- Per-agent skills field added to AgentConfig
- Skill source format validation implemented
- Integration with config parser complete

✅ **Validation** (AC-10)
- Empty skills field warning implemented
- Invalid skill source format validation implemented
- Duplicate skill detection implemented

✅ **npx Handling** (AC-11, AC-12)
- npx availability check implemented with clear error messages
- Exit code forwarding implemented for all npx commands

---

## 3. Sprint 3 Status

### 3.1 Agent Completion Status

| Agent | TODO Status | .agent_done | Tasks Complete | Tasks Remaining | Blockers |
|-------|-------------|-------------|----------------|----------------|----------|
| 1 | ✅ Complete | ✅ Exists | 10/10 | 0 | None |
| 2 | ✅ Complete | ✅ Exists | 9/9 | 0 | None |
| 3 | 🔄 In Progress | ❌ Missing | ~3/28 | ~25 | None |
| 4 | ✅ Complete | ✅ Exists | 10/10 | 0 | None |

### 3.2 Sprint 3 Work Remaining

**Agent 3 Tasks (TODO3.md):**
- Task 4: Metrics Integration with switchboard metrics Command (1 task complete, 6 remaining)
- Task 5: Error Handling and Reporting (0 tasks complete, 5 remaining)
- Task 6: Unit Tests (0 tasks complete, 5 remaining)
- Task 7: Integration Tests (0 tasks complete, 3 remaining)
- Task 8: Documentation (0 tasks complete, 3 remaining)
- Task 9: Code Quality (0 tasks complete, 5 remaining)

**Note:** Tasks 1-3 are complete as indicated by checkmarks in TODO3.md (lines 12-40)

### 3.3 Sprint 3 Blockers

**Status:** No active blockers for Sprint 3

All dependencies resolved:
- Agent 2 completed container script injection (`.agent_done_2` exists)
- Agent 3 is unblocked and can proceed with remaining tasks

---

## 4. Recommendations

### 4.1 Immediate Actions

1. **Monitor Agent 3 Progress** (Priority: HIGH)
   - Agent 3 is the only remaining agent working on Sprint 3
   - Approximately 25 tasks remain in TODO3.md
   - No blockers - Agent 3 should be able to complete

2. **Verify TODO3.md AGENT QA Section** (Priority: MEDIUM)
   - The AGENT QA section (lines 156-170) needs to ensure proper completion signaling
   - Once Agent 3 completes, `.agent_done_3` should be created
   - Once all `.agent_done_*` files exist, `.sprint_complete` should be created

3. **Plan Sprint 4** (Priority: MEDIUM)
   - Sprint 4 tasks should be broken down into more atomic units as outlined above
   - Focus on Documentation, Testing, Performance, and Backwards Compatibility

### 4.2 Backlog Refactoring Recommendations

1. **Break Down Vague Documentation Tasks** (Lines 207-221, 223-241)
   - As detailed in section 2.2.1 and 2.2.2 above
   - Create atomic, implementable tasks for each deliverable

2. **Break Down Vague Testing Tasks** (Lines 175-205)
   - As detailed in section 2.2.3 and 2.2.4 above
   - Ensure each test is a specific, atomic task

3. **Clarify Performance Testing Requirements** (Lines 253-259)
   - Define specific performance baselines and SLA thresholds
   - Document how to measure and verify performance metrics

### 4.3 Open Questions Resolution

The feature document identifies 5 open questions (section 9). Current documented decisions:

| OQ | Topic | Decision | Documentation Status |
|----|-------|----------|---------------------|
| OQ-1 | Skill install latency and agent timeouts | Manual timeout adjustment (user responsibility) | ⚠️ Not yet documented |
| OQ-2 | Skill version pinning | Deferred to future iteration | ⚠️ Not yet documented |
| OQ-3 | Skill caching across runs | No caching (fresh install each run) | ⚠️ Not yet documented |
| OQ-4 | npx skills version pinning | Use latest version | ⚠️ Not yet documented |
| OQ-5 | Skill install failure policy | Hard abort on failure | ⚠️ Not yet documented |

**Recommendation:** These decisions should be documented in Sprint 4 as outlined in section 2.2.2.

---

## 5. Feature Completion Assessment

### 5.1 Completion Metrics

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Acceptance Criteria Complete | 10/12 | 12/12 | 🔄 83% |
| Sprints Complete | 2/4 | 4/4 | 🔄 50% |
| Core Functionality | ✅ Complete | N/A | ✅ |
| Container Integration | 🔄 75% | 100% | 🔄 |
| Documentation | ⏸️ Not Started | 100% | ⏸️ |
| Testing Coverage | 🔄 ~80% | >80% | 🔄 |
| Performance Testing | ⏸️ Not Started | 100% | ⏸️ |

### 5.2 Estimated Work Remaining

| Sprint | Tasks | Estimated Effort |
|--------|-------|------------------|
| Sprint 3 (remaining) | ~25 tasks (Agent 3 only) | ~1 week |
| Sprint 4 (documentation) | ~25-30 tasks | ~2-3 weeks |
| Sprint 4 (testing) | ~20-25 tasks | ~1-2 weeks |
| Sprint 4 (performance) | ~10-15 tasks | ~1 week |
| **Total** | **~80-95 tasks** | **~5-7 weeks** |

### 5.3 Completion Roadmap

**Current Phase:** Sprint 3 Container Integration (AC-08, AC-09)

**Next Phases:**
1. **Sprint 3 Completion** (~1 week)
   - Agent 3 completes failure detection and error recovery
   - All Sprint 3 QA passes
   - Sprint 3 declared complete

2. **Sprint 4: Documentation** (~2-3 weeks)
   - README updates
   - CLI documentation
   - Configuration documentation
   - Troubleshooting guide
   - Design decisions documentation

3. **Sprint 4: Testing** (~1-2 weeks)
   - Unit tests for remaining code paths
   - Integration tests for container integration
   - Backwards compatibility tests
   - Performance benchmarks

4. **Sprint 4: Final Polish** (~1 week)
   - Code quality (clippy, fmt)
   - Test coverage verification (>80%)
   - Performance SLA verification
   - Documentation review

5. **Feature Complete** (Target: ~6-8 weeks from now)
   - All 12 acceptance criteria met
   - All tests passing
   - Documentation complete
   - Ready for review

---

## 6. Critical Issues and Risks

### 6.1 Critical Issues

**None identified.**

### 6.2 Risks

1. **Agent 3 Velocity Risk** (MEDIUM)
   - Agent 3 has 25 remaining tasks
   - If Agent 3 encounters blockers or issues, Sprint 3 could be delayed
   - Mitigation: Monitor Agent 3 progress closely, be ready to assist or reassign

2. **Documentation Debt Risk** (LOW)
   - Documentation tasks are vague and need breakdown
   - If not properly planned, Sprint 4 could take longer than estimated
   - Mitigation: Break down documentation tasks into atomic units as recommended

3. **Performance SLA Risk** (LOW)
   - Performance baselines not yet established
   - May not meet 3-second skills list SLA or 15-second installation SLA
   - Mitigation: Add performance tests early in Sprint 4 to establish baselines

4. **macOS Testing Limitation** (LOW - Documented)
   - Cannot test on macOS in current environment
   - Mitigation: Document testing procedure, defer macOS testing to post-v0.1.0

---

## 7. Conclusions

1. **No Gaps in Core Requirements:** All 12 acceptance criteria are covered by planned or completed work.

2. **Backlog Quality:** Generally good, but documentation and testing tasks need to be broken down into more atomic units.

3. **Sprint 3 Progress:** 75% complete (3/4 agents done). Agent 3 should be able to complete remaining work without blockers.

4. **Feature On Track:** At current pace, feature should complete in ~6-8 weeks.

5. **Action Items:**
   - Monitor Agent 3 progress on TODO3.md
   - Break down vague documentation and testing tasks in backlog before Sprint 4
   - Document design decisions for open questions in Sprint 4
   - Establish performance baselines early in Sprint 4

---

## 8. Next Steps for Architect

1. Wait for Agent 3 to complete Sprint 3 tasks
2. Verify Sprint 3 completion (all .agent_done_* files exist)
3. Create .sprint_complete when Sprint 3 is done
4. Clear TODO*.md files for Sprint 4
5. Pull Sprint 4 tasks from backlog (with refined breakdown)
6. Distribute Sprint 4 tasks across agents
7. Monitor Sprint 4 progress

---

**End of Gap Analysis**
