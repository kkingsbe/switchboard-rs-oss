# Question: Timeout Format Support

**Section:** PRD §6.2 (Field Reference)
**Status:** RESOLVED
**Date:** 2026-02-13
**Resolution Date:** 2026-02-13T05:42:00.000Z

## Issue
PRD §6.2 Field Reference specifies the `timeout` field as:

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `agent[].timeout` | string | no | `"30m"` | Max container runtime (e.g. `"30m"`, `"2h"`) |

The description only shows examples with minutes (`"30m"`) and hours (`"2h"`). However, the current implementation in `src/docker/run/wait/timeout.rs` also supports seconds (`"30s"`).

This creates ambiguity about:
- Are seconds (`"30s"`) a valid timeout format?
- Are there other valid formats (e.g., days, weeks)?
- Should we remove the seconds support to strictly match the PRD?

## Context
The BACKLOG.md (line 184) indicates: "fix: Remove seconds support from timeout parsing" with a note that "PRD §6.2 only specifies '30m' and '2h' (minutes and hours)."

However, removing this may break user expectations if seconds are a reasonable timeout unit for short-lived agents.

## Question
Please clarify the supported timeout formats:
1. Should seconds (`"30s"`, `"60s"`) be supported?
2. Are any other formats valid (days, weeks, milliseconds)?
3. Should we strictly limit to minutes and hours as shown in the PRD examples?
4. What error message should be shown for invalid timeout formats?

## Proposed Resolution

**RESOLVED BY:** ARCHITECT_DECISION_timeout_format.md (2026-02-13T05:20:09.000Z)

### Resolution Summary
All questions in this file have been resolved by the architectural decision document.

### Answers to Questions:
1. **Should seconds be supported?** YES - seconds are valid and useful for short-lived agents
2. **Other formats (days, weeks)?** NO - limit to s, m, h for v0.1 simplicity
3. **Strict limit to minutes/hours?** NO - support seconds for user flexibility
4. **Error messages?** Clear, actionable messages showing expected format (e.g., "Invalid timeout format: 'abc'. Expected format like '30s', '5m', '1h'")

### Implementation Details
- Supported formats: seconds ("30s"), minutes ("30m"), hours ("2h")
- Valid range: 1s to 168h (7 days)
- Current implementation already supports all three formats correctly
- No changes required to existing code in src/docker/run/wait/timeout.rs
- Enhanced error messages for better user experience
- Documentation updates needed to reflect all supported formats

See ARCHITECT_DECISION_timeout_format.md for complete implementation requirements and examples.
