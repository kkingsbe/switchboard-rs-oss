# Pre-existing Implementation Verification

> **Source:** Distilled from loops 6, 7
> **Created:** 2026-03-11T16:00:05Z
> **Last updated:** 2026-03-11T16:00:05Z
> **Confidence:** high

## Context

When working in a workspace where feature implementation may already exist in the codebase, the executor's role shifts from implementation to verification. This pattern occurs when code was written in earlier development phases or by other team members before the milestone task was assigned. This skill applies to any milestone-based task in an existing project where implementation may already be present.

## Pattern

**Verify existing implementation rather than re-implement:**

1. **Check for existing code:** Before implementing, search the codebase for similar functionality
2. **Verify correctness:** Run tests, build, and check if existing code meets success criteria
3. **Report honestly:** Clearly state whether code was newly implemented or already existed
4. **Commit appropriately:** If only state files changed (verification results), note this in the report
5. **Focus on criteria:** The verifier checks if success criteria are met, not just "lines of code added"

## Anti-Patterns

- **Re-implementing existing code:** Writing new code when working implementation already exists
- **Claiming new implementation:** Stating code was "implemented" when it already existed
- **Ignoring existing patterns:** Not following established code patterns in the module
- **Verification skipping:** Assuming existing code works without running tests/build

## Evidence

- **Loop 6 (M4 - Git Diff Capture):** Implementation already existed in codebase. Executor verified it works, commit only contained state files. Verifier confirmed: "Implementation already existed - task was verification rather than new implementation"
- **Loop 7 (M5 - Log Rotation):** Implementation already existed in codebase (same pattern as M4). Executor verified all 9 tests pass. Commit `64a4bab` only contains state files. Verifier noted: "Similar to M4 - code was pre-existing in repository"

## Applicability

This skill applies when:
- Working on an existing project with prior implementations
- Success criteria can be verified through tests/build
- The verifier checks for functional correctness, not just "new code"

This skill may NOT apply when:
- Working on greenfield implementations with no existing code
- The task explicitly requires new implementation from scratch
- No tests or verification criteria exist for the feature
