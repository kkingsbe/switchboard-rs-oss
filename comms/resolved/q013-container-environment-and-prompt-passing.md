# Resolution: Container Environment Variables and Prompt Passing

**Original Question:** comms/outbox/q013-container-environment-and-prompt-passing.md
**Resolution Date:** 2026-02-14T15:30:00.000Z
**Status:** ✅ RESOLVED - Architect Decision Created

---

## Resolution Summary

Implementation details resolved via architect decision: **ARCHITECT_DECISION_container_env_prompt.md**

### Decisions Made:

1. **Prompt Passing Mechanism:** Option A - CLI argument only
   - Use `kilo --agent-name "<name>" --prompt "<prompt>"` as CLI arguments
   - Do NOT set PROMPT or AGENT_NAME as environment variables
   - Dockerfile ENTRYPOINT is `["kilo"]` (correct per PRD §7)
   - Prompt is passed via `cmd` argument to container
   - Aligns with Kilo Code CLI's expected interface

2. **Prompt File Handling:** Option A - Read file content, pass as CLI argument
   - When `prompt_file` is specified: Read file content at `switchboard up`/`switchboard run` time
   - Pass file content as `--prompt` argument to Kilo Code CLI
   - Do NOT mount prompt file into container
   - Do NOT use PROMPT_FILE environment variable
   - Simpler than mounting and allows path resolution on host

3. **AGENT_NAME Purpose:** Option B - Unused, remove from implementation
   - AGENT_NAME environment variable serves no purpose in Kilo Code CLI
   - Agent name is used by Switchboard scheduler, not Kilo Code CLI
   - Remove AGENT_NAME environment variable from container run
   - Remove from PRD example (confusing/wrong)

4. **Environment Variable Priority:** Option A - User env vars are for agent config only
   - `agent[].env` settings are passed as environment variables to container
   - These are for agent-specific configuration (API keys, feature flags)
   - No built-in env vars (PROMPT, AGENT_NAME) to conflict with
   - Keep namespace clean - user vars are purely for agent use

---

## Implementation Tasks

This decision will be implemented as part of Sprint 4 drift cleanup:
- [`TODO4.md`](../TODO4.md) - "fix: Remove AGENT_NAME and PROMPT env vars (prompt should be CLI argument per PRD §4.3)"
- [`TODO4.md`](../TODO4.md) - "fix: Align entrypoint with PRD specification (prompt as command argument, not entrypoint)"
- Implementation location: `src/docker/run/run.rs`

---

## Related Files

- [`ARCHITECT_DECISION_container_env_prompt.md`](ARCHITECT_DECISION_container_env_prompt.md) - Full decision document
- [`Dockerfile`](../Dockerfile) - ENTRYPOINT ["kilo"]
- [`PRD.md`](../PRD.md) §4.3 - Container Execution Model
