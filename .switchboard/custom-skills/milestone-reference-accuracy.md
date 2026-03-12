# Milestone Reference Accuracy

> **Source:** Distilled from loops 2, 3, 4
> **Created:** 2026-03-11T03:33:10Z
> **Last updated:** 2026-03-11T13:03:00Z
> **Confidence:** high

## Context

When implementing milestones in a multi-milestone project, the executor must ensure that commit messages, reports, and milestone references accurately reflect the actual work performed. This skill applies to any code implementation task involving milestone-based workflow.

## Pattern

**DO ensure milestone references match the actual work:**

1. **Before committing:** Verify the milestone label (e.g., [M2]) matches the code being committed
2. **In reports:** Confirm the milestone identity in EXECUTION_REPORT.md matches the implementation
3. **Cross-check:** Review git diff to confirm the scope aligns with the claimed milestone
4. **Verifier alignment:** Ensure the executor and verifier agree on which milestone is being implemented

## Anti-Patterns

- **Mismatched milestone labels:** Implementing M2 code but referencing M1 in commits
- **Scope drift:** Adding code for a future milestone while working on current one
- **Copy-paste errors:** Using milestone references from previous work without updating
- **No commits at all:** Failing to create any commits with milestone references — changes remain unstaged
- **False reporting:** Claiming no work was done when code was actually implemented

## Evidence

- **Loop 2 (PARTIAL):** Executor correctly implemented scheduler events (M2), but commit and report incorrectly referenced M1 instead of M2 — scope violation
- **Loop 3 (PASS):** Executor corrected milestone references. Verifier confirmed correct [M2] label in commit `3bff647`
- **Loop 4 (M3):** Executor made NO COMMITS at all for M3 implementation. 477 lines of code were added but remained unstaged — complete violation of milestone reference requirement

## Applicability

This skill is critical when:
- Working on multi-milestone projects with sequential milestones
- Using milestone labels in commit messages (e.g., [M1], [M2], [M3])
- The verifier checks milestone identity as part of scope compliance

This skill may NOT apply when:
- Working on single-milestone tasks without milestone labeling
- Milestone references are not part of the project's verification criteria
