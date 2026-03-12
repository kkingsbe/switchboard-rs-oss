# TDD Comprehensive Tests

> **Source:** Distilled from loops 1, 3, 4
> **Created:** 2026-03-11T03:33:10Z
> **Last updated:** 2026-03-11T13:03:00Z
> **Confidence:** high

## Context

When implementing feature code in a Rust project, writing comprehensive tests before or alongside implementation provides strong verification confidence. This skill applies to any code implementation task where test coverage is part of the verification criteria.

## Pattern

**Write comprehensive tests for verification:**

1. **Test-driven approach:** Write tests that verify each success criterion before considering the task complete
2. **Coverage:** Aim for enough tests to verify all functional requirements (e.g., 35 tests for M1, 4 tests for M2)
3. **Integration tests:** Include integration tests that verify the feature works in context
4. **Build + test:** Always run `cargo build` and `cargo test` to confirm implementation correctness

## Anti-Patterns

- **Minimal tests:** Writing only 1-2 tests when more are needed to cover requirements
- **Implementation-first:** Writing code without tests, then adding tests as an afterthought
- **Ignoring test failures:** Proceeding despite failing tests
- **Scope-limited tests:** Tests that only cover the "happy path"
- **Deferring tests:** Leaving tests for later rather than writing alongside implementation

## Evidence

- **M1 (Loop 1):** 35 observability tests passed - verifier confirmed TDD approach provides strong confidence in implementation correctness
- **M2 (Loop 3):** 4 scheduler event tests passed (uptime calculation, lifecycle events, event emission) - all 4 criteria verified
- **M3 (Loop 4):** Executor deviated from strict TDD (wrote implementation first), but wrote comprehensive tests afterward (21 container event tests). All tests pass. The deviation was acceptable because the resulting test coverage was thorough.

## Applicability

This skill is critical when:
- The verifier uses test pass rates as verification criteria
- The project has established test infrastructure (cargo test)
- Multiple success criteria need to be individually verified

This skill may NOT apply when:
- The task is purely documentation or configuration (no code)
- The project has no test infrastructure
- Tests are explicitly out of scope for the task
