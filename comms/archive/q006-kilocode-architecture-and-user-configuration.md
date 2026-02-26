# Question: .kilocode Directory Architecture and User Configuration

**Section:** PRD §4.2, §9
**Status:** OPEN
**Date:** 2026-02-13

## Issue

PRD §4.2 specifies that `.kilocode` should be copied from the Switchboard repo into the Docker image. This creates a fundamental architectural problem for a globally distributed tool:

1. Users install Switchboard globally via `cargo install --path .`
2. The Dockerfile builds the agent image with `.kilocode` copied from the Switchboard repo
3. ALL users would share the same API keys (the ones pre-configured in the Switchboard repo)

This contradicts the expected use case where:
- Each user needs their own API keys
- Different users may use different AI providers
- API keys must be kept secure and private

## Question

Please clarify the intended architecture for `.kilocode` configuration:

1. Should `.kilocode` be copied from the user's project directory (where they place their own API keys) instead of from the Switchboard repo?
2. If so, should the user be required to create `.kilocode/` in their project directory, or should there be a fallback mechanism?
3. What should happen if `.kilocode` is missing? Should Switchboard:
   - Fail with a helpful error message (per PRD §9)?
   - Prompt users to configure it?
   - Use environment variables as an alternative?
4. What is the intended format and structure of `.kilocode/`? (This relates to q002)
5. Should the Dockerfile be changed to copy `.kilocode` from the build context (user's project) instead of the Switchboard repo?

## Proposed Resolution Options

**Option A: Project-local .kilocode**
- Users create `.kilocode/` directory in their project root
- Dockerfile copies from build context (user's project)
- Each project has its own configuration
- Clear error if missing

**Option B: User-home .kilocode**
- Users configure `.kilocode` in their home directory (~/.kilocode)
- Dockerfile mounts from user's home directory
- Configuration shared across all projects
- Global configuration model

**Option C: Environment Variables**
- API keys passed via environment variables at runtime
- No .kilocode directory required in image
- Most flexible but potentially more complex setup

**Option D: Hybrid**
- Support multiple configuration sources with priority:
  1. Project-local `.kilocode/`
  2. User-home `.kilocode/`
  3. Environment variables

Please specify which approach should be implemented or if a different architecture is intended.
