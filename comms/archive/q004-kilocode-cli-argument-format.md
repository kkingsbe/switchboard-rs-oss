# Question: Kilo Code CLI Argument Format (AGENT_NAME and PROMPT)

**Section:** PRD §4.2, §4.3, §7
**Status:** OPEN
**Date:** 2026-02-13

## Issue
PRD §4.3 shows the container execution model:

```bash
docker run --rm \
  -v /path/to/user/project:/workspace \
  -e AGENT_NAME=<name> \
  -e PROMPT="<prompt>" \
  switchboard-agent:latest \
  kilo --prompt "<prompt>"
```

This is ambiguous because:
1. The prompt appears twice: once as an environment variable (`-e PROMPT="<prompt>"`) and once as a CLI argument (`kilo --prompt "<prompt>"`)
2. The `AGENT_NAME` is passed as an environment variable but doesn't appear in the CLI arguments
3. PRD §7 sets `ENTRYPOINT ["kilo"]` in the Dockerfile, which means `kilo` should not be repeated in the command

## Context
The drift documentation in TODO.md and BACKLOG.md indicates this should be resolved by passing prompts as CLI arguments only, but it's unclear:
- Whether Kilo Code CLI actually reads AGENT_NAME and PROMPT environment variables
- What the correct argument format should be for the Kilo Code CLI
- Whether the example in PRD §4.3 is intentional or an error

## Question
Please clarify the correct format for passing data to Kilo Code CLI:
1. Should `AGENT_NAME` and `PROMPT` be environment variables, CLI arguments, or both?
2. If CLI arguments, what is the correct argument format? (e.g., `kilo --agent-name <name> --prompt "<prompt>"`)
3. If environment variables, does Kilo Code CLI actually read `AGENT_NAME` and `PROMPT` environment variables?
4. Should the Dockerfile ENTRYPOINT be `["kilo"]`, and if so, should the container command include `kilo` again?

## Proposed Resolution (to be filled)
