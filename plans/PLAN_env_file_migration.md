# Plan: Environment Variable Migration from switchboard.toml to switchboard.env

## Executive Summary

This plan outlines the implementation of a separate `switchboard.env` file for storing sensitive environment variables, replacing the current insecure practice of storing API keys and tokens directly in `gast## 1. Current Stateown.toml`.

---

 Analysis

### 1.1 How Environment Variables Are Currently Stored

| Location | Format | Example | Security |
|----------|--------|---------|----------|
| Agent env | Inline TOML table | `env = { AGENT_ID = "1" }` | OK for non-sensitive |
| Discord token_env | String field | `token_env = "DISCORD_TOKEN"` or actual token | ⚠️ Both patterns used |
| Discord api_key_env | String field | `api_key_env = "OPENROUTER_API_KEY"` or actual key | ⚠️ Both patterns used |

### 1.2 Current Security Issues in switchboard.toml

From [`switchboard.toml:56`](switchboard.toml:56):
```toml
token_env = "${DISCORD_TOKEN}"
```

From [`switchboard.toml:61`](switchboard.toml:61):
```toml
api_key_env = "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"
```

These are actual credentials stored in plain text - a serious security vulnerability!

### 1.3 Code Locations

| File | Purpose |
|------|---------|
| [`src/config/mod.rs`](src/config/mod.rs:1) | Config parsing (Config, Agent structs) |
| [`src/discord/config.rs`](src/discord/config.rs:1) | Discord configuration (DiscordSection) |
| [`src/docker/run/run.rs`](src/docker/run/run.rs:1) | Container environment variable handling |
| [`src/cli/mod.rs`](src/cli/mod.rs:1) | CLI commands that use config |

---

## 2. Proposed Design

### 2.1 switchboard.env File Format

Standard `.env` format with KEY=value pairs:

```bash
# switchboard.env - Environment variables for switchboard
# Copy this file to switchboard.env and fill in your values
# NEVER commit this file to version control (it's already in .gitignore)

# Discord Configuration
DISCORD_TOKEN=your-discord-bot-token-here
OPENROUTER_API_KEY=sk-or-v1-your-openrouter-api-key-here

# Agent Environment Variables
AGENT_ID=1
API_KEY=your-api-key-here
LOG_LEVEL=info
MAX_RETRIES=3
```

### 2.2 switchboard.toml Reference Syntax

Two options for referencing environment variables in switchboard.toml:

#### Option A: `${ENV_VAR_NAME}` Syntax (Recommended)
```toml
# Agent environment - use env: prefix for references
[[agent]]
name = "gt-dev-1"
env = { 
    AGENT_ID = "1",
    API_KEY = "${MY_API_KEY}",
    LOG_LEVEL = "${LOG_LEVEL:-info}"  # with default
}

# Discord - use env: prefix for references
[discord]
enabled = true
token_env = "${DISCORD_TOKEN}"
channel_id = "1472443428569874533"

[discord.llm]
api_key_env = "${OPENROUTER_API_KEY}"
```

#### Option B: `env:VAR_NAME` Syntax (Alternative)
```toml
env = { 
    API_KEY = "env:MY_API_KEY",
}
```

**Decision: Use Option A (`${VAR}`) syntax** because:
- More widely recognized (used in Docker, Compose, many tools)
- Supports default values: `${VAR:-default}`
- More intuitive for users familiar with shell variable expansion

### 2.3 Environment Variable Resolution Order

When resolving an environment variable reference:

```
1. switchboard.env file (loaded first, same directory as switchboard.toml)
2. Process environment variables (already set in shell)
3. Default value (if specified: ${VAR:-default})
4. Error if not found and no default
```

---

## 3. Implementation Plan

### Phase 1: Core Infrastructure Changes

#### 1.1 Add dotenv dependency
**File:** [`Cargo.toml`](Cargo.toml:1)

Add the `dotenv` crate to dependencies:
```toml
dotenv = "0.15"
```

#### 1.2 Create env file loader module
**New File:** [`src/config/env.rs`](src/config/env.rs:1)

```rust
// Core functionality:
pub fn load_env_file(config_dir: &Path) -> Result<HashMap<String, String>, EnvError>;

pub fn resolve_env_var(
    value: &str,
    env_vars: &HashMap<String, String>,
) -> Result<String, EnvError>;

pub fn parse_env_reference(value: &str) -> Option<(String, Option<String>)>;
// Returns (var_name, default_value) if value is ${VAR} or ${VAR:-default}
```

### Phase 2: Config Module Updates

#### 2.1 Modify Agent struct
**File:** [`src/config/mod.rs`](src/config/mod.rs:762)

Current:
```rust
pub env: Option<HashMap<String, String>>,
```

New: Add a method to resolve environment variable references:
```rust
impl Agent {
    /// Resolve environment variables with env file and defaults
    pub fn resolve_env(&self, env_file_vars: &HashMap<String, String>) -> Vec<String> {
        // Convert HashMap<String, String> to resolved KEY=value strings
        // For each value, check if it's ${VAR} or ${VAR:-default}
        // Resolve against env_file_vars, then process env
    }
}
```

#### 2.2 Modify DiscordSection
**File:** [`src/discord/config.rs`](src/discord/config.rs:341)

Update token_env and api_key_env resolution to use the new env file loader:
```rust
impl DiscordSection {
    /// Resolve environment variable references in Discord config
    pub fn resolve_env(&self, env_file_vars: &HashMap<String, String>) -> ResolvedDiscordConfig {
        // Resolve token_env, api_key_env using env file
    }
}
```

#### 2.3 Update Config loading
**File:** [`src/config/mod.rs`](src/config/mod.rs:945)

```rust
impl Config {
    pub fn from_toml(path: &Path) -> Result<Self, ConfigError> {
        // Existing code...
        
        // NEW: Load switchboard.env file
        let env_file_vars = load_env_file(path)?;
        
        // NEW: Store env file vars for later resolution
        // Could be stored in Config struct or resolved immediately
    }
}
```

### Phase 3: Container Integration

#### 3.1 Update build_container_env_vars
**File:** [`src/docker/run/run.rs`](src/docker/run/run.rs:145)

The function already receives `Vec<String>` in "KEY=value" format. No changes needed here - resolution happens in config module.

#### 3.2 Update CLI commands
**File:** [`src/cli/mod.rs`](src/cli/mod.rs:1449)

```rust
// Current:
let env_vars = agent.env(settings);

// New:
let env_vars = agent.resolve_env(env_file_vars);
```

### Phase 4: Discord Module Updates

#### 4.1 Update Discord config loading
**File:** [`src/discord/mod.rs`](src/discord/mod.rs:225)

Replace the hacky detection:
```rust
// OLD (problematic):
if toml_cfg.token_env.contains('.') {
    // Use as actual token
}

// NEW:
let env_vars = load_env_file_from_config_dir()?;
let resolved = resolve_env_var(&toml_cfg.token_env, &env_vars)?;
```

### Phase 5: Documentation and Migration

#### 5.1 Update switchboard.sample.toml
**File:** [`switchboard.sample.toml`](switchboard.sample.toml:1)

Replace inline env examples with references:
```toml
# OLD:
env = { API_KEY = "your-api-key-here", LOG_LEVEL = "info" }

# NEW:
env = { API_KEY = "${API_KEY}", LOG_LEVEL = "${LOG_LEVEL:-info}" }
```

#### 5.2 Create switchboard.sample.env
**New File:** [`switchboard.sample.env`](switchboard.sample.env:1)

```bash
# switchboard.sample.env - Sample environment variables
# Copy to switchboard.env and fill in your values

# Discord Bot Token
DISCORD_TOKEN=your-discord-bot-token-here

# OpenRouter API Key
OPENROUTER_API_KEY=your-openrouter-api-key-here

# Agent Configuration
LOG_LEVEL=info
MAX_RETRIES=3
```

#### 5.3 Update documentation
- Update [`docs/setup.md`](docs/setup.md:1) with switchboard.env instructions
- Add migration guide

---

## 4. Backward Compatibility

### 4.1 Migration Strategy

| Scenario | Behavior |
|----------|----------|
| switchboard.toml has inline value (not reference) | Use value directly (backward compatible) |
| switchboard.toml has `${VAR}` reference | Resolve from switchboard.env or env |
| switchboard.toml has `env:VAR` (old style) | Convert to `${VAR}` internally |
| switchboard.env missing but referenced | Error with helpful message |
| Only switchboard.env has value, no reference | Works (env var is available in container) |

### 4.2 Gradual Migration Path

1. **Phase 1 (Opt-in):** Add new env file support alongside existing inline values
2. **Phase 2 (Deprecation):** Warn when inline sensitive values detected
3. **Phase 3 (Required):** Require references for sensitive values (future major version)

---

## 5. File Changes Summary

| Action | File | Description |
|--------|------|-------------|
| Modify | [`Cargo.toml`](Cargo.toml:1) | Add dotenv dependency |
| Create | [`src/config/env.rs`](src/config/env.rs:1) | New env file loader module |
| Modify | [`src/config/mod.rs`](src/config/mod.rs:1) | Add env resolution to Agent |
| Modify | [`src/discord/config.rs`](src/discord/config.rs:1) | Add env resolution to DiscordSection |
| Modify | [`src/discord/mod.rs`](src/discord/mod.rs:1) | Use new env resolution |
| Modify | [`src/cli/mod.rs`](src/cli/mod.rs:1) | Use resolved env vars |
| Create | [`switchboard.sample.env`](switchboard.sample.env:1) | Sample env file |
| Modify | [`switchboard.sample.toml`](switchboard.sample.toml:1) | Update examples |
| Modify | [`.gitignore`](.gitignore:1) | Ensure switchboard.env is ignored |

---

## 6. Error Handling

### 6.1 Error Types (New)

```rust
#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("Environment variable {0} not found")]
    VarNotFound(String),
    
    #[error("Failed to load switchboard.env: {0}")]
    LoadError(String),
    
    #[error("Invalid environment variable reference: {0}")]
    InvalidReference(String),
}
```

### 6.2 User-Friendly Messages

```
Error: Environment variable 'DISCORD_TOKEN' not found

Solution:
1. Create a switchboard.env file in the same directory as switchboard.toml
2. Add: DISCORD_TOKEN=your-token-here
3. Or set the variable in your shell environment
```

---

## 7. Testing Plan

### 7.1 Unit Tests

| Test | Description |
|------|-------------|
| `test_env_file_parsing` | Parse standard .env format |
| `test_env_reference_resolution` | Resolve ${VAR} and ${VAR:-default} |
| `test_missing_var_error` | Error when referenced var not found |
| `test_inline_value_unchanged` | Non-reference values passed through |

### 7.2 Integration Tests

| Test | Description |
|------|-------------|
| `test_env_file_loaded_with_config` | switchboard.env loaded when config loaded |
| `test_container_receives_resolved_env` | Env vars resolved before container start |
| `test_discord_uses_resolved_env` | Discord gets resolved API key |

---

## 8. Security Considerations

1. **switchboard.env in .gitignore** - Already present (`.env` and `switchboard.toml` are ignored)
2. **No logging of secrets** - Ensure resolved values not logged
3. **File permissions** - Recommend 600 for switchboard.env
4. **Validation** - Warn if switchboard.env has overly permissive permissions

---

## 9. Migration Example

### Before (Insecure):
```toml
# switchboard.toml
[discord]
enabled = true
token_env = "MTQ2NjU1MTMyOTk5OTgxODg6.GrMeWu..."

[discord.llm]
api_key_env = "sk-or-v1-abc123..."
```

### After (Secure):
```bash
# switchboard.env (in same directory as switchboard.toml)
DISCORD_TOKEN=MTQ2NjU1MTMyOTk5OTgxODg6.GrMeWu...
OPENROUTER_API_KEY=sk-or-v1-abc123...
```

```toml
# switchboard.toml
[discord]
enabled = true
token_env = "${DISCORD_TOKEN}"

[discord.llm]
api_key_env = "${OPENROUTER_API_KEY}"
```

---

## 10. Implementation Order

```
1. [ ] Add dotenv to Cargo.toml
2. [ ] Create src/config/env.rs module
3. [ ] Add Agent::resolve_env() method
4. [ ] Add DiscordSection::resolve_env() method
5. [ ] Update Config::from_toml() to load env file
6. [ ] Update src/cli/mod.rs to use resolved env
7. [ ] Update src/discord/mod.rs to use resolved env
8. [ ] Create switchboard.sample.env
9. [ ] Update switchboard.sample.toml examples
10. [ ] Add unit tests
11. [ ] Add integration tests
12. [ ] Update documentation
```

---

This plan provides a secure, backward-compatible migration path that addresses the current security vulnerabilities while maintaining existing functionality.
