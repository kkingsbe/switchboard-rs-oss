# Discord Agent Feature - Backlog

## Configuration
- [ ] Parse `[discord]` section from switchboard.toml
- [ ] Create DiscordConfig struct (enabled, token_env, channel_id)
- [ ] Create LlmConfig struct (provider, api_key_env, model, max_tokens, system_prompt_file)
- [ ] Create ConversationConfig struct (max_history, ttl_minutes)
- [ ] Add env var loading for Discord token and LLM API key

## Discord Gateway
- [ ] Implement gateway URL retrieval from Discord API
- [ ] Create WebSocket connection with tokio-tungstenite
- [ ] Handle Hello (opcode 10) and extract heartbeat_interval
- [ ] Implement Identify (opcode 2) with intents (GUILD_MESSAGES | MESSAGE_CONTENT)
- [ ] Implement heartbeat loop (opcode 1)
- [ ] Handle MESSAGE_CREATE events, filter by channel and bot user ID
- [ ] Handle READY event, store bot user ID
- [ ] Implement reconnection with resume (opcode 6) and exponential backoff

## Discord REST API
- [ ] Implement send_message to channel endpoint
- [ ] Implement message chunking for >2000 chars (split on paragraphs, newlines, or 1990 char limit)
- [ ] Implement rate limiting with X-RateLimit-Remaining and X-RateLimit-Reset headers
- [ ] Handle 429 responses with Retry-After

## LLM Integration
- [ ] Create OpenRouter client struct
- [ ] Implement chat completion request format
- [ ] Implement tool-use loop with max 10 iterations
- [ ] Handle LLM errors (401, 429, 500+, timeout, malformed response)
- [ ] Implement conversation history trimming
- [ ] Implement conversation TTL cleanup task (every 5 minutes)

## Tools Implementation
- [ ] Define file_bug tool (JSON schema + implementation to comms/inbox/)
- [ ] Define file_task tool (JSON schema + implementation to comms/inbox/)
- [ ] Define get_status tool (JSON schema + implementation - read signal files, TODO counts, inbox/outbox counts)
- [ ] Define list_inbox tool (JSON schema + implementation)
- [ ] Define read_outbox tool (JSON schema + implementation - read and archive to comms/archive/)
- [ ] Define read_todos tool (JSON schema + implementation)
- [ ] Define read_backlog tool (JSON schema + implementation)
- [ ] Define add_to_backlog tool (JSON schema + implementation - append to BACKLOG.md)
- [ ] Define read_file tool with path traversal prevention (JSON schema + implementation)
- [ ] Define list_directory tool (JSON schema + implementation)

## Outbox Poller
- [ ] Create outbox poller task (every 60 seconds)
- [ ] Scan comms/outbox/ for .md files
- [ ] Format messages with "📬 **Agent Update**" prefix
- [ ] Relay messages to Discord channel
- [ ] Move sent files to comms/archive/

## Integration
- [ ] Add Discord listener task initialization to switchboard up command
- [ ] Add outbox poller task initialization to switchboard up command
- [ ] Implement graceful shutdown (cancel tasks on SIGINT/SIGTERM)
- [ ] Add default system prompt or load from system_prompt_file
- [ ] End-to-end testing with real Discord bot token
