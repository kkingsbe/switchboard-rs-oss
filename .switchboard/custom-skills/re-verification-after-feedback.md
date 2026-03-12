# Re-verification After Feedback

> **Source:** Distilled from loops 2, 3, 5
> **Created:** 2026-03-11T03:33:10Z
> **Last updated:** 2026-03-11T13:03:00Z
> **Confidence:** medium

## Context

When receiving a PARTIAL verdict from the verifier, the executor should address the specific feedback and resubmit for re-verification. This iterative approach leads to eventual PASS. This skill applies to any task where the verifier provides feedback that needs to be addressed.

## Pattern

**Iterate on verifier feedback until PASS:**

1. **Understand feedback:** Carefully read the verifier's PARTIAL or FAIL feedback to identify specific issues
2. **Address each issue:** Fix each identified problem (e.g., milestone references, scope violations, missing criteria)
3. **Re-verify:** Run self-verification before resubmitting (build, tests, scope check)
4. **Demonstrate fix:** In the next loop, explicitly show how each feedback item was addressed

## Anti-Patterns

- **Ignoring feedback:** Proceeding without addressing verifier concerns
- **Partial fixes:** Addressing some feedback but not all
- **New issues introduced:** Fixing old issues but creating new ones
- **Rushing resubmission:** Not self-verifying before resubmitting

## Evidence

- **Loop 2 (PARTIAL):** Executor implemented scheduler events correctly, but commit/report incorrectly referenced M1 instead of M2 — scope violation
- **Loop 3 (PASS):** Executor corrected milestone references. All 4 criteria verified: scheduler.started/stopped events, uptime calculation, 4 tests pass
- **Loop 5 (M2 PASS):** Previous PARTIAL issues (milestone reference) resolved. All 4 criteria now verified: scheduler.started/stopped events implemented, uptime calculation works, 4 tests pass. Executor corrected milestone references in re-verification.

## Applicability

This skill applies when:
- Verifier provides PARTIAL verdict with specific issues to fix
- The project uses iterative verification (loop-based workflow)
- Success criteria must be re-verified after fixes

This skill may NOT apply when:
- The task receives a PASS on first attempt
- The task receives a FAIL with fundamental blocking issues
- The workflow does not support iterative resubmission
