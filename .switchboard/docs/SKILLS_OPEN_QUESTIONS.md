# Skills Feature - Open Questions (Decision Records)

This document tracks open questions and deferred features for the Switchboard skills feature. Each entry documents the current behavior, trade-offs, and recommendations for future development.

---

## OQ-1: Skill install latency and agent timeouts

**Status:** Deferred

**Current Behavior:**
When a skill is installed via `npx`, the installation can take significant time depending on network conditions and package size. Currently, users must manually adjust agent timeouts to accommodate potential installation delays. There is no automatic detection or adjustment of timeouts based on skill installation requirements.

**Trade-offs:**
- **Pro (Manual timeout):** Simplicity - users have full control over timeout values
- **Pro (Manual timeout):** Predictability - no hidden behavior changes based on detected conditions
- **Con (Manual timeout):** User burden - requires understanding of both skills and timeout configuration
- **Con (Auto-adjustment):** Complexity - adds detection logic and potential for incorrect assumptions
- **Con (Auto-adjustment):** May not account for all factors (network speed, package size variations)

**Related Issues:**
- Create issue to track

**Recommendation:**
For v0.1.0, keep manual timeout adjustment. Document this requirement clearly in the skills troubleshooting section. Consider adding a GitHub issue/RFC for auto-adjustment feature in a future release after user feedback is collected.

---

## OQ-2: Skill version pinning support

**Status:** Deferred

**Current Behavior:**
When a skill is specified in the configuration, Switchboard always installs/uses the latest available version of the skill package. There is no mechanism to pin a specific version or range of versions.

**Trade-offs:**
- **Pro (Latest version):** Users always get the newest features and bug fixes
- **Pro (Latest version):** Simpler implementation - no version resolution logic needed
- **Con (Latest version):** No reproducibility - same config may behave differently over time
- **Con (Version pinning):** Adds complexity to configuration parsing and version resolution
- **Con (Version pinning):** Requires understanding of version specifiers (semver ranges)

**Related Issues:**
- Create issue to track

**Recommendation:**
Defer version pinning for v0.1.0. Create a GitHub issue to track requirements and use cases for version pinning. This will help inform the design when implementing this feature in a future release.

---

## OQ-3: Skill caching across runs

**Status:** Deferred

**Current Behavior:**
Every time Switchboard runs, it reinstalls all skills from scratch using `npx -y`. There is no persistent cache of installed skills between runs.

**Trade-offs:**
- **Pro (Fresh install):** Simplicity - no cache management logic needed
- **Pro (Fresh install):** Predictability - always uses latest version of skill packages
- **Pro (Fresh install):** No stale artifacts - reduces potential for corrupted installations
- **Con (Fresh install):** Network overhead - downloads packages on every run
- **Con (Fresh install):** Slower startup - installation time added to every run
- **Con (Caching):** Adds complexity - cache invalidation, storage management
- **Con (Caching):** Potential for stale packages - may miss security updates

**Related Issues:**
- Create issue to track

**Recommendation:**
Accept the network overhead for v0.1.0 to ensure simplicity and freshness. Create a GitHub issue for caching feature request. Consider implementing a simple cache with TTL (time-to-live) in a future release.

---

## OQ-4: npx skills version pinning

**Status:** Deferred

**Current Behavior:**
When running skills via `npx`, Switchboard always uses `npx -y` to ensure the latest version of the skill package is used. There is no option to pin to a specific version or to use a locally installed version.

**Trade-offs:**
- **Pro (Always latest):** Users get latest features without manual intervention
- **Pro (Always latest):** Simpler - no need to manage local installations
- **Con (Always latest):** No reproducibility - behavior may change between runs
- **Con (Version pinning):** Adds complexity to configuration and execution
- **Con (Version pinning):** May frustrate users who want consistent behavior

**Related Issues:**
- Create issue to track

**Recommendation:**
Keep `npx -y` behavior for v0.1.0. Create a GitHub issue to discuss version pinning options and gather user requirements. This could be combined with OQ-2 (Skill version pinning support) for a comprehensive solution.

---

## OQ-5: Skill install failure policy

**Status:** Deferred

**Current Behavior:**
If a skill fails to install (e.g., package not found, network error), Switchboard aborts with a non-zero exit code. There is no mechanism to mark a skill as optional (`skills_optional` flag is deferred).

**Trade-offs:**
- **Pro (Always abort):** Fail-fast - immediately notifies user of problems
- **Pro (Always abort):** Prevents partial/incomplete skill sets that may cause confusion
- **Con (Always abort):** All skills become required - no way to have optional skills
- **Con (Optional skills):** Adds configuration complexity
- **Con (Optional skills):** May mask real errors if users ignore failures

**Related Issues:**
- Create issue to track

**Recommendation:**
Maintain abort-on-failure for v0.1.0. Create a GitHub issue for the optional skills feature request. When implementing, consider:
- Adding a `skills_optional` configuration flag
- Distinguishing between "package not found" (optional) vs "network error" (maybe required)
- Providing clear error messages that indicate which skills failed and why

---

## Summary

| OQ | Title | Deferred For | Issue Created |
|----|-------|--------------|----------------|
| OQ-1 | Skill install latency and agent timeouts | v0.1.0 | No |
| OQ-2 | Skill version pinning support | v0.1.0 | No |
| OQ-3 | Skill caching across runs | v0.1.0 | No |
| OQ-4 | npx skills version pinning | v0.1.0 | No |
| OQ-5 | Skill install failure policy | v0.1.0 | No |

All open questions above are deferred for v0.1.0. GitHub issues should be created to track these for future development consideration.
