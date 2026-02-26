# Testability Enhancement — Backlog

> Feature Doc: addtl-features/testability-enhancement-feature.md
> Created: 2026-02-21T17:15:00Z
> Last Updated: 2026-02-21T18:56:00Z

## Completed Tasks (Sprint 1-2)

### Phase 1: Core Trait Infrastructure ✅

- [x] Define DockerClientTrait in src/traits/mod.rs with all 7 methods (ping, image_exists, build_image, run_container, stop_container, container_logs, wait_container)
- [x] Define ProcessExecutorTrait in src/traits/mod.rs with execute and execute_with_env methods
- [x] Create supporting types: ProcessOutput, ExitStatus (enum: Code/Signal/Unknown), ProcessError
- [x] Implement RealDockerClient struct that wraps bollard::Docker
- [x] Implement RealProcessExecutor struct that wraps std::process::Command

### Phase 2: Module Refactoring - Docker ✅

- [x] Refactor docker/mod.rs to accept DockerClientTrait via dependency injection
- [x] Refactor docker/run.rs to accept DockerClientTrait via dependency injection
- [x] Update all callers of docker module to provide trait implementations

---

## Remaining Tasks

### Phase 3: Module Refactoring - Other Modules

- [ ] Refactor scheduler/mod.rs to accept DockerClientTrait via dependency injection
- [ ] Refactor architect/mod.rs to use ProcessExecutorTrait for git operations
- [ ] Refactor architect/state.rs to use ProcessExecutorTrait
- [ ] Refactor skills/mod.rs to use ProcessExecutorTrait for npx/skills calls
- [ ] Refactor cli/mod.rs to wire up trait dependencies

### Phase 4: Testing Infrastructure

> ⚠️ **Prerequisite needed**: Add `mockall` and `async-trait` dependencies to Cargo.toml before starting these tasks

- [ ] Add mockall dependency to Cargo.toml (dev-dependencies)
- [ ] Add async-trait dependency to Cargo.toml
- [ ] Create mock implementations: MockDockerClient, MockProcessExecutor
- [ ] Create tests/common/ module with fixtures and assertions
- [ ] Create unit tests for docker/mod.rs using mocks (target: 85% coverage)
- [ ] Create unit tests for docker/run.rs using mocks (target: 85% coverage)
- [ ] Create unit tests for scheduler/mod.rs using mocks (target: 80% coverage)
- [ ] Create unit tests for architect/mod.rs using mocks (target: 85% coverage)
- [ ] Create unit tests for architect/state.rs using mocks (target: 85% coverage)
- [ ] Create unit tests for skills/mod.rs using mocks (target: 85% coverage)
- [ ] Create unit tests for cli/mod.rs using mocks (target: 75% coverage)

### Phase 5: Integration & Quality

- [ ] Run full test suite and ensure < 5.5 minutes execution time
- [ ] Verify unit tests run in < 30 seconds
- [ ] Verify overall coverage reaches 80%+
- [ ] Fix any clippy warnings (target: 0 warnings)
- [ ] Ensure rustfmt passes on all modified files
- [ ] Verify backward compatibility - no production behavior changes

---

## Summary

| Phase | Status | Tasks |
|-------|--------|-------|
| Phase 1: Core Trait Infrastructure | ✅ Complete | 5/5 |
| Phase 2: Module Refactoring - Docker | ✅ Complete | 3/3 |
| Phase 3: Module Refactoring - Other Modules | 🔄 Pending | 0/5 |
| Phase 4: Testing Infrastructure | 🔄 Pending | 0/11 |
| Phase 5: Integration & Quality | 🔄 Pending | 0/6 |

**Total Progress: 8/30 tasks complete (27%)**
