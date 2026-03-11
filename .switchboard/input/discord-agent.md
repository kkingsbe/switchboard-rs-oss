# Discord Concierge Integration — Design Document

## 1. Overview

Switchboard gains a built-in Discord concierge: a conversational LLM agent that lives in a Discord channel, listens to all messages (except its own), and provides a natural-language interface to the multi-agent system. It can file bugs, create tasks, check status, relay agent updates, and answer questions about the project — all through Discord chat.

No separate processes, no HTTP glue, no discli. It's a native Switchboard subsystem that starts with `switchboard up`.

## 2. User Experience

### Setup

```toml
# switchboard.toml

[discord]
enabled = true
token_env = "DISCORD_TOKEN"
channel_id = "1474550134388949272"

[discord.llm]
provider = "openrouter"
api_key_env = "OPENROUTER_API_KEY"
model = "anthropic/claude-sonnet-4"
max_tokens = 1024
system_prompt_file = "prompts/concierge.md"   # optional

[discord.conversation]
max_history = 30
ttl_minutes = 120
```

```bash
# .env or shell exports
export DISCORD_TOKEN="your-bot-token"
export OPENROUTER_API_KEY="your-openrouter-key"

switchboard up
# → Scheduler started (3 agents configured)
# → Discord concierge connected to #dev-agents
```

### Interaction

```
User:  hey, the config parser crashes when you pass an empty string for timeout
Bot:   Filed that as a bug — "Config parser crash on empty timeout string" with
       high severity. It's in the inbox, the next triage cycle will pick it up.

User:  what's going on right now?
Bot:   QA is running (started ~20 min ago). Dev agents 1 and 2 are idle — sprint
       complete. 4 items in the inbox waiting for triage. Outbox is empty.

User:  can you add a feature request to the backlog for webhook support?
Bot:   Done — added [FEATURE] Webhook trigger support to BACKLOG.md.

User:  what does the scheduler module look like?
Bot:   The scheduler is in src/scheduler/mod.rs. It's currently a stub — just the
       module structure with a placeholder start() function. Sprint 2 is when it
       gets implemented per the PRD.
```

## 3. Architecture

```
switchboard up
  │
  ├── Scheduler Task (existing)
  │     └── cron loop → spawn Docker containers
  │
  ├── Discord Listener Task (new)
  │     ├── connects via Discord Gateway WebSocket
  │     ├── receives MessageCreate events
  │     ├── filters: ignore own messages, only configured channel
  │     ├── loads/creates per-user conversation
  │     ├── calls LLM via OpenRouter (tool-use loop)
  │     ├── executes tools against repo filesystem
  │     └── sends response to Discord channel
  │
  └── Outbox Poller Task (new)
        ├── checks comms/outbox/ every 60 seconds
        ├── sends new files to Discord channel
        └── moves sent files to comms/archive/
```

### Module Structure

```
src/
├── discord/
│   ├── mod.rs            # Public API: start_discord_listener()
│   ├── config.rs         # DiscordConfig, LlmConfig, ConversationConfig
│   ├── gateway.rs        # Discord WebSocket gateway (connect, heartbeat, reconnect)
│   ├── api.rs            # Discord REST API (send_message, with chunking for >2000 chars)
│   ├── listener.rs       # Message event handler, routing, response dispatch
│   ├── conversation.rs   # Per-user conversation state (history, trimming, TTL)
│   ├── llm.rs            # OpenRouter client (chat completion with tool-use loop)
│   ├── tools.rs          # Tool definitions (JSON schema) + implementations
│   └── outbox.rs         # Periodic outbox → Discord relay
```

## 4. Configuration

### `[discord]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | `bool` | `false` | Enable/disable the Discord integration |
| `token_env` | `String` | `"DISCORD_TOKEN"` | Name of env var containing the bot token |
| `channel_id` | `String` | required | Discord channel ID to listen on |

### `[discord.llm]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider` | `String` | `"openrouter"` | LLM provider (only `openrouter` for v1) |
| `api_key_env` | `String` | `"OPENROUTER_API_KEY"` | Name of env var containing the API key |
| `model` | `String` | `"anthropic/claude-sonnet-4"` | Model identifier |
| `max_tokens` | `u32` | `1024` | Max tokens per LLM response |
| `system_prompt_file` | `Option<String>` | `None` | Path to custom system prompt (uses built-in default if omitted) |

### `[discord.conversation]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_history` | `usize` | `30` | Max messages per user conversation |
| `ttl_minutes` | `u64` | `120` | Expire conversations after N minutes of inactivity |

## 5. Discord Gateway

Use raw WebSocket rather than a full framework like serenity — keeps dependencies lighter and gives us control over exactly what events we handle.

### Connection Flow

1. GET `https://discord.com/api/v10/gateway/bot` → get WebSocket URL
2. Connect via `tokio-tungstenite`
3. Receive Hello (opcode 10) → extract `heartbeat_interval`
4. Send Identify (opcode 2) with token + intents (`GUILD_MESSAGES | MESSAGE_CONTENT`)
5. Start heartbeat loop (send opcode 1 every `heartbeat_interval` ms)
6. Listen for Dispatch events (opcode 0)

### Events We Care About

| Event | Action |
|-------|--------|
| `READY` | Store bot's own user ID (for self-message filtering) |
| `MESSAGE_CREATE` | Process if `channel_id` matches config AND `author.id != bot_user_id` |

### Reconnection

On disconnect or missed heartbeat ACK:
1. Attempt resume (opcode 6) with stored `session_id` and `seq`
2. If resume fails, do fresh Identify
3. Exponential backoff on repeated failures (1s, 2s, 4s, ... max 60s)

### Dependencies

- `tokio-tungstenite` — WebSocket client (already async-native with tokio)
- `serde_json` — parse gateway payloads (already a dep)
- `reqwest` — REST API calls (already a dep)

No need for `serenity` or `twilight` — we only need one channel, one event type.

## 6. Discord REST API

### Send Message

```
POST https://discord.com/api/v10/channels/{channel_id}/messages
Authorization: Bot {token}
Content-Type: application/json

{"content": "message text"}
```

### Message Chunking

Discord has a 2000-char limit. If the LLM response exceeds this:
1. Split on paragraph boundaries (`\n\n`) where possible
2. Fall back to splitting on newlines (`\n`)
3. Last resort: split at 1990 chars with `…` continuation marker
4. Send chunks sequentially with a small delay (250ms) to maintain order

### Rate Limiting

Respect `X-RateLimit-Remaining` and `X-RateLimit-Reset` headers. If rate limited (429), wait for `Retry-After` seconds before retrying.

## 7. LLM Integration (OpenRouter)

### Request Format

OpenRouter uses the OpenAI-compatible chat completions API:

```
POST https://openrouter.ai/api/v1/chat/completions
Authorization: Bearer {api_key}
Content-Type: application/json

{
  "model": "anthropic/claude-sonnet-4",
  "max_tokens": 1024,
  "messages": [...],
  "tools": [...]
}
```

### Tool-Use Loop

```
loop:
  1. Send messages + tools to OpenRouter
  2. Parse response
  3. If response contains tool_calls:
     a. Execute each tool call
     b. Append assistant message (with tool_calls) to history
     c. Append tool results to history
     d. Continue loop
  4. If response is a text completion:
     a. Extract text content
     b. Append to history
     c. Break — return text as Discord reply
  5. Safety: break after 10 iterations (prevent infinite loops)
```

### Error Handling

| Error | Action |
|-------|--------|
| 401 Unauthorized | Log error, reply "⚠️ LLM API key is invalid. Check your OPENROUTER_API_KEY." |
| 429 Rate Limited | Wait `Retry-After`, retry once, then reply "⚠️ Rate limited, try again shortly." |
| 500+ Server Error | Retry once after 2s, then reply "⚠️ LLM provider is having issues." |
| Timeout (30s) | Reply "⚠️ Response timed out. Try a simpler question." |
| Malformed response | Log full response, reply "⚠️ Got an unexpected response from the LLM." |

## 8. Conversation Management

### State Structure

```rust
struct Conversation {
    user_id: String,
    messages: Vec<ChatMessage>,    // OpenAI-format messages
    last_active: Instant,
}

struct ConversationManager {
    conversations: HashMap<String, Conversation>,
    max_history: usize,
    ttl: Duration,
}
```

### Storage

In-memory only for v1. Conversations are lost on restart — this is fine because:
- The concierge is stateless in terms of side effects (all state lives in files)
- Users can re-explain context easily
- Avoids serialization complexity

### Trimming Strategy

When `messages.len() > max_history`:
1. Always keep the system prompt (message 0) — this is injected at call time, not stored
2. Drop oldest user/assistant pairs from the front
3. Keep the most recent `max_history` messages

### TTL Cleanup

A background task runs every 5 minutes, removing conversations where
`now - last_active > ttl`. This prevents unbounded memory growth from
users who chat once and never return.

## 9. Tools

### Tool Definitions

All tools use the OpenAI function-calling JSON schema format.

#### `file_bug`
File a bug report into `comms/inbox/`.

```json
{
  "name": "file_bug",
  "description": "File a bug report into the agent inbox for triage and fixing",
  "parameters": {
    "type": "object",
    "properties": {
      "title": { "type": "string", "description": "Short bug title" },
      "description": { "type": "string", "description": "Detailed description including steps to reproduce" },
      "severity": { "type": "string", "enum": ["critical", "high", "medium", "low"], "description": "Bug severity" }
    },
    "required": ["title", "description"]
  }
}
```

**Implementation:** Creates `comms/inbox/YYYY-MM-DD_HHMMSS_discord_bug_<slug>.md` with standardized header.

#### `file_task`
File a task or feature request into `comms/inbox/`.

```json
{
  "name": "file_task",
  "description": "File a task or feature request into the agent inbox",
  "parameters": {
    "type": "object",
    "properties": {
      "title": { "type": "string" },
      "description": { "type": "string" },
      "priority": { "type": "string", "enum": ["high", "medium", "low"] }
    },
    "required": ["title", "description"]
  }
}
```

**Implementation:** Creates `comms/inbox/YYYY-MM-DD_HHMMSS_discord_task_<slug>.md`.

#### `get_status`
Check agent status, signal files, and TODO progress.

```json
{
  "name": "get_status",
  "description": "Check the current status of all agents — signal files, TODO progress, inbox/outbox counts",
  "parameters": { "type": "object", "properties": {} }
}
```

**Implementation:** Reads signal files (`.qa_in_progress`, `.sprint_complete`, etc.), counts TODO items (`[x]` vs `[ ]`), counts inbox/outbox files.

#### `list_inbox`
List pending items in `comms/inbox/`.

```json
{
  "name": "list_inbox",
  "description": "List all pending items in the agent inbox",
  "parameters": { "type": "object", "properties": {} }
}
```

#### `read_outbox`
Read and archive outbox messages (relays agent updates to user).

```json
{
  "name": "read_outbox",
  "description": "Read pending agent messages from comms/outbox/, relay them, and archive",
  "parameters": { "type": "object", "properties": {} }
}
```

**Implementation:** Reads all `.md` files in `comms/outbox/`, returns their content, moves them to `comms/archive/`.

#### `read_todos`
Read TODO progress for agents.

```json
{
  "name": "read_todos",
  "description": "Read TODO file progress for a specific agent or all agents",
  "parameters": {
    "type": "object",
    "properties": {
      "agent": { "type": "string", "description": "Agent name/number, or 'all'" }
    }
  }
}
```

#### `read_backlog`
Read BACKLOG.md.

```json
{
  "name": "read_backlog",
  "description": "Read the current project backlog",
  "parameters": { "type": "object", "properties": {} }
}
```

#### `add_to_backlog`
Append an item to BACKLOG.md.

```json
{
  "name": "add_to_backlog",
  "description": "Add a new item to BACKLOG.md",
  "parameters": {
    "type": "object",
    "properties": {
      "item": { "type": "string", "description": "The backlog item description" },
      "tag": { "type": "string", "enum": ["FEATURE", "BUG", "CHORE", "IDEA"] }
    },
    "required": ["item"]
  }
}
```

#### `read_file`
Read any file in the repo.

```json
{
  "name": "read_file",
  "description": "Read a file from the project repository",
  "parameters": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "Relative path from repo root" }
    },
    "required": ["path"]
  }
}
```

**Implementation:** Reads file content, truncates at 3000 chars. For directories, returns listing. Rejects paths that escape the repo root (`..`).

#### `list_directory`
List directory contents.

```json
{
  "name": "list_directory",
  "description": "List files and subdirectories in a project directory",
  "parameters": {
    "type": "object",
    "properties": {
      "path": { "type": "string", "description": "Relative path from repo root (use '.' for root)" }
    },
    "required": ["path"]
  }
}
```

### Security

- **Path traversal prevention**: All `read_file` / `list_directory` paths are canonicalized and verified to be within the workspace root. Reject anything containing `..` or resolving outside the repo.
- **No write tools beyond inbox/backlog**: The concierge cannot modify source code, TODOs, or agent prompts. It can only write to `comms/inbox/` and append to `BACKLOG.md`.
- **Token never logged**: The Discord token and API key are read from env vars and never written to logs or files.

## 10. Outbox Auto-Relay

A background tokio task that runs alongside the listener:

```
every 60 seconds:
  1. Scan comms/outbox/ for .md files
  2. For each file:
     a. Read content
     b. Format: "📬 **Agent Update** — `{filename}`\n\n{content}"
     c. Send to Discord channel
     d. Move to comms/archive/
  3. If no files, do nothing
```

This means agents don't need any changes — they write to outbox as they always have, and updates appear in Discord automatically.

## 11. Default System Prompt

Built into the binary, overridable via `system_prompt_file`:

```markdown
You are the Switchboard Concierge — a friendly assistant embedded in Discord that
helps users interact with a multi-agent AI development system.

## What you can do
- **File bugs and tasks** into the agent inbox (they'll be picked up by triage)
- **Check system status** — which agents are running, TODO progress, signal states
- **Read agent updates** from the outbox and relay them
- **Browse project files** — source code, TODOs, backlogs, bug reports
- **Add items to the backlog** for future sprints

## How the system works
The project uses coordinated AI agents that communicate via files:
- `TODO*.md` files are agent work queues
- `comms/inbox/` receives new bugs and tasks from you
- `comms/outbox/` contains agent messages for you
- Signal files (`.sprint_complete`, `.qa_complete`, etc.) coordinate agent phases
- `BACKLOG.md` holds future work items
- `BUGS.md` contains the latest QA report

## Guidelines
- Be conversational and concise — this is Discord, not a report
- Keep responses under 1500 characters when possible
- When filing bugs, extract a clear title and description from the user's message
- When checking status, give a quick summary, not a data dump
- Use tools proactively — if someone describes a bug, file it; don't just acknowledge
- If you're not sure what the user wants, ask — don't guess wrong
```

## 12. Integration with `switchboard up`

In `src/cli/mod.rs`, the `up` command handler gains:

```rust
// After starting the scheduler...
if config.discord.enabled {
    let discord_config = config.discord.clone();
    let workspace = config.settings.workspace_path.clone();

    tokio::spawn(async move {
        if let Err(e) = discord::start(discord_config, workspace).await {
            tracing::error!("Discord listener failed: {}", e);
        }
    });

    tracing::info!("Discord concierge connected to channel {}", config.discord.channel_id);
}
```

The Discord listener runs as a peer task to the scheduler. If it crashes, it logs the error but doesn't take down the scheduler. The gateway reconnection logic handles transient failures.

## 13. Cargo Dependencies

New additions to `Cargo.toml`:

```toml
[dependencies]
# Existing deps...

# Discord gateway
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
futures-util = "0.3"           # for StreamExt on WebSocket

# Already present but confirm:
# reqwest (for Discord REST + OpenRouter API)
# serde, serde_json (for payload parsing)
# tokio (async runtime)
# tracing (logging)
```

That's it — no heavy Discord framework needed.

## 14. Testing Strategy

| Component | Test Approach |
|---|---|
| `config.rs` | Unit tests — parse valid/invalid TOML with `[discord]` section |
| `tools.rs` | Unit tests — create temp dirs, exercise each tool, verify file output |
| `conversation.rs` | Unit tests — add messages, verify trimming, verify TTL expiry |
| `llm.rs` | Unit tests with mock HTTP — verify tool-use loop terminates, handles errors |
| `api.rs` | Unit tests — verify message chunking logic for >2000 char messages |
| `gateway.rs` | Integration test — mock WebSocket server, verify connect/heartbeat/reconnect |
| `listener.rs` | Integration test — end-to-end with mock gateway + mock LLM |

## 15. Effort Estimate

| Component | Days | Notes |
|---|---|---|
| Config parsing (`config.rs`) | 0.5 | Extends existing TOML parser |
| Discord gateway (`gateway.rs`) | 2 | WebSocket lifecycle, heartbeat, reconnect |
| Discord REST API (`api.rs`) | 0.5 | Send message, chunking, rate limits |
| Message listener (`listener.rs`) | 1 | Event routing, self-filter, response dispatch |
| OpenRouter LLM client (`llm.rs`) | 1.5 | Chat completions, tool-use loop, error handling |
| Tool implementations (`tools.rs`) | 1 | ~10 tools, all filesystem ops |
| Conversation manager (`conversation.rs`) | 0.5 | History, trimming, TTL cleanup |
| Outbox poller (`outbox.rs`) | 0.5 | Periodic scan + relay |
| Integration into `switchboard up` | 0.5 | Spawn tasks, graceful shutdown |
| System prompt tuning | 0.5 | Iteration on behavior |
| Tests | 1.5 | Unit + integration |
| **Total** | **~10 days** | ~2 weeks with buffer |

## 16. Future Extensions (Out of Scope for v1)

- **Multiple channels**: Listen on multiple channels with different tool sets
- **Persistent conversations**: Serialize to SQLite for cross-restart continuity
- **Image attachments**: Let users attach screenshots to bug reports
- **Threaded replies**: Reply in Discord threads to keep the channel clean
- **Additional LLM providers**: Direct Anthropic API, OpenAI, local models via LM Studio
- **Per-user permissions**: Restrict who can file bugs vs. read-only status checks
- **Typing indicator**: Show "bot is typing..." while the LLM processes
- **Reaction-based confirmations**: React with ✅ when a bug is filed