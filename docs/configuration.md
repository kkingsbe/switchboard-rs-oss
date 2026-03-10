# Configuration Reference

Switchboard uses TOML configuration files to define agents, schedules, and integrations. This document provides a complete reference for all configuration options.

> **Last updated:** 2026-03-06

## File Location

Switchboard looks for configuration in the following locations:

| Location | Description |
|----------|-------------|
| `./switchboard.toml` | Default location in current directory |
| Custom path | Use `switchboard -c <path> config.toml` |

### Validation

Validate your configuration before running:

```bash
switchboard validate switchboard.toml
```

---

## Full TOML Schema

### `[settings]` - Global Settings

The `[settings]` section provides defaults that apply to all agents. All fields are optional.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `image_name` | String | `"switchboard-agent"` | Docker image name for agent containers |
| `image_tag` | String | `"latest"` | Docker image tag/version |
| `log_dir` | String | `".switchboard/logs"` | Directory for agent log files |
| `timezone` | String | `"system"` | Timezone for cron schedules (IANA format or "system") |
| `overlap_mode` | String | `"skip"` | How to handle concurrent executions: `"skip"` or `"queue"` |
| `silent_timeout` | String | (none) | Default silent timeout (e.g., "5m", "0" to disable) |

```toml
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "America/New_York"
overlap_mode = "skip"
```

---

### `[[agent]]` - Agent Definitions

Each `[[agent]]` section defines a scheduled task. At least one agent is required.

#### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Unique identifier for this agent |
| `schedule` | String | Cron expression for execution schedule |
| `prompt` XOR `prompt_file` | String | Task prompt (inline or file path) |

#### Optional Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `env` | Table | `{}` | Environment variables passed to the container |
| `readonly` | Boolean | `false` | If true, agent cannot write to filesystem |
| `timeout` | String | `"30m"` | Maximum execution duration |
| `overlap_mode` | String | (from settings) | Override global overlap mode |
| `silent_timeout` | String | (from settings) | Auto-terminate if no logs for duration |
| `max_queue_size` | Number | `3` | Max queued runs when overlap_mode is "queue" |
| `skills` | Array | `[]` | List of skills available to this agent |

#### Agent Example

```toml
[[agent]]
name = "daily-report"
schedule = "0 9 * * *"
prompt = "Generate a daily summary of codebase changes."

# Optional: Override global settings
env = { API_KEY = "your-key" }
timeout = "1h"
readonly = false
```

---

### `[discord]` - Discord Bot Configuration

Enable the Discord concierge to interact with agents via a Discord channel.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | Boolean | `false` | Enable/disable Discord integration |
| `token_env` | String | `"DISCORD_TOKEN"` | Env var containing Discord bot token |
| `channel_id` | String | (required) | Discord channel ID to listen on |
| `intents` | Number | `21504` | Gateway intents (see below) |

```toml
[discord]
enabled = true
token_env = "${DISCORD_TOKEN}"
channel_id = "1474550134388949272"
intents = 21504
```

#### Gateway Intents

Intents control which Discord events the bot receives:

| Value | Intent | Notes |
|-------|--------|-------|
| `512` | GUILD_MESSAGES | Messages in servers |
| `4096` | DIRECT_MESSAGES | Direct messages |
| `16384` | MESSAGE_CONTENT | **Requires verification** in Discord Developer Portal |
| `21504` | All combined | Default value |

> **Note:** If you receive 400 Bad Request errors, your bot may not have MESSAGE_CONTENT intent verified. Use `intents = 512` as a fallback.

---

### `[discord.llm]` - LLM Configuration

Configure the language model for Discord responses.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | String | `"openrouter"` | LLM provider (only `openrouter` supported) |
| `api_key_env` | String | `"OPENROUTER_API_KEY"` | Env var containing API key |
| `model` | String | `"anthropic/claude-sonnet-4"` | Model identifier |
| `max_tokens` | Number | `1024` | Maximum tokens per response |
| `system_prompt_file` | String | (built-in) | Path to custom system prompt |

```toml
[discord.llm]
provider = "openrouter"
api_key_env = "${OPENROUTER_API_KEY}"
model = "anthropic/claude-sonnet-4"
max_tokens = 1024
system_prompt_file = "prompts/concierge.md"
```

---

### `[discord.conversation]` - Conversation Settings

Control conversation history and expiration.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_history` | Number | `20` | Maximum messages per user history |
| `ttl_minutes` | Number | `60` | Conversation expiration time in minutes |

```toml
[discord.conversation]
max_history = 30
ttl_minutes = 120
```

---

## Cron Expressions

Switchboard uses 5-field cron expressions: `minute hour day month day-of-week`

### Format

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, 0=Sunday)
│ │ │ │ │
* * * * *
```

### Special Characters

| Character | Meaning | Example |
|-----------|---------|---------|
| `*` | Any value | `* * * * *` = every minute |
| `*/n` | Every n units | `*/15 * * * *` = every 15 minutes |
| `,` | List values | `0 9 * * 1,3,5` = Mon/Wed/Fri at 9 AM |
| `-` | Range | `0 9 * * 1-5` = weekdays at 9 AM |

### Common Examples

| Expression | Description |
|------------|-------------|
| `0 * * * *` | Every hour at :00 |
| `*/15 * * * *` | Every 15 minutes |
| `0 9 * * *` | Daily at 9:00 AM |
| `0 9 * * 1-5` | Weekdays at 9:00 AM |
| `0 0 * * *` | Daily at midnight |
| `0 8 * * 1` | Every Monday at 8:00 AM |
| `0 0 1 * *` | First day of every month at midnight |

> **Tip:** Use [crontab.guru](https://crontab.guru) to build and verify cron expressions.

---

## Environment Variables

### Referencing in Configuration

Use the `${VAR_NAME}` syntax to reference environment variables:

```toml
[discord]
token_env = "${DISCORD_TOKEN}"
channel_id = "${DISCORD_CHANNEL_ID}"

[discord.llm]
api_key_env = "${OPENROUTER_API_KEY}"
```

#### Default Values

Provide fallback values using `${VAR_NAME:-default}`:

```toml
token_env = "${DISCORD_TOKEN:-}"           # Empty string if not set
api_key_env = "${OPENROUTER_API_KEY:-sk}"  # "sk" if not set
```

### Required Variables

For Discord integration, set these in your environment:

| Variable | Description |
|----------|-------------|
| `DISCORD_TOKEN` | Your Discord bot token |
| `DISCORD_CHANNEL_ID` | Discord channel ID to listen on |
| `OPENROUTER_API_KEY` | Your OpenRouter API key |

Create a `.env` file (add to `.gitignore`):

```bash
# .env
DISCORD_TOKEN=your-bot-token-here
DISCORD_CHANNEL_ID=1474550134388949272
OPENROUTER_API_KEY=your-openrouter-key-here
```

---

## Configuration Examples

### Minimal Configuration

```toml
[[agent]]
name = "hello"
schedule = "0 9 * * *"
prompt = "Say hello."
```

### Single Agent with Full Options

```toml
[settings]
image_name = "switchboard-agent"
timezone = "America/New_York"
overlap_mode = "skip"

[[agent]]
name = "code-reviewer"
schedule = "0 9 * * 1-5"  # Weekdays at 9 AM
prompt = "Review recent PRs and provide feedback."
timeout = "1h"
env = { GITHUB_TOKEN = "your-token" }
```

### Multiple Agents

```toml
[settings]
log_dir = ".switchboard/logs"

[[agent]]
name = "morning-report"
schedule = "0 9 * * *"
prompt = "Generate daily summary."

[[agent]]
name = "health-check"
schedule = "*/15 * * * *"
prompt = "Check system health."
timeout = "5m"
overlap_mode = "queue"
max_queue_size = 3

[[agent]]
name = "log-analyzer"
schedule = "0 0 * * *"
prompt = "Analyze yesterday's logs."
readonly = true
timeout = "30m"
```

### Full Featured with Discord

```toml
[settings]
image_name = "switchboard-agent"
image_tag = "latest"
log_dir = ".switchboard/logs"
timezone = "UTC"
overlap_mode = "skip"

[[agent]]
name = "daily-summary"
schedule = "0 9 * * *"
prompt = "Generate a daily project summary."
timeout = "1h"
skills = ["code-analysis"]

[discord]
enabled = true
token_env = "${DISCORD_TOKEN}"
channel_id = "${DISCORD_CHANNEL_ID}"
intents = 21504

[discord.llm]
provider = "openrouter"
api_key_env = "${OPENROUTER_API_KEY}"
model = "anthropic/claude-sonnet-4"
max_tokens = 1024

[discord.conversation]
max_history = 30
ttl_minutes = 120
```

---

## Overlap Modes

### Skip Mode (Default)

If an agent is already running when a new scheduled execution triggers, the new run is **skipped**.

```toml
overlap_mode = "skip"
```

Best for:
- Long-running tasks where concurrent execution could cause issues
- Tasks that are idempotent (safe to skip)
- Resource-intensive operations

### Queue Mode

If an agent is already running, new runs are **queued** and executed sequentially.

```toml
overlap_mode = "queue"
max_queue_size = 5  # Maximum queued runs
```

Best for:
- Quick tasks where missing a run is problematic
- Tasks that must all be processed
- High-frequency schedules

---

## Timeout Format

Timeouts use the format: `<number><unit>`

| Unit | Description | Example |
|------|-------------|---------|
| `s` | Seconds | `"30s"` |
| `m` | Minutes | `"5m"`, `"30m"`, `"60m"` |
| `h` | Hours | `"1h"`, `"2h"` |

> **Note:** Compound formats like `"2h30m"` are not supported. Use `"150m"` instead.

```toml
timeout = "30s"   # 30 seconds
timeout = "15m"   # 15 minutes
timeout = "2h"    # 2 hours
```

---

## Skills Configuration

Agents can be granted access to skills that provide specialized capabilities:

```toml
[[agent]]
name = "developer"
schedule = "0 9 * * 1-5"
prompt = "Work on pending tasks."
skills = ["frontend-design", "code-analysis"]
```

See [skills.md](skills.md) for more information on creating and using skills.

---

## Validation

Always validate your configuration before running:

```bash
# Validate configuration
switchboard validate switchboard.toml

# Validate with custom path
switchboard validate -c path/to/config.toml
```

Common validation errors:
- Missing required fields (`name`, `schedule`, `prompt`)
- Invalid cron expressions
- Both `prompt` and `prompt_file` specified (choose one)
- Invalid timeout format
- Duplicate agent names

---

## Next Steps

- [Installation Guide](installation.md) - Set up switchboard
- [Skills Documentation](skills.md) - Create custom agent skills
- [Discord Integration](discord.md) - Configure the Discord concierge
- [Troubleshooting](troubleshooting.md) - Common issues and solutions
