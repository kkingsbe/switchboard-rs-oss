# Question: Overlap Mode Configuration

**Section:** PRD §4.2, §9
**Status:** RESOLVED
**Date:** 2026-02-13
**Resolution Date:** 2026-02-13T05:42:00.000Z

## Issue
PRD §9 states: "If an agent is still running when its next scheduled execution fires, skip the new run and log a warning (configurable: skip or queue)"

This statement is ambiguous about:
- Where should the overlap mode be configured? (global `settings` section or per-agent `agent` section?)
- What is the default behavior when not explicitly configured?
- Are only "skip" and "queue" modes supported, or are there other modes?
- BACKLOG.md mentions a "kill" mode that is not referenced in the PRD - should this be supported?

## Context
The current scheduler implementation does not track running agents or detect overlaps. We need to implement this feature per PRD §9, but the configuration format is not clearly specified.

## Question
Please clarify:
1. Should overlap mode be configurable globally (in `settings`), per-agent, or both with agent-level overriding global?
2. What is the default overlap behavior when not explicitly configured?
3. Which overlap modes should be supported: "skip", "queue", "kill", or a subset?
4. If "queue" is supported, is there a maximum queue size?

## Proposed Resolution

**RESOLVED BY:** ARCHITECT_DECISION_overlap_mode.md (2026-02-13T05:19:30.000Z)

### Resolution Summary
All questions in this file have been resolved by the architectural decision document.

### Answers to Questions:
1. **Configuration location:** BOTH global (settings.overlap_mode) and per-agent (agent.overlap_mode), with agent-level overriding global
2. **Default behavior:** "skip" mode (as specified in PRD §9)
3. **Supported modes:** "skip" and "queue" (per PRD §9). "kill" mode is out of scope for v0.1
4. **Queue size:** Default 3, configurable per-agent via max_queue_size

### Implementation Details
- Configuration structure supports global and per-agent override
- effective_overlap_mode() method resolves to: agent-level → global → default (Skip)
- Default queue size: 3, configurable per-agent via max_queue_size
- Kill mode deferred to future consideration (v0.2+)

See ARCHITECT_DECISION_overlap_mode.md for complete implementation requirements and examples.
