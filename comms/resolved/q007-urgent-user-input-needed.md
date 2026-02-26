# q007: Urgent User Input Required for v0.1 Architecture Decisions

**Status:** RESOLVED
**Resolution Date:** 2026-02-13T14:26:00.000Z
**Resolved By:** ARCHITECT_DECISION_kilocode_config_management.md

> **RESOLVED:** All questions in this file have been resolved by the architectural decision document.
> See [`ARCHITECT_DECISION_kilocode_config_management.md`](../../ARCHITECT_DECISION_kilocode_config_management.md) for complete details.

---

## Decision 1: .kilocode Directory Format (q002)

### Question
What is the structure and format of the `.kilocode` directory that gets copied into containers?

### Context
- PRD §6.2 mentions `.kilocode` directory configuration
- Dockerfile copies `.kilocode/` to `/root/.kilocode/` in the container
- This is external to Switchboard (managed by user)
- But Switchboard needs to know the format to provide better error messages

### Options (please choose one or specify your own)

**Option A: Kilo Code standard format**
```
.kilocode/
├── config.yaml      # Kilo Code CLI configuration
└── prompts/         # Prompt templates
    ├── default.md
    └── custom.md
```

**Option B: Minimal format (Switchboard doesn't care)**
- Treat `.kilocode/` as opaque
- Only check that it exists (don't validate contents)
- Let Kilo Code CLI handle all validation

**Option C: Custom format (please describe)**
```
.kilocode/
└── [your structure here]
```

### Recommendation
**Option B** - Treat as opaque. Kilo Code CLI handles validation, Switchboard only checks directory exists.

---

## Decision 2: .kilocode Architecture - API Key Management (q006)

### Question
**CRITICAL**: How should API keys be managed for Kilo Code CLI in the containerized environment?

### Context
- Kilo Code CLI needs API keys to function (e.g., OpenAI, Anthropic)
- These keys are sensitive and must NOT be in version control
- Multiple possible approaches with different security tradeoffs
- This decision affects q002 (directory format)

### Options (please choose one)

**Option A: Mount from Host**
```bash
docker run -v ~/.kilocode:/root/.kilocode ...
```
- User manages keys on host system
- Keys are mounted into container at runtime
- **Pros**: Simple, user has full control, no extra Switchboard code
- **Cons**: Keys must exist on host, requires user setup

**Option B: Environment Variables**
```bash
# In switchboard.toml
[settings]
kilocode_api_key = "sk-..."  # OR: env variable reference

# Docker sets: KILOCODE_API_KEY=sk-...
```
- Keys stored in switchboard.toml or referenced from env
- **Pros**: Explicit in config, can use host env vars, no mount needed
- **Cons**: Keys in config file (if not using env reference), must be carefully documented

**Option C: Docker Secrets (docker run --secret)**
```bash
docker run --secret id=kilocode_api_key,src=/path/to/key ...
```
- Uses Docker secrets mechanism
- **Pros**: Most secure, keys never in env/filesystem
- **Cons**: More complex, requires user to use --secret flag

**Option D: User-Provided Directory (Option A + .kilocode structure)**
```bash
.kilocode/
├── config.yaml      # Contains: api_key: "${KILOCODE_API_KEY}" or plain text
└── prompts/
    └── default.md

docker run -v .kilocode:/root/.kilocode -e KILOCODE_API_KEY=sk-...
```
- User manages `.kilocode/` directory structure
- Keys can be in config file OR environment variables
- **Pros**: Flexible, Kilo Code CLI standard, supports both approaches
- **Cons**: User must understand Kilo Code CLI format

### Recommendation
**Option D** - This is the standard Kilo Code CLI approach. The `.kilocode/` directory structure follows Kilo Code conventions, and keys can be either:
1. In `config.yaml` (less secure but simpler)
2. Environment variables (more secure, recommended)

**Combined with q002 = Option B** (treat `.kilocode/` as opaque):
- Switchboard checks `.kilocode/` exists on host
- Mounts it to `/root/.kilocode/` in container
- User manages keys via Kilo Code's standard mechanisms
- Switchboard doesn't need to know the directory structure

---

## Summary Table

| Question | Options | Recommendation | Urgency |
|----------|---------|----------------|---------|
| q002 (.kilocode format) | A, B, C | B (opaque) | Medium |
| q006 (API keys) | A, B, C, D | D (standard + env) | **HIGH** |

---

## Resolution Summary

All questions in this file have been resolved by [`ARCHITECT_DECISION_kilocode_config_management.md`](../../ARCHITECT_DECISION_kilocode_config_management.md).

### Answers to Questions:

**Decision 1: .kilocode Directory Format (q002)**
- **Answer:** Treat `.kilocode/` as **opaque** (Option B)
- **Implementation:** Switchboard only checks that directory exists; Kilo Code CLI handles all validation
- **No format specification needed** - Kilo Code CLI manages its own configuration

**Decision 2: .kilocode Architecture - API Key Management (q006)**
- **Answer:** Mount `.kilocode` from user's project directory at runtime (Option A)
- **Implementation:**
  - Users create `.kilocode/` in their project root
  - Docker mounts this to `/root/.kilocode/` at runtime
  - Each project has its own configuration
  - Clear error if directory is missing
- **Modified Dockerfile:** Remove `COPY .kilocode` line; create directory with `RUN mkdir -p /root/.kilocode`
- **Updated mount command:** Add `-v .kilocode:/root/.kilocode` to `docker run`

### Implementation Requirements

See ARCHITECT_DECISION_kilocode_config_management.md for complete implementation details:
- Dockerfile modifications (remove COPY, create directory)
- Config parser updates (add `kilocode_config_dir` setting)
- Docker run command updates (add .kilocode mount)
- Error messages (missing .kilocode directory)
- Documentation updates (README.md, .gitignore)

---

## Next Steps

1. **[COMPLETED]** Please reply with your choices for q002 and q006
2. **[COMPLETED]** Once you respond, I will:
     - Document your decisions in ARCHITECT_DECISION_*.md files
     - Move resolved questions to comms/resolved/
     - Update BACKLOG.md if needed
     - Sprint 3 agents can continue working independently

**Note**: Sprint 3 agents (TODO1-4.md) can proceed in parallel while you provide these answers. None of the current tasks block on these decisions.

---

Generated: 2026-02-13T05:44:00.000Z
Source: Architect Workflow Task 4
Updated: 2026-02-13T11:35:00.000Z (q004 resolved)
Resolved: 2026-02-13T14:26:00.000Z (q002 and q006 resolved)
