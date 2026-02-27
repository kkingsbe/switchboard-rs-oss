# Discord Concierge Integration

The Discord Concierge is an AI-powered conversational bot that lives in your Discord server, providing a natural-language interface to your multi-agent AI development system.

## What is Discord Concierge?

Discord Concierge is a built-in Switchboard feature that runs a conversational AI agent directly in a Discord channel. Unlike traditional chatbots, it has access to your project's development workflow and can:

- **File bugs and tasks** into the agent inbox for triage
- **Check system status** — agent states, TODO progress, signal files
- **Relay agent updates** from the outbox to Discord
- **Browse project files** — read source code, backlogs, and documentation
- **Add items to the backlog** for future sprints

### Use Cases

| Use Case | Description |
|----------|-------------|
| **Automated Support** | Team members can ask questions about the project status without interrupting development |
| **Quick Bug Filing** | Report bugs directly from Discord that get picked up by agents automatically |
| **Status Dashboard** | Real-time visibility into what's happening with your AI agents |
| **Team Assistance** | Answer common questions about codebase structure, TODOs, and backlog items |

## Setup

### Step 1: Create a Discord Application

1. Go to [Discord Developer Portal](https://discord.com/developers/applications)
2. Click **New Application** and give it a name
3. Navigate to the **Bot** section on the left sidebar
4. Click **Reset Token** to get your bot token (copy it immediately — you won't see it again)

### Step 2: Configure Bot Permissions

1. In the Developer Portal, go to **Bot** → **Privileged Gateway Intents**
2. Enable **Message Content Intent** (required for reading messages)
3. Go to **OAuth2** → **URL Generator**
4. Select the following scopes:
   - `bot`
5. Select the following permissions:
   - `Send Messages`
   - `Read Message History`
6. Copy the generated URL and open it in your browser to invite the bot to your server

### Step 3: Get Channel ID

1. Open Discord and enable Developer Mode (Settings → Advanced → Developer Mode)
2. Right-click on the channel where you want the bot to operate
3. Click **Copy Channel ID**

### Step 4: Configure Switchboard

Add the Discord configuration to your [`switchboard.toml`](configuration.md):

```toml
[discord]
enabled = true
token_env = "DISCORD_TOKEN"
channel_id = "YOUR_CHANNEL_ID"

[discord.llm]
provider = "openrouter"
api_key_env = "OPENROUTER_API_KEY"
model = "anthropic/claude-sonnet-4"
max_tokens = 1024

[discord.conversation]
max_history = 30
ttl_minutes = 120
```

### Step 5: Set Environment Variables

Create a `.env` file or export these variables in your shell:

```bash
# Required
export DISCORD_TOKEN="your-bot-token-here"
export OPENROUTER_API_KEY="your-openrouter-api-key-here"
```

> **Note:** Get your OpenRouter API key at [openrouter.ai](https://openrouter.ai)

### Step 6: Start Switchboard

```bash
switchboard up
```

You should see output indicating the Discord concierge connected:

```
Scheduler started (3 agents configured)
Discord concierge connected to #channel-name
```

## Configuration Reference

### `[discord]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `false` | Enable/disable Discord integration |
| `token_env` | `String` | `"DISCORD_TOKEN"` | Environment variable name for bot token |
| `channel_id` | `String` | required | Discord channel ID to listen on |
| `intents` | `Integer` | `21504` | Gateway intents (rarely need to change) |

### `[discord.llm]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | `String` | `"openrouter"` | LLM provider (only `openrouter` for v1) |
| `api_key_env` | `String` | `"OPENROUTER_API_KEY"` | Environment variable for API key |
| `model` | `String` | `"anthropic/claude-sonnet-4"` | Model identifier |
| `max_tokens` | `u32` | `1024` | Maximum tokens per response |
| `system_prompt_file` | `Option<String>` | `None` | Custom system prompt file path |

### `[discord.conversation]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_history` | `usize` | `30` | Maximum messages per conversation |
| `ttl_minutes` | `u64` | `120` | Conversation expiration time |

## Available Tools

The Discord concierge has access to the following tools:

### Bug and Task Management

| Tool | Description |
|------|-------------|
| `file_bug` | File a bug report into `comms/inbox/` with title, description, and severity |
| `file_task` | File a task or feature request into `comms/inbox/` |
| `add_to_backlog` | Append items to `BACKLOG.md` |

### Status and Information

| Tool | Description |
|------|-------------|
| `get_status` | Check agent status, signal files, and TODO progress |
| `list_inbox` | List pending items in `comms/inbox/` |
| `read_outbox` | Read and relay agent messages from `comms/outbox/` |
| `read_todos` | Read TODO file progress |
| `read_backlog` | Read `BACKLOG.md` contents |

### File Operations

| Tool | Description |
|------|-------------|
| `read_file` | Read any project file (path traversal protected) |
| `list_directory` | List directory contents |

## Security Considerations

The Discord concierge is designed with security in mind:

- **Path Traversal Protection**: The `read_file` and `list_directory` tools reject paths containing `..` or resolving outside the workspace root
- **Limited Write Access**: The bot can only write to `comms/inbox/` and append to `BACKLOG.md` — it cannot modify source code
- **Token Security**: Discord tokens and API keys are read from environment variables and never logged or written to files
- **Conversation Isolation**: Each user gets their own conversation context that expires after inactivity

## Common Issues

### Bot Not Responding

| Cause | Solution |
|-------|----------|
| Wrong channel ID | Verify the `channel_id` in your config matches the Discord channel |
| Missing intents | Ensure **Message Content Intent** is enabled in Discord Developer Portal |
| Bot not in server | Re-invite the bot using the OAuth2 URL generator |
| Environment variables not set | Verify `DISCORD_TOKEN` and `OPENROUTER_API_KEY` are exported |

### Permission Errors

| Error | Solution |
|-------|----------|
| `Missing Access` | Bot needs to be in the specified channel with proper permissions |
| `Cannot send messages` | Ensure bot has "Send Messages" permission in the channel |
| `400 Bad Request` | Usually indicates missing Message Content intent verification |

To fix permission issues:
1. Go to Discord Developer Portal → Your Bot → OAuth2 → URL Generator
2. Regenerate the invite URL with all required permissions
3. Re-invite the bot

### Rate Limiting

Discord has rate limits on messages (5 per second). The concierge automatically:
- Adds small delays between message chunks
- Respects `X-RateLimit-Remaining` headers
- Waits on 429 responses before retrying

If you see rate limiting errors:
- Wait a moment and try again
- Reduce conversation frequency
- Check [Discord's API status](https://discordstatus.com/)

### LLM Errors

| Error | Solution |
|-------|----------|
| `401 Unauthorized` | Check your `OPENROUTER_API_KEY` is valid |
| `429 Rate Limited` | Wait and retry — OpenRouter has rate limits |
| `Timeout` | Try a simpler question or increase `max_tokens` |
| `Invalid model` | Verify the model name in `[discord.llm]` section |

## Example Interactions

```
User:  hey, the config parser crashes when you pass an empty string
Bot:   Filed that as a bug — "Config parser crash on empty timeout string" 
       with high severity. It's in the inbox for the next triage cycle.

User:  what's going on right now?
Bot:   Dev agents 1 and 2 are idle. 4 items in the inbox waiting for triage.
       No active runs at the moment.

User:  can you add webhook support to the backlog?
Bot:   Done — added [FEATURE] Webhook trigger support to BACKLOG.md.
```

## Next Steps

- [Configuration](configuration.md) — Full TOML reference
- [Skills](skills.md) — Extending agent capabilities
- [Troubleshooting](troubleshooting.md) — Common issues and solutions
