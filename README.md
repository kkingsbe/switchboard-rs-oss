# 🏙️ Switchboard

**A Rust CLI tool for scheduling AI coding agents via Docker**

![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Version](https://img.shields.io/badge/version-0.1.0-green.svg)
![Status](https://img.shields.io/badge/status-Work%20in%20Progress-yellow.svg)
![codecov](https://codecov.io/gh/your-username/kilocode/branch/main/graph/badge.svg)

---

## Overview

**Switchboard** is a powerful CLI tool that orchestrates scheduled AI coding agent execution via Docker containers. It acts as a cron scheduler for the [Kilo Code CLI](https://www.npmjs.com/package/@kilocode/cli), allowing you to define AI agents with prompts, schedule them using familiar cron expressions, and run each in its own isolated Docker container.

Whether you need automated code reviews, dependency updates, documentation maintenance, or any other AI-powered development workflow, Switchboard provides a reliable, isolated, and schedulable execution environment.

For detailed product requirements, see [`PRD.md`](PRD.md). For architectural details, see [`ARCHITECTURE.md`](ARCHITECTURE.md).

---

## Installation

For detailed installation instructions covering all methods, see the [Installation Guide](docs/INSTALLATION.md).

### Prerequisites

- **Docker** (required): Docker must be installed and running for agent container execution
- **Rust toolchain** (for building from source or installing from crates.io): Minimum Rust 1.70.0 recommended
- **Git** (for cloning the repository): Required for source installation

### Installation from Source

To build and install Switchboard from source:

```bash
git clone https://github.com/your-org/switchboard.git && cd switchboard
cargo install --path .
```

This installs the `switchboard` binary globally on your system. The `cargo install` command builds the project in release mode by default, which provides optimized binaries.

For development purposes, you can build a debug version instead:

```bash
cargo build
```

The debug binary will be located at `target/debug/switchboard`.

### Installation from crates.io

Install Switchboard directly from crates.io with a single command:

```bash
cargo install switchboard
```

For detailed instructions including version selection, updating, and troubleshooting, see the [Installation Guide - Installing from crates.io](docs/INSTALLATION.md#installing-from-cratesio).

### Binary Download

Pre-compiled binaries for Linux, macOS, and Windows will be available for download from the [GitHub Releases](https://github.com/your-org/switchboard/releases) page.

Download the appropriate binary for your platform:

```bash
# Linux
curl -L https://github.com/your-org/switchboard/releases/latest/download/switchboard-linux-x86_64 -o switchboard
chmod +x switchboard
sudo mv switchboard /usr/local/bin/

# macOS
curl -L https://github.com/your-org/switchboard/releases/latest/download/switchboard-darwin-x86_64 -o switchboard
chmod +x switchboard
sudo mv switchboard /usr/local/bin/

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/your-org/switchboard/releases/latest/download/switchboard-windows-x86_64.exe" -OutFile "switchboard.exe"
```

*Note: Binary releases are coming soon. For now, please install from source.*

### Verification

After installation, verify that Switchboard is properly installed by checking the version:

```bash
switchboard --version
```

Expected output:
```
switchboard 0.1.0
```

### Troubleshooting

Having trouble installing or running switchboard? See our [Installation Troubleshooting Guide](docs/INSTALLATION_TROUBLESHOOTING.md) for detailed help. Quick fixes for common issues:

#### Rust Toolchain Issues
**Symptoms:** `rust: command not found` or `cargo: command not found`  
**Solution:** Install Rust from https://rustup.rs/

#### Docker Daemon Issues
**Symptoms:** `Cannot connect to Docker daemon`  
**Solution:** Start Docker Desktop or run `sudo systemctl start docker` (Linux)

#### Permission Denied Errors
**Symptoms:** `Permission denied` when running Docker  
**Solution:** Add user to docker group: `sudo usermod -aG docker $USER` then logout/login

#### Path Not Found Errors
**Symptoms:** `switchboard: command not found` after installation  
**Solution:** Add `~/.cargo/bin` to PATH or run `source $HOME/.cargo/env`

For platform-specific issues, see the [Platform Compatibility](docs/PLATFORM_COMPATIBILITY.md) documentation.

---

## Quick Start

Follow these steps to get Switchboard running with your first agent:

1. **Create a `switchboard.toml` config file:**

Navigate to your project directory and create the configuration file:

```bash
cd ~/my-project
```

Create `switchboard.toml` with a simple agent:

```toml
version = "0.1.0"

[[agent]]
name = "hello-world"
schedule = "0 */6 * * *"           # Every 6 hours
prompt = "Hello! Please review the README.md file and suggest improvements."
```

2. **Validate your configuration:**

```bash
switchboard validate
```

This checks your `switchboard.toml` for syntax errors and validates all agent definitions.

3. **Build the Docker image:**

```bash
switchboard build
```

This builds the agent container image with Kilo Code CLI and required dependencies.

4. **Run a single agent:**

```bash
switchboard run hello-world
```

This executes the agent immediately (ignoring its schedule) so you can verify it works.

5. **Start the scheduler:**

```bash
switchboard up
```

The scheduler starts in the foreground and will trigger your agents according to their schedules. Use `Ctrl+C` to stop it.

### Common First-Time Scenarios

#### 🚀 Scenario 1: Running a One-Time Task (Manual Trigger)

Sometimes you want to execute an agent immediately without waiting for its scheduled time:

```bash
# Run the agent once (ignores schedule)
switchboard run hello-world
```

This is useful for:
- Testing a new agent before scheduling it
- Running on-demand tasks
- Debugging agent behavior

#### 📅 Scenario 2: Running Multiple Agents on Different Schedules

Define multiple agents in your `switchboard.toml` with different schedules:

```toml
version = "0.1.0"

[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"           # Every 6 hours
prompt = "Review recent changes and suggest improvements."

[[agent]]
name = "doc-updater"
schedule = "0 2 * * 1"             # Every Monday at 2 AM
prompt_file = "prompts/update-docs.md"
readonly = true

[[agent]]
name = "security-scan"
schedule = "@daily"                # Once per day
prompt = "Scan for security vulnerabilities."
```

All agents run in parallel according to their schedules.

#### 🔍 Scenario 3: Debugging an Agent with Logs

When an agent isn't behaving as expected, view its logs:

```bash
# View all agent logs
switchboard logs

# View logs for a specific agent
switchboard logs hello-world

# Follow logs in real-time
switchboard logs --follow

# Follow logs for a specific agent
switchboard logs hello-world --follow
```

Logs are stored in the configured log directory (default: `.switchboard/logs/`).

#### 📝 Scenario 4: Using a Prompt File Instead of Inline Prompt

For longer or reusable prompts, use a separate file:

```toml
version = "0.1.0"

[[agent]]
name = "comprehensive-audit"
schedule = "0 3 * * 0"             # Every Sunday at 3 AM
prompt_file = "prompts/full-audit.md"
readonly = true
timeout = "2h"
```

Create the prompt file at `prompts/full-audit.md`:

```markdown
# Full Repository Audit

Please perform a comprehensive audit of this codebase:

1. Check for security vulnerabilities
2. Review code quality and consistency
3. Identify areas for improvement
4. Suggest refactoring opportunities

Provide a detailed report with actionable recommendations.
```

---

## CLI Commands

Switchboard provides the following CLI commands:

### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <PATH>` | Path to the configuration file (default: `./switchboard.toml`) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

### Core Commands

#### `switchboard up`

Build agent image and start the scheduler.

```bash
# Start the scheduler in the foreground
switchboard up

# Start the scheduler in the background
switchboard up --detach
```

| Flag | Description |
|------|-------------|
| `--detach`, `-d` | Run the scheduler in the background |

#### `switchboard run <AGENT_NAME>`

Immediately execute a single agent, bypassing the scheduler.

```bash
switchboard run hello-world
switchboard run my-agent
switchboard run security-scanner
```

| Argument | Description |
|----------|-------------|
| `AGENT_NAME` | Name of the agent to execute (required) |

#### `switchboard build`

Build or rebuild the agent Docker image.

```bash
switchboard build
```

#### `switchboard list`

Print all configured agents, their schedules, and prompts.

```bash
switchboard list
```

#### `switchboard logs [AGENT_NAME]`

View logs from agent runs.

```bash
# View all agent logs
switchboard logs

# View logs for a specific agent
switchboard logs hello-world

# Follow logs in real-time
switchboard logs --follow

# Follow logs for a specific agent
switchboard logs hello-world --follow

# Show last N lines
switchboard logs --tail 100
switchboard logs hello-world --tail 200
```

| Argument | Description |
|----------|-------------|
| `AGENT_NAME` | Name of the agent to view logs for (optional) |

| Flag | Description |
|------|-------------|
| `--follow`, `-f` | Stream logs as they are generated |
| `--tail` | Show the last N lines (default: 50) |

#### `switchboard metrics`

Display agent execution metrics.

```bash
# Show metrics summary
switchboard metrics

# Show detailed metrics
switchboard metrics --detailed

# Show metrics for a specific agent
switchboard metrics --agent hello-world
```

| Flag | Description |
|------|-------------|
| `--detailed` | Show detailed metrics view |
| `--agent <NAME>` | Show detailed metrics for a specific agent |
| `-c, --config <PATH>` | Path to configuration file |

#### `switchboard down`

Stop the scheduler and any running agent containers.

```bash
# Stop scheduler and containers
switchboard down

# Stop and clean up .switchboard directory
switchboard down --cleanup
```

| Flag | Description |
|------|-------------|
| `-c, --cleanup` | Clean up .switchboard directory (logs, PID files, etc.) |

#### `switchboard validate`

Parse and validate the configuration file.

```bash
switchboard validate
```

#### `switchboard status`

Check scheduler health and status.

```bash
switchboard status
```

---

## Skills

Switchboard integrates with [skills.sh](https://skills.sh) to provide Agent Skills — reusable procedural knowledge that extends your AI agents' capabilities. Skills are installed inside each agent's container at startup, giving agents specialized knowledge for tasks like security auditing, frontend design, code review, and more.

### Overview

Agent Skills are a lightweight, open standard for extending AI agent behavior. The skills.sh marketplace, created by Vercel Labs, aggregates community-authored skills installable via the `npx skills` CLI. Switchboard provides a first-class `switchboard skills` CLI experience and allows you to declare skills per-agent in your `switchboard.toml`.

**Key features:**
- Browse and search skills from the terminal with `switchboard skills list`
- Install skills with `switchboard skills install`
- Declare per-agent skills in `switchboard.toml` for automatic provisioning
- Skills are installed inside each agent's container at startup — no host-side Node.js required

### Configuring Skills in switchboard.toml

Add the `skills` field to any `[[agent]]` section to declare which skills that agent should have available:

```toml
[[agent]]
name = "security-scan"
schedule = "0 2 * * *"
prompt = "Scan for security vulnerabilities."
skills = ["security-audit"]

[[agent]]
name = "ui-reviewer"
schedule = "0 9 * * 1"
prompt = "Review UI components."
skills = [
    "vercel-labs/agent-skills@frontend-design",
    "anthropics/skills@code-review"
]
```

#### Skill Definition Format

Skills are specified as an array of skill sources in one of these formats:

| Format | Description | Example |
|--------|-------------|---------|
| `owner/repo` | Install all skills from a repository | `vercel-labs/agent-skills` |
| `owner/repo@skill-name` | Install a specific skill from a repository | `vercel-labs/agent-skills@frontend-design` |

**Rules:**
- If `skills` is omitted, the agent receives no skills
- Skills are installed inside the container at startup (not on the host)
- Each agent installs only its declared skills — different agents can have different skill sets
- A failed skill installation aborts the agent run and is logged

### Switchboard Skills CLI Commands

Switchboard provides a `skills` subcommand family for managing skills:

#### `switchboard skills list`

Browse available skills from skills.sh:

```bash
# Interactive skill browser
switchboard skills list

# Search for skills matching a term
switchboard skills list --search react
```

#### `switchboard skills install`

Install a skill into your project:

```bash
# Install a specific skill
switchboard skills install vercel-labs/agent-skills@frontend-design

# Install all skills from a repository
switchboard skills install vercel-labs/agent-skills

# Install globally (available to all projects)
switchboard skills install --global vercel-labs/agent-skills@security-audit
```

#### `switchboard skills installed`

List all skills currently installed in your project:

```bash
switchboard skills installed
```

#### `switchboard skills remove`

Remove an installed skill:

```bash
switchboard skills remove frontend-design

# Remove without confirmation
switchboard skills remove frontend-design --yes
```

#### `switchboard skills update`

Update installed skills to their latest versions:

```bash
# Update all skills
switchboard skills update

# Update a specific skill
switchboard skills update frontend-design
```

### Prerequisites

- **Inside the container**: The Docker image includes Node.js (`node:22-slim`), so skills are installed automatically at runtime
- **On the host**: `npx` and Node.js are only required for the `switchboard skills list`, `install`, and `update` commands. If you only declare skills in `switchboard.toml` and use `switchboard run`, no host-side Node.js is needed.

### Example: Security Scanning Agent

Here's a complete example of an agent configured with security scanning skills:

```toml
[[agent]]
name = "security-scanner"
schedule = "0 2 * * *"        # Every day at 2 AM
prompt = """
Perform a comprehensive security scan of the codebase.
Focus on:
1. Known vulnerability patterns
2. Hardcoded secrets or API keys
3. Insecure dependency versions
4. SQL injection risks
5. XSS vulnerabilities

Report findings with severity levels and remediation steps.
"""
readonly = true               # Read-only for security analysis
timeout = "1h"
skills = [
    "anthropics/skills@security-audit",
    "vercel-labs/agent-skills@secret-detection"
]
```

### Troubleshooting Skills Issues

| Issue | Solution |
|-------|----------|
| `npx: command not found` | Install Node.js from https://nodejs.org |
| Skill installation fails inside container | Check container logs with `switchboard logs <agent-name>` |
| Agent can't find skill | Ensure skill is listed in the agent's `skills` field in `switchboard.toml` |

For more details on the skills feature, see [`addtl-features/skills-feature.md`](addtl-features/skills-feature.md).

## Discord Concierge

Switchboard includes a Discord concierge — a conversational LLM agent that lives in a Discord channel, listens to messages, and provides a natural-language interface to the multi-agent system. It can file bugs, create tasks, check status, relay agent updates, and answer questions about the project — all through Discord chat.

### Overview

The Discord concierge is a built-in Switchboard subsystem that starts automatically when you run `switchboard up` with Discord enabled. It connects to a configured Discord channel and responds to user messages using an LLM (via OpenRouter) with access to project tools.

**Key features:**
- Conversational interface in Discord — no CLI needed
- File bugs and tasks directly into the agent inbox
- Check agent status, TODO progress, and inbox/outbox counts
- Read and relay agent messages from the outbox
- Browse and query project files

### Required Environment Variables

To use the Discord concierge, you must set these environment variables in your shell or `.env` file:

| Variable | Description |
|----------|-------------|
| `DISCORD_TOKEN` | Your Discord bot token |
| `OPENROUTER_API_KEY` | Your OpenRouter API key (for LLM calls) |
| `DISCORD_CHANNEL_ID` | The Discord channel ID to listen on |

### Optional Environment Variables

These can be customized in the `[discord]` section of your `switchboard.toml`:

| Variable | Default | Description |
|----------|---------|-------------|
| `DISCORD_LLM_MODEL` | `anthropic/claude-sonnet-4` | LLM model to use |
| `DISCORD_LLM_PROVIDER` | `openrouter` | LLM provider (currently only `openrouter` supported) |
| `DISCORD_MAX_TOKENS` | `1024` | Max tokens per LLM response |
| `DISCORD_MAX_HISTORY` | `30` | Maximum conversation messages per user |
| `DISCORD_TTL_MINUTES` | `120` | Conversation expiration time |
| `DISCORD_SYSTEM_PROMPT_FILE` | (none) | Path to custom system prompt markdown file |

### Enabling Discord in switchboard.toml

Add a `[discord]` section to your `switchboard.toml` configuration file. See the full configuration reference in [`switchboard.sample.toml`](switchboard.sample.toml):

```toml
[discord]
# Enable/disable the Discord concierge integration
enabled = true

# Discord bot token (from environment variable)
token_env = "DISCORD_TOKEN"

# Channel ID to listen on
channel_id = "1474550134388949272"

[discord.llm]
# LLM provider (currently only openrouter is supported)
provider = "openrouter"

# OpenRouter API key environment variable name
api_key_env = "OPENROUTER_API_KEY"

# Model identifier (default: anthropic/claude-sonnet-4)
model = "anthropic/claude-sonnet-4"

# Maximum tokens per LLM response
max_tokens = 1024
```

### Tools Available to the Discord Concierge

| Tool | Description |
|------|-------------|
| `file_bug` | File a bug report into `comms/inbox/` |
| `file_task` | File a task or feature request |
| `get_status` | Check agent status and TODO progress |
| `list_inbox` | List pending items in `comms/inbox/` |
| `read_outbox` | Read and relay agent messages |
| `read_todos` | Read TODO file progress |
| `read_backlog` | Read BACKLOG.md |
| `add_to_backlog` | Add items to BACKLOG.md |
| `read_file` | Read any project file |
| `list_directory` | List directory contents |

For more details on the Discord concierge feature, see [`addtl-features/discord-agent.md`](addtl-features/discord-agent.md).

---

## Troubleshooting

This section helps you resolve common issues when using Switchboard. If you encounter problems not covered here, please [file an issue](https://github.com/yourusername/switchboard/issues) or check the [Installation Troubleshooting Guide](docs/INSTALLATION_TROUBLESHOOTING.md).

### Docker daemon not running errors

If you encounter errors related to Docker not being available, Switchboard cannot execute agents since they run in isolated Docker containers.

**Common error messages:**
```
Error: Docker daemon is not running
Error: Failed to connect to Docker daemon
Error: Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**How to check if Docker is running:**

```bash
# Check if Docker daemon is running
docker ps

# Expected output: List of running containers (or empty table)
# If this fails, Docker is not running
```

**How to start the Docker daemon:**

**macOS:**
- Open **Docker Desktop** from Applications
- Wait for the Docker icon in the menu bar to indicate it's running (steady, not animating)
- Verify with `docker ps`

**Linux:**
```bash
# Start Docker daemon
sudo systemctl start docker

# Enable Docker to start on boot
sudo systemctl enable docker

# Check Docker status
sudo systemctl status docker
```

**Troubleshooting Docker installation issues:**

If `docker ps` fails even after starting the daemon:

1. **Verify Docker installation:**
   ```bash
   docker --version
   docker info
   ```

2. **Check Docker daemon logs (Linux):**
   ```bash
   sudo journalctl -u docker.service -n 50
   ```

3. **Restart Docker daemon (Linux):**
   ```bash
   sudo systemctl restart docker
   ```

4. **For Docker Desktop (macOS/Windows):**
   - Check the Docker Desktop dashboard for error messages
   - Ensure you have sufficient disk space
   - Try restarting Docker Desktop
   - Verify Docker Desktop has proper permissions

5. **User permissions (Linux):**
   ```bash
   # Add your user to the docker group to avoid using sudo
   sudo usermod -aG docker $USER
   
   # Log out and log back in for the group change to take effect
   # Then verify without sudo:
   docker ps
   ```

### Missing .kilocode directory errors

The `.kilocode` directory contains Kilo Code CLI configuration, including API keys and other settings required for agent execution.

**Why this error occurs:**
Switchboard invokes the Kilo Code CLI inside Docker containers, which expects to find configuration in the `.kilocode` directory. If this directory is missing, agents cannot authenticate with AI services.

**Common error messages:**
```
Error: .kilocode directory not found
Error: Failed to locate Kilo Code configuration
Error: Missing .kilocode/config.json
```

**How to locate and verify the .kilocode directory:**

```bash
# Check if .kilocode exists in your home directory
ls -la ~/.kilocode

# Check if it exists in your current project directory
ls -la .kilocode

# Find all .kilocode directories on your system
find ~ -type d -name ".kilocode" 2>/dev/null
```

**How to set up the .kilocode directory structure:**

The `.kilocode` directory typically contains:

```
.kilocode/
├── config.json          # Main configuration file
├── api_keys.json        # API keys for various services
└── logs/                # Optional: local logs
```

**Creating the .kilocode directory:**

```bash
# Create the directory in your home (recommended)
mkdir -p ~/.kilocode/logs

# Or create it in your project directory
mkdir -p .kilocode/logs
```

**Where API keys should be placed:**

API keys should be stored in `~/.kilocode/api_keys.json`:

```json
{
  "anthropic": "your-anthropic-api-key-here",
  "openai": "your-openai-api-key-here",
  "other_provider": "your-other-api-key"
}
```

**Best practices for API keys:**

- Never commit `.kilocode/api_keys.json` to version control
- Add `.kilocode/` to your `.gitignore` file
- Use environment variables as an alternative (see Configuration Reference)
- Rotate API keys regularly for security

**For more details on setting up the .kilocode directory, see the [setup documentation](docs/setup.md) (coming soon).**

### Config validation failures

Switchboard validates your configuration before execution. Validation errors prevent agents from running to avoid unexpected behavior.

**Common validation error messages:**

| Error | Meaning |
|-------|---------|
| `Invalid TOML syntax` | Malformed TOML in switchboard.toml |
| `Missing required field: name` | Agent missing the `name` field |
| `Duplicate agent name: "my-agent"` | Two agents have the same name |
| `Invalid cron expression: "0-60 * * * *"` | Cron syntax error (minute must be 0-59) |
| `Invalid timeout format: "30x"` | Timeout must be in Ns, Nm, or Nh format |
| `Exactly one of 'prompt' or 'prompt_file' must be provided` | Agent missing prompt or has both |
| `Prompt file not found: prompts/missing.md` | Referenced file doesn't exist |
| `Invalid overlap mode: "unknown"` | Must be "skip" or "queue" |

**How to run `switchboard validate`:**

```bash
# Validate default config file
switchboard validate

# Validate custom config file
switchboard validate --config /path/to/switchboard.toml
```

**Common syntax mistakes:**

**1. Missing quotes around strings:**
```toml
# ❌ Invalid
name = my-agent
schedule = 0 */6 * * *

# ✅ Valid
name = "my-agent"
schedule = "0 */6 * * *"
```

**2. Invalid cron expressions:**
```toml
# ❌ Invalid (minute must be 0-59)
schedule = "0-60 * * * *"

# ❌ Invalid (5-field format required)
schedule = "0 * * * * *"

# ✅ Valid
schedule = "0 */6 * * *"  # Every 6 hours
schedule = "*/30 * * * *"  # Every 30 minutes
```

**3. Invalid timeout format:**
```toml
# ❌ Invalid
timeout = "30"
timeout = "30x"
timeout = "30min"

# ✅ Valid (Ns, Nm, Nh format)
timeout = "30s"  # 30 seconds
timeout = "5m"   # 5 minutes
timeout = "1h"   # 1 hour
```

**4. Missing required fields:**
```toml
# ❌ Invalid (missing name)
[[agent]]
schedule = "0 */6 * * *"
prompt = "Review code"

# ✅ Valid
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review code"
```

**5. Duplicate agent names:**
```toml
# ❌ Invalid (duplicate "my-agent" name)
[[agent]]
name = "my-agent"
schedule = "0 */6 * * *"
prompt = "Task 1"

[[agent]]
name = "my-agent"  # Duplicate!
schedule = "0 */6 * * *"
prompt = "Task 2"

# ✅ Valid (unique names)
[[agent]]
name = "agent-1"
schedule = "0 */6 * * *"
prompt = "Task 1"

[[agent]]
name = "agent-2"
schedule = "0 */6 * * *"
prompt = "Task 2"
```

**How to debug TOML syntax issues:**

1. **Use an online TOML validator:**
   - [TOML Lint](https://www.toml-lint.com/)
   - Copy your config and check for syntax errors

2. **Validate with Rust's TOML parser:**
   ```bash
   # If you have Rust installed, you can test TOML parsing
   cargo install toml-cli
   toml-cli parse switchboard.toml
   ```

3. **Check for common issues:**
   - Unmatched brackets `[` or `]`
   - Missing commas in arrays `[[agent]]`
   - Incorrect indentation (though TOML is whitespace-insensitive)
   - Missing newline between sections

4. **Use `switchboard validate` after each change:**
   ```bash
   # Make incremental changes and validate often
   switchboard validate
   ```

**Examples of valid vs invalid configurations:**

**Valid configuration:**
```toml
version = "0.1.0"

[settings]
log_dir = ".switchboard/logs"
timezone = "America/New_York"

[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review recent code changes"
timeout = "30m"

[[agent]]
name = "doc-updater"
schedule = "0 2 * * 1"
prompt_file = "prompts/update-docs.md"
readonly = true
timeout = "1h"
```

**Invalid configuration (with issues marked):**
```toml
version = "0.1.0"

[settings]
log_dir = .switchboard/logs  # ❌ Missing quotes
timezone = "America/New_York"

[[agent]]
# ❌ Missing 'name' field
schedule = "0 */6 * * *"
prompt = "Review recent code changes"
timeout = "30x"  # ❌ Invalid timeout format

[[agent]]
name = "code-reviewer"  # ❌ Duplicate name (if above had one)
schedule = "0 60 * * *"  # ❌ Invalid hour (must be 0-23)
prompt = "Update docs"
prompt_file = "prompts/docs.md"  # ❌ Both prompt and prompt_file
```

### Container timeout issues

Switchboard enforces timeout limits on agent execution to prevent runaway processes and resource exhaustion. Understanding timeout behavior is essential for configuring agents appropriately.

**Explanation of timeout configuration:**

The `timeout` field in `switchboard.toml` specifies the maximum execution time for an agent container. When the timeout is reached, the container is terminated gracefully.

**Default timeout:** `"30m"` (30 minutes)

**Timeout format:** `"Ns"` (seconds), `"Nm"` (minutes), `"Nh"` (hours)

**How to adjust timeout values:**

```toml
# 30 seconds (for quick tasks)
timeout = "30s"

# 5 minutes (default-ish)
timeout = "5m"

# 30 minutes (default)
timeout = "30m"

# 2 hours (for complex tasks)
timeout = "2h"

# No timeout (use with caution - may cause resource exhaustion)
# Note: Switchboard requires a timeout; set a very high value instead
timeout = "168h"  # 1 week
```

**What happens when a timeout is reached:**

1. **Graceful termination attempt:**
   - Switchboard sends `SIGTERM` to the container
   - Container has a brief grace period to clean up

2. **Force termination (if needed):**
   - If the container doesn't stop, Switchboard sends `SIGKILL`
   - Container is forcibly terminated

3. **Cleanup:**
   - Container resources are released
   - Logs are preserved (up to the timeout point)
   - Metrics record the timeout as a failed run

4. **Logging:**
   - Timeout is logged with timestamp
   - Agent metrics show increased "timeout count"
   - Error logged: `Agent timed out after <timeout>`

**How to identify timeout issues in logs:**

```bash
# View logs for timeout messages
switchboard logs my-agent | grep -i timeout

# Or manually inspect log files
cat .switchboard/logs/my-agent-*.log | grep -i timeout
```

**Common timeout log patterns:**

```
[2024-02-15 10:30:45] Agent timed out after 30m
[2024-02-15 10:30:45] Terminating container my-agent-abc123 due to timeout
[2024-02-15 10:30:46] Container my-agent-abc123 stopped (exit code: 137)
```

Exit code 137 typically indicates a `SIGKILL` (timeout termination).

**Recommendations for setting appropriate timeouts:**

Based on agent complexity and task type:

| Agent Type | Suggested Timeout | Reasoning |
|------------|------------------|-----------|
| **Simple code review** | `"5m"` - `"15m"` | Quick analysis of small codebase |
| **Documentation update** | `"10m"` - `"30m"` | Reading + writing files |
| **Security scan** | `"15m"` - `"45m"` | May need deep analysis |
| **Full codebase audit** | `"30m"` - `"2h"` | Large codebase, comprehensive |
| **Refactoring task** | `"30m"` - `"3h"` | Complex analysis + edits |
| **Dependency update** | `"15m"` - `"1h"` | Download + analyze + update |
| **Performance profiling** | `"10m"` - `"1h"` | Execution + analysis |

**Factors to consider:**

1. **Codebase size:**
   - Small projects (<100 files): `"5m"` - `"30m"`
   - Medium projects (100-1000 files): `"15m"` - `"1h"`
   - Large projects (>1000 files): `"30m"` - `"3h"`

2. **Task complexity:**
   - Read-only tasks (e.g., analysis): Shorter timeouts
   - Read/write tasks (e.g., refactoring): Longer timeouts
   - Network-dependent tasks: Add buffer for network latency

3. **AI model speed:**
   - Faster models can handle more in less time
   - Complex prompts may require longer processing

4. **Resource constraints:**
   - Consider available CPU/memory
   - Higher timeouts = higher resource usage risk

**Debugging timeout issues:**

If an agent consistently times out:

1. **Check the timeout value:**
   ```bash
   switchboard list | grep my-agent
   ```

2. **View logs to understand what was happening:**
   ```bash
   switchboard logs my-agent --tail 100
   ```

3. **Run with increased timeout temporarily:**
   ```toml
   [[agent]]
   name = "my-agent"
   schedule = "0 */6 * * *"
   prompt = "Your prompt"
   timeout = "2h"  # Increased from 30m
   ```

4. **Optimize the prompt:**
   - Make prompts more specific and focused
   - Break large tasks into smaller subtasks
   - Use `prompt_file` for complex prompts to iterate more easily

5. **Use metrics to track timeout frequency:**
   ```bash
   switchboard metrics --agent my-agent --detailed
   ```
   Check the "Timeout Count" metric to see if timeouts are recurring.

### Log location and viewing

Switchboard maintains comprehensive logs for all agent executions and scheduler activity. Understanding log management helps with debugging and monitoring.

**Default log directory location:**

By default, logs are stored in:
```
.switchboard/logs/
```

This directory is created automatically in your project root (where `switchboard.toml` is located) on first execution.

**How to customize log directory:**

You can override the default location in `switchboard.toml`:

```toml
[settings]
log_dir = "/var/log/switchboard"           # Absolute path
log_dir = "./custom_logs"             # Relative to config file
log_dir = "~/.switchboard/logs"            # Home directory
```

**Log file naming convention:**

Log files follow this naming pattern:
```
<agent-name>-<timestamp>-<uuid>.log
```

Example:
```
code-reviewer-20240215T103045-a1b2c3d4.log
doc-updater-20240215T143022-e5f6g7h8.log
```

Components:
- `agent-name`: Name from `[[agent]]` configuration
- `timestamp`: ISO 8601 format (`YYYYMMDDTHHMMSS`)
- `uuid`: Unique identifier for the execution run

**Scheduler logs:**
```
scheduler.log
```

Contains scheduler lifecycle events and agent scheduling information.

**How to view logs using `switchboard logs` command:**

```bash
# View all agent logs (most recent first)
switchboard logs

# View logs for a specific agent
switchboard logs code-reviewer

# View scheduler logs
switchboard logs --scheduler

# Follow logs in real-time (like tail -f)
switchboard logs --follow
switchboard logs code-reviewer --follow

# View last N lines (default: 50)
switchboard logs --tail 100
switchboard logs code-reviewer --tail 200

# Combine options
switchboard logs code-reviewer --follow --tail 100
```

**Log file format:**

Each log entry includes:
```
[<timestamp>] [<stream>] <message>
```

Example:
```
[2024-02-15 10:30:45] [stdout] Starting code review...
[2024-02-15 10:31:02] [stdout] Analyzing src/main.rs...
[2024-02-15 10:32:15] [stderr] Warning: Unused import found
[2024-02-15 10:35:00] [stdout] Review complete. 3 issues found.
```

Streams:
- `[stdout]`: Standard output (normal program output)
- `[stderr]`: Standard error (warnings, errors)

**How to manually inspect log files:**

```bash
# List all log files
ls -la .switchboard/logs/

# View the most recent log file for an agent
ls -t .switchboard/logs/code-reviewer-*.log | head -1 | xargs cat

# Search for errors in all logs
grep -r "Error" .switchboard/logs/

# Search for a specific keyword
grep -r "timeout" .switchboard/logs/

# View log file size (large files may indicate issues)
du -h .switchboard/logs/*.log | sort -h
```

**Log rotation and management:**

Switchboard implements automatic log rotation to prevent disk space exhaustion:

1. **Automatic rotation:**
   - Logs are rotated based on file size (configurable)
   - Old logs are compressed to save space
   - A maximum number of log files are retained per agent

2. **Manual log cleanup:**
   ```bash
   # Remove logs older than 7 days
   find .switchboard/logs/ -name "*.log" -mtime +7 -delete
   
   # Remove all logs (use with caution)
   rm -rf .switchboard/logs/*
   
   # Remove logs for a specific agent
   rm .switchboard/logs/agent-name-*.log
   ```

3. **Archive old logs:**
   ```bash
   # Create an archive of old logs
   tar -czf switchboard-logs-backup-$(date +%Y%m%d).tar.gz .switchboard/logs/
   
   # Then clean up the original logs
   rm .switchboard/logs/*.log
   ```

4. **Monitoring disk usage:**
   ```bash
   # Check total log directory size
   du -sh .switchboard/logs/
   
   # Find largest log files
   du -h .switchboard/logs/*.log | sort -rh | head -10
   ```

**Log-related configuration options:**

```toml
[settings]
log_dir = ".switchboard/logs"           # Custom log directory
# Future options (may not be implemented yet):
# log_max_size = "100M"            # Max size before rotation
# log_max_files = 10               # Max files to retain
# log_compress = true              # Compress old logs
```

**Tips for effective log management:**

1. **Use `switchboard logs` for quick viewing:**
   - Easier than manually finding and opening files
   - Supports filtering by agent and tailing

2. **Archive old logs periodically:**
   - Set up a cron job to archive logs weekly/monthly
   - Keeps the log directory manageable

3. **Monitor for anomalies:**
   - Large log files may indicate runaway processes
   - Frequent error logs suggest agent issues
   - Missing logs may indicate execution failures

4. **Use log analysis tools:**
   ```bash
   # Count errors across all logs
   grep -r "Error" .switchboard/logs/ | wc -l
   
   # Find timeout occurrences
   grep -r "timed out" .switchboard/logs/
   
   # Analyze execution duration from logs
   grep "started\|completed" .switchboard/logs/code-reviewer-*.log
   ```

5. **Integration with log aggregation:**
   - Consider forwarding logs to external systems (e.g., ELK stack, Splunk)
   - Use log shippers like Filebeat or Fluentd for larger deployments

---

## Configuration

Switchboard uses a `switchboard.toml` configuration file placed at the root of your project. For the complete configuration schema and all available options, see [`PRD.md §6`](PRD.md#6-configuration-file).

### Minimal Example

```toml
version = "0.1.0"

[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"           # Every 6 hours
prompt = "Review all files changed in the last 6 hours."

[[agent]]
name = "doc-updater"
schedule = "0 2 * * 1"             # Every Monday at 2 AM
prompt_file = "prompts/update-docs.md"
readonly = true
timeout = "1h"
```

### Configuration Reference

For full details on all configuration fields including global settings, environment variables, timeout formats, and cron expressions, see the [Configuration Reference](#configuration-reference-1) section below or [`PRD.md §6`](PRD.md#6-configuration-file).

---

## Skills

Switchboard integrates with [skills.sh](https://skills.sh) to enable agent skills—reusable procedural knowledge that extends AI agent behavior. The Skills feature provides a `switchboard skills` CLI subcommand family for browsing, installing, and managing agent skills without leaving the terminal.

### Overview

Agent Skills are a lightweight, open standard for extending AI agent behavior with reusable capabilities. Skills can include:
- Domain-specific knowledge (e.g., security auditing, frontend design patterns)
- Development workflows (e.g., code review procedures, testing strategies)
- Technical expertise (e.g., specific framework best practices, API integrations)

Switchboard's Skills integration delegates all skill discovery and installation operations to the `npx skills` CLI. This means:
- Skills are sourced from the skills.sh marketplace or any public GitHub repository
- Switchboard provides a thin wrapper around `npx skills` for ergonomic integration with `switchboard.toml`
- No additional credentials or API keys are required beyond what `npx skills` needs

### Requirements

- **Node.js and `npx`** on the host machine (required for `switchboard skills list`, `install`, `update`, and `remove` commands)
- **Node.js inside agent containers** (automatically available via the `node:22-slim` base image)

### npx Requirements

The `skills list`, `skills install`, and `skills update` commands require **npx** (the Node.js package runner) to be installed on your host machine.

**If you encounter this error:**
```
Error: npx is required for this command. Install Node.js from https://nodejs.org
```

**Install Node.js and npx:**

Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install nodejs npm
```

macOS (using Homebrew):
```bash
brew install node
```

Other platforms: Download from [nodejs.org](https://nodejs.org)

**Verify installation:**
```bash
node --version
npx --version
```

**Commands that require npx:**
- `switchboard skills list` - Browse available skills from the npx skills registry
- `switchboard skills install <source>` - Install a skill from GitHub, npm, or local path
- `switchboard skills update` - Update installed skills

**Note:** Container execution does **NOT** require npx on the host. Skills declared in `switchboard.toml` are installed inside containers automatically via the `node:22-slim` base image.

For detailed troubleshooting, see [docs/NETWORK_FAILURE_HANDLING.md](docs/NETWORK_FAILURE_HANDLING.md) section "Issue 1: 'npx: command not found'".

### Declaring Skills in Configuration

Skills are declared per-agent using the `skills` field in `[[agent]]` sections. Each agent receives only the skills explicitly declared for it.

```toml
version = "0.1.0"

[[agent]]
name = "security-scan"
schedule = "0 2 * * *"
prompt = "Scan for security vulnerabilities."
skills = ["vercel-labs/agent-skills@security-audit"]

[[agent]]
name = "ui-reviewer"
schedule = "0 9 * * 1"
prompt = "Review UI components."
skills = [
    "vercel-labs/agent-skills@frontend-design",
    "anthropic/skills@react-best-practices"
]

[[agent]]
name = "general-reviewer"
schedule = "0 */6 * * *"
prompt = "Review recent changes."
# No skills field = this agent has no skills
```

**Skill source formats:**
- `owner/repo` — installs all skills from the repository
- `owner/repo@skill-name` — installs a specific skill from the repository
- Full GitHub or GitLab URLs are also supported

**Important notes:**
- Skills must be explicitly declared per-agent. Omitting the `skills` field means the agent has no skills.
- Skills are installed inside each agent's container at startup, not on the host.
- A failed skill installation aborts the agent run and is logged in `switchboard logs`.

### Skill Source Formats

Switchboard supports multiple formats for specifying skill sources. Choose the format that best fits your use case.

#### Format Options

**1. `owner/repo` Format**

Installs all skills from a repository.

```toml
[[agent]]
name = "frontend-dev"
skills = [
    "vercel-labs/agent-skills",  # Installs all skills from this repository
]
```

Use this format when:
- You want to install all available skills from a repository
- The repository contains a curated collection of skills for a specific domain
- You're exploring what skills are available

**2. `owner/repo@skill-name` Format**

Installs a specific skill from a repository.

```toml
[[agent]]
name = "security-audit"
skills = [
    "vercel-labs/agent-skills@security-audit",  # Only installs the security-audit skill
    "anthropics/skills@frontend-design",        # Only installs the frontend-design skill
]
```

Use this format when:
- You need a specific skill and don't want unnecessary dependencies
- You're optimizing container size by installing only required skills
- You know exactly which skill you need

**3. Full URL Format**

Installs from full GitHub or GitLab URLs.

```toml
[[agent]]
name = "custom-skills"
skills = [
    "https://github.com/vercel/agent-skills",
    "https://gitlab.com/owner/repo@specific-skill",
]
```

Use this format when:
- The repository is not on GitHub or GitLab
- You need to specify a custom Git host
- You're using self-hosted Git repositories

#### Format Validation

**Valid format requirements:**
- Must contain at least one forward slash (`/`) separating owner and repository
- Owner and repository names can contain: letters, numbers, hyphens, underscores
- The optional `@skill-name` can only appear at the end
- Skill names follow the same character rules as owner/repo names

**Valid examples:**
- `owner/repo`
- `owner/repo@skill-name`
- `my-org/my-repo@frontend-skills`
- `123user/456repo`

**Invalid examples:**
- `owner-only` — missing slash
- `owner@only` — slash before @ is required
- `owner/repo/extra` — too many slashes
- `` — empty string

#### CLI Usage

Install skills from the command line:

```bash
# Install all skills from a repository
switchboard skills install owner/repo

# Install a specific skill
switchboard skills install owner/repo@skill-name

# Install from a full URL
switchboard skills install https://github.com/owner/repo
```

#### Best Practices

1. **Start specific, expand as needed**: Begin with `owner/repo@skill-name` for known requirements, then expand to `owner/repo` if you need more skills.

2. **Avoid duplication**: Switchboard validates that the same skill source doesn't appear multiple times for a single agent.

3. **Use descriptive names**: When creating your own skill repositories, use clear, descriptive names for both the repository and individual skills.

4. **Test before production**: Use `switchboard validate` to check that your skill sources are valid before running agents in production.

#### Related Sections

- [Declaring Skills in Configuration](#declaring-skills-in-configuration) — How to configure skills in `switchboard.toml`
- [Skills CLI Commands](#skills-cli-commands) — Command-line skill management tools
- [Skills Validation](#skills-validation) — How Switchboard validates skill configurations

For detailed technical requirements, see [addtl-features/skills-feature.md](../addtl-features/skills-feature.md).

### Skills CLI Commands

Switchboard provides the `switchboard skills` subcommand family for skill management:

#### `switchboard skills list [--search <query>]`

Browse and search available skills from the skills.sh marketplace.

```bash
# Interactive browser (uses fzf-style interface)
switchboard skills list

# Search for specific skills
switchboard skills list --search react
switchboard skills list --search security
```

This command delegates to `npx skills find` and passes its output through directly, preserving formatting and interactivity.

#### `switchboard skills install <source> [--global]`

Install a skill into your project.

```bash
# Install a specific skill to project directory
switchboard skills install vercel-labs/agent-skills@frontend-design

# Install all skills from a repository
switchboard skills install vercel-labs/agent-skills

# Install globally to ~/.kilocode/skills/
switchboard skills install --global anthropic/skills@security-audit
```

Skills are installed to `.kilocode/skills/` in the project directory by default, or `~/.kilocode/skills/` when using `--global`.

#### `switchboard skills installed`

List all skills currently installed in your project, showing name, description, scope, and agent assignments.

```bash
switchboard skills installed
```

Example output:
```
Installed Skills

  Project (.kilocode/skills/)
  ─────────────────────────────────────────────────────────────────
  frontend-design          High-quality UI/UX design guidelines     [ui-reviewer]
  security-audit           Security vulnerability scanning           [security-scan]

  Global (~/.kilocode/skills/)
  ─────────────────────────────────────────────────────────────────
  skill-creator            Create and improve agent skills           [all agents]

  3 skills installed (2 project, 1 global)
```

#### `switchboard skills remove <name> [--global] [--yes]`

Remove an installed skill from your project.

```bash
# Remove a project-level skill (prompts for confirmation)
switchboard skills remove frontend-design

# Remove without confirmation
switchboard skills remove frontend-design --yes

# Remove a global skill
switchboard skills remove --global skill-creator
```

If a skill is referenced in `switchboard.toml`, a warning is printed but removal proceeds.

#### `switchboard skills update [<skill-name>]`

Update installed skills to their latest versions.

```bash
# Update all project-level skills
switchboard skills update

# Update a specific skill
switchboard skills update frontend-design
```

This delegates to `npx skills update` and passes output through directly.

#### Command Help Outputs

##### `switchboard skills list --help`

```
List available skills from the registry

List skills from the skills.sh registry. You can filter results using the --search flag to find specific skills.

# Examples

List all available skills: ```text switchboard skills list ```

Search for specific skills: ```text switchboard skills list --search docker switchboard skills list --search "file operations" ```

Usage: switchboard skills list [OPTIONS]

Options:
  -s, --search <SEARCH>
          Filter skills by query string
          
          Search terms to filter skills list. This filters by name, description, and other metadata from the skills.sh registry.

  -h, --help
          Print help (see a summary with '-h')
```

##### `switchboard skills install --help`

```
Install a skill from a source

Install a skill from a GitHub repository, npm package, or local path. Skills are installed by default to the project-level skills directory.

# Examples

Install a skill from GitHub: ```text switchboard skills install owner/repo ```

Install a specific skill from a repo: ```text switchboard skills install owner/repo@skill-name ```

Install globally (available to all projects): ```text switchboard skills install --global owner/repo ```

Usage: switchboard skills install [OPTIONS] <SOURCE>

Arguments:
  <SOURCE>
          Skill source (e.g., npm package name, GitHub URL, or local path)
          
          The source can be: - GitHub repository: `owner/repo` or `owner/repo@skill-name` - Full GitHub URL: `https://github.com/owner/repo` - GitLab URL: `https://gitlab.com/owner/repo` - npm package name

Options:
      --global
          Install globally instead of project-local
          
          When set, installs the skill to the global skills directory (~/.kilocode/skills/) instead of the project-level directory (.kilocode/skills/). Global skills are available to all projects.

  -h, --help
          Print help (see a summary with '-h')
```

##### `switchboard skills installed --help`

```
List installed skills

List all currently installed skills in both project and global scopes. Shows skill name, description, version, source, and which agents use each skill.

# Examples

List all installed skills: ```text switchboard skills installed ```

List only global skills: ```text switchboard skills installed --global ```

Usage: switchboard skills installed [OPTIONS]

Options:
      --global
          Show only global skills
          
          When set, only shows skills from the global skills directory (~/.kilocode/skills/). Project-level skills from .kilocode/skills/ are not displayed.

  -h, --help
          Print help (see a summary with '-h')
```

##### `switchboard skills remove --help`

```
Remove an installed skill

Removes a skill from either the project or global skills directory. Shows a warning if the skill is still referenced by agents in the configuration. Requires confirmation unless the --yes flag is used.

# Examples

Remove a project skill with confirmation: ```text switchboard skills remove frontend-design ```

Remove a global skill: ```text switchboard skills remove --global skill-creator ```

Remove without confirmation: ```text switchboard skills remove --yes frontend-design ```

Usage: switchboard skills remove [OPTIONS] <SKILL_NAME>

Arguments:
  <SKILL_NAME>
          Name of the skill to remove
          
          The name of the skill directory to remove from the skills directory.

Options:
      --global
          Remove from global skills directory
          
          When set, removes the skill from the global skills directory (~/.kilocode/skills/) instead of the project-level directory (.kilocode/skills/).

      --yes
          Skip confirmation prompt
          
          When set, bypasses the confirmation prompt and removes the skill immediately. Use with caution.

  -h, --help
          Print help (see a summary with '-h')
```

##### `switchboard skills update --help`

```
Update installed skills to their latest versions

If a specific skill name is provided, only that skill is updated. If no skill name is provided, all installed skills are updated.

# Examples

Update all installed skills: ```text switchboard skills update ```

Update a specific skill: ```text switchboard skills update frontend-design ```

Usage: switchboard skills update [-- <skill-name>]

Arguments:
  [skill-name]
          Optional skill name to update. If omitted, updates all installed skills

Options:
  -h, --help
          Print help (see a summary with '-h')
```

### Skills and Container Execution

When an agent with declared skills starts its container, Switchboard automatically installs those skills inside the container before the Kilo Code CLI is invoked:

1. Container starts with the base `node:22-slim` image
2. For each skill in the agent's `skills` list, `npx skills add <source> -a kilo -y` runs
3. Skills install sequentially in declaration order
4. If installation succeeds, the Kilo Code CLI executes the agent's prompt
5. If installation fails, the container exits with a non-zero code and logs the error

This approach means:
- No host-side skill management is required for scheduled agents
- Skills are always up to date on each run
- Each agent can have a different set of skills
- No Node.js is required on the host for scheduled execution (only inside containers)

### Skills Validation

The `switchboard validate` command checks skill-related configuration:

```bash
switchboard validate
```

Validation includes:
- Warning if an agent has an empty `skills = []` field
- Error if a skills entry has invalid format (not `owner/repo` or `owner/repo@skill-name`)
- Error if the same skill source appears more than once in a single agent's `skills` list

### Example Workflow

Here's a complete example of using skills in Switchboard:

```bash
# 1. Search for relevant skills
switchboard skills list --search frontend

# 2. Install a skill for your project
switchboard skills install vercel-labs/agent-skills@frontend-design

# 3. Configure an agent to use the skill
# Edit switchboard.toml:
[[agent]]
name = "ui-reviewer"
schedule = "0 9 * * 1"
prompt = "Review the UI components and suggest improvements."
skills = ["vercel-labs/agent-skills@frontend-design"]

# 4. Validate the configuration
switchboard validate

# 5. Test the agent (skills install inside container)
switchboard run ui-reviewer

# 6. Check which agents use which skills
switchboard skills installed

# 7. Start the scheduler (agents install skills on each run)
switchboard up
```

For more detailed technical specifications, see [`addtl-features/skills-feature.md`](addtl-features/skills-feature.md).

### Troubleshooting

This section covers common issues you may encounter when using the Skills feature and how to resolve them.

#### 1. npx not found

**Problem:** When running `switchboard skills list`, `switchboard skills install`, or `switchboard skills update`, you see an error like:

```
Error: npx is required for this command. Install Node.js from https://nodejs.org
```

**Likely Cause:** Node.js and npx are not installed on your host machine.

**Solution:**

1. Install Node.js (which includes npx):
   - **Ubuntu/Debian:** `sudo apt-get update && sudo apt-get install nodejs npm`
   - **macOS:** `brew install node`
   - **Windows:** Download from [nodejs.org](https://nodejs.org)

2. Verify installation:
   ```bash
   node --version
   npx --version
   ```

3. If already installed but not found, ensure the Node.js bin directory is in your PATH:
   ```bash
   # Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.nvm/versions/node/*/bin:$PATH"
   ```

**Note:** Container execution does NOT require npx on the host. Skills are installed inside containers automatically via the `node:22-slim` base image.

---

#### 2. Invalid skill source format

**Problem:** You receive a validation error when specifying skills in `switchboard.toml`:

```
Error: Invalid skill source format: "invalid-format"
```

**Likely Cause:** The skill source string doesn't match one of the supported formats.

**Solution:**

Ensure your skill source uses one of these valid formats:

| Format | Example | Description |
|--------|---------|-------------|
| `owner/repo` | `vercel-labs/agent-skills` | Install all skills from repository |
| `owner/repo@skill-name` | `vercel-labs/agent-skills@security-audit` | Install specific skill |
| Full GitHub URL | `https://github.com/owner/repo` | Full URL format |
| Full GitHub URL with skill | `https://github.com/owner/repo#path/to/skill` | URL with path |

Example valid configuration:
```toml
[[agent]]
name = "security-scan"
skills = [
    "vercel-labs/agent-skills@security-audit",  # Correct format
    "https://github.com/myorg/my-skills",        # Full URL also works
]
```

---

#### 3. Skill installation failures

**Problem:** An agent fails to run because skill installation fails:

```
Error: Failed to install skill: vercel-labs/agent-skills
Error: npx skills install failed with exit code 1
```

**Likely Causes:**
- The skill repository doesn't exist or is private
- The skill name doesn't exist in the repository
- The repository doesn't have a valid SKILL.md file

**Solution:**

1. **Verify the skill exists:**
   ```bash
   switchboard skills list  # Browse available skills
   ```

2. **Test skill installation manually:**
   ```bash
   npx skills install owner/repo@skill-name
   ```

3. **Check for private repositories:** Skills must be in public repositories. For private skills, consider using a public mirror or hosting your own skill registry.

4. **Verify the skill name:** Make sure the skill name (after `@`) exists in the repository:
   ```toml
   # Check repository for valid skill names
   skills = ["owner/repo@valid-skill-name"]
   ```

5. **Check agent logs for details:**
   ```bash
   switchboard logs <agent-name>
   ```

---

#### 4. Duplicate skill entries

**Problem:** Validation fails with a duplicate skill error:

```
Error: Duplicate skill entry: "vercel-labs/agent-skills@security-audit"
```

**Likely Cause:** The same skill is listed multiple times in an agent's skills array.

**Solution:**

Remove duplicate entries from your `switchboard.toml`:

```toml
# ❌ Invalid - duplicate skill
[[agent]]
name = "security-scan"
skills = [
    "vercel-labs/agent-skills@security-audit",
    "vercel-labs/agent-skills@security-audit",  # Duplicate!
]

# ✅ Valid - unique skills
[[agent]]
name = "security-scan"
skills = [
    "vercel-labs/agent-skills@security-audit",
    "vercel-labs/agent-skills@code-review",
]
```

Run validation to confirm:
```bash
switchboard validate
```

---

#### 5. Network issues during skill installation

**Problem:** Skills fail to install due to network errors:

```
Error: Network error: Could not resolve host: github.com
Error: Failed to clone repository: Connection timed out
```

**Likely Causes:**
- No internet connectivity
- Firewall or proxy blocking connections
- DNS resolution issues
- GitHub/GitLab is temporarily unavailable

**Solution:**

1. **Check internet connectivity:**
   ```bash
   ping -c 3 github.com
   curl -I https://github.com
   ```

2. **Check Docker network settings:**
   ```bash
   docker network ls
   docker network inspect bridge
   ```

3. **Configure proxy if needed:**
   ```bash
   # Set environment variables
   export HTTP_PROXY=http://proxy.example.com:8080
   export HTTPS_PROXY=http://proxy.example.com:8080
   ```

4. **For corporate networks:** Ensure Docker has access to external networks. You may need to configure Docker's DNS or proxy settings in `/etc/docker/daemon.json`.

5. **Retry the operation:** Network issues are often transient:
   ```bash
   switchboard skills update
   # or
   switchboard run <agent-name>
   ```

6. **Pre-cache skills:** If network issues are common, consider building the Docker image with skills pre-installed using a custom Dockerfile.

---

#### 6. Malformed SKILL.md warnings

**Problem:** You see warnings about malformed SKILL.md files:

```
Warning: Malformed SKILL.md in security-audit: missing required field 'name'
Warning: Malformed SKILL.md in code-review: invalid format in 'commands' section
```

**Likely Cause:** The skill's SKILL.md file doesn't conform to the expected schema.

**Solution:**

1. **Understand SKILL.md requirements:** A valid SKILL.md should have:
   - `name` - Skill name
   - `description` - What the skill does
   - `commands` - List of commands the skill provides (optional)
   - `rules` - Usage rules (optional)

2. **These warnings are non-fatal:** The skill may still work, but some features might be limited.

3. **Verify the skill source:** Check if the skill is from an official or well-maintained repository:
   ```bash
   switchboard skills list  # Look for official skills
   ```

4. **Report to skill maintainers:** If you rely on a specific skill, consider reporting the issue to the skill repository maintainers.

5. **Use an alternative skill:** If a skill has persistent issues, look for alternatives:
   ```bash
   switchboard skills list | grep <keyword>
   ```

6. **Suppress warnings if needed:** These warnings don't prevent execution. If the skill works despite warnings, you can continue using it.

---

### Getting More Help

If you're still experiencing issues:

1. **Check the full logs:**
   ```bash
   switchboard logs <agent-name>
   switchboard logs --scheduler
   ```

2. **Validate your configuration:**
   ```bash
   switchboard validate
   ```

3. **Check the skills.sh documentation:**
   - [Skills.sh documentation](https://skills.sh/docs)
   - [Skills.sh CLI reference](https://skills.sh/cli)

4. **File an issue:** If you believe you've found a bug, [open an issue](https://github.com/your-org/switchboard/issues) with:
   - The exact error message
   - Your switchboard.toml configuration
   - Output of `switchboard validate`
   - Relevant log excerpts

---

## Container Skill Installation Behavior

This section explains how skills are installed inside Docker containers when agents run, including the step-by-step process, key behaviors, and dependencies.

### What is a Container Skill?

A **container skill** is a pre-built module that extends the capabilities of Kilo Code agents running inside Docker containers. Skills are automatically installed inside agent containers at startup before the agent begins executing its task.

Container skills enable AI agents to have specialized capabilities such as:
- Frontend design tools (e.g., UI/UX design guidelines)
- Security auditors (vulnerability scanning)
- Specialized code analysis utilities
- Language-specific helpers (Python, Rust, JavaScript, etc.)

### Configuration

Skills are declared per-agent in [`switchboard.toml`](switchboard.toml:1) using the optional `skills` field:

```toml
version = "0.1.0"

[[agent]]
name = "security-scan"
schedule = "0 2 * * *"
prompt = "Scan for security vulnerabilities."
skills = [
    "vercel-labs/agent-skills@security-audit",
    "anthropic/skills@frontend-design"
]

[[agent]]
name = "general-reviewer"
schedule = "0 */6 * * *"
prompt = "Review recent changes."
# No skills field = this agent has no skills
```

**Important:**
- The `skills` field is completely optional
- Omitting the field means the agent has no skills
- Each agent has independent skills (no sharing between agents)
- Backwards compatible with existing configurations

### Skill Formats

Switchboard supports two formats for specifying skill sources:

| Format | Example | Description |
|---------|---------|-------------|
| `owner/repo` | `vercel-labs/agent-skills` | Installs all skills from the repository |
| `owner/repo@skill-name` | `vercel-labs/agent-skills@frontend-design` | Installs a specific skill from the repository |

Full GitHub or GitLab URLs are also supported.

**Validation:**
- `switchboard validate` checks skill formats and warns on empty lists
- Invalid formats produce validation errors
- Duplicate entries are detected and reported

### Installation Process (5 Phases)

The installation process flows through five phases from configuration to execution:

#### Phase 1: Configuration Validation (Host Side)

1. **User declares skills in `switchboard.toml`** using the optional `skills` field
2. **`switchboard validate` checks configuration:**
   - Validates skill entry format using regex pattern
   - Warns if `skills = []` is present but empty
   - Reports errors for invalid formats or duplicates

#### Phase 2: Entrypoint Script Generation (Host Side)

1. **Skill format validation:** Each skill is validated for correct format
2. **Preexisting skill detection:** Scans `.kilocode/skills/` directory for manually managed skills
3. **Script generation:** Creates a POSIX shell script with the following structure:
   ```sh
   #!/bin/sh
   set -e  # Exit immediately on any command failure

   # Error handler
   handle_error() {
       local exit_code=$1
       if [ $exit_code -ne 0 ]; then
           echo "[SKILL INSTALL ERROR] Command failed with exit code $exit_code"
       fi
   }
   trap 'handle_error $?' EXIT

   # Install skills sequentially
   npx skills add owner/repo@skill-name -a kilo -y 2>&1 | while IFS= read -r line; do
     echo "[SKILL INSTALL STDERR] $line"
   done

   # Hand off to Kilo Code CLI
   exec kilocode --yes "$@"
   ```
4. **Empty skills handling:** Returns empty string, container uses default entrypoint

#### Phase 3: Container Creation (Docker Side)

1. **Docker container configuration:**
   - Custom entrypoint script is set when skills are present
   - Entrypoint set to `/bin/sh -c <script>`
   - When no skills, `entrypoint` remains `None` (uses Dockerfile default)

2. **Skills directory mounting:**
   - If `.kilocode/skills/` exists, it's mounted read-only into container at `/workspace/.kilocode/skills`
   - Allows manually managed skills to be accessible inside container

3. **Container creation and start:** Container created and started using Docker API

#### Phase 4: Skill Installation (Container Side)

1. **Script execution at container startup:**
   - Container starts and runs the entrypoint script via `/bin/sh -c`
   - Skills are installed sequentially in declaration order
   - `set -e` ensures immediate exit on any command failure

2. **npx skills add execution:**
   - Each skill installed using: `npx skills add <source> -a kilo -y`
   - `-a kilo`: Author/app identifier for the skills registry
   - `-y`: Auto-confirm, skip interactive prompts
   - stderr captured and prefixed with `[SKILL INSTALL STDERR]`

3. **Preexisting skill handling:**
   - Skills already in `.kilocode/skills/` skip npx installation
   - Logged as: `[SKILL INSTALL] Using preexisting skill: <name> (skipping npx installation)`

4. **Handoff to Kilo Code CLI:**
   - After all skills installed, script executes: `exec kilocode --yes "$@"`
   - `exec` replaces shell process with kilocode (proper signal handling)

#### Phase 5: Agent Execution (Container Side)

1. **Kilo Code CLI runs with original arguments:**
   - Agent prompt is passed as a positional argument
   - All original CLI arguments are forwarded
   - Agent executes with installed skills available in its environment

### Key Behaviors

#### Skills Installed Fresh Each Container Run

- Skills are NOT persisted between container runs
- Each run installs fresh from source
- First run with skills may take longer (downloading)
- Ensures skills are always up to date

#### Sequential Installation

- Skills are installed one at a time in declaration order
- Allows for dependency ordering between skills
- Failed skill stops entire installation
- `set -e` ensures immediate exit on failure

#### Per-Agent Skill Scoping

- Each agent has independent skills
- Skills must be explicitly declared for each agent
- No implicit inheritance or sharing between agents
- If `skills` field is omitted, agent receives no skills

#### Backwards Compatibility

- `skills` field is completely optional
- Existing configs without skills continue to work
- No warnings or errors for missing skills field
- Mixed environments supported (some agents with skills, some without)

### Dependencies

#### Required Inside Container

| Dependency | Version/Type | Purpose |
|------------|---------------|---------|
| **Node.js** | v22+ | Provides `npx` command for skill installation |
| **npx** | Included with Node.js | Package runner that invokes skills CLI |
| **POSIX sh** | `/bin/sh` | Shell interpreter for entrypoint script |

**Satisfied by existing Docker image:** Base image `node:22-slim` includes Node.js v22, npx, and `/bin/sh`.

#### Required on Host (for CLI commands only)

| Dependency | Purpose | Commands Requiring |
|------------|---------|-------------------|
| **Node.js** | Provides `npx` | `switchboard skills list`, `install`, `update` |
| **npx** | Package runner | Same as above |

**Note:** Host-side Node.js is NOT required for container skill installation. Only for CLI commands that manually invoke npx.

### Performance Considerations

- **Skill installation time counts toward agent timeout budget**
- Users should account for skill install time in timeout values
- **Target performance:** 15 seconds for single skill installation
- Sequential installation means multiple skills take proportionally longer
- First run with new skills may be slower (downloading dependencies)

### Error Conditions

| Error Type | Condition | Behavior |
|------------|-----------|----------|
| **Invalid Skill Format** | Missing/multiple `/`, empty components | Validation error by `switchboard validate` |
| **Empty Skills Field** | `skills = []` | Warning issued, container uses default entrypoint |
| **Duplicate Skill** | Same skill appears multiple times | Validation error by `switchboard validate` |
| **Installation Failure** | Network unavailable, invalid repo | Script exits with non-zero code, failure logged |
| **Container Timeout** | Timeout too small for installation | Container sent SIGTERM/SIGKILL, unknown installation status |
| **npx Not Found** | Container image lacks Node.js/npx | Entrypoint script fails, container exits with error |

For detailed technical specifications, see [`addtl-features/skills-feature.md`](addtl-features/skills-feature.md).

---

## Skill Installation Failure Handling

When skills are installed inside Docker containers during agent execution, failures can occur due to network issues, invalid repositories, missing dependencies, or other problems. Switchboard provides comprehensive failure handling including exit code detection, detailed logging, and metrics tracking.

### Non-Zero Exit Code Behavior

The system detects failed skill installations by monitoring the container's exit code:

**Detection logic:**
- **Exit code 0**: Skills installed successfully, agent execution proceeds normally
- **Non-zero exit code (and not timed out)**: Skill installation failed, agent run aborted
- **Timeout**: Container terminated before installation completed—status is unknown

**How it works:**

When a container with skills finishes execution, Switchboard examines the exit code:

```
Exit code analysis:
├─ 0 → Skills installed successfully
├─ 1-127 → Installation failed (non-zero code, no timeout)
├─ 128+ → Signal termination (e.g., 137 = SIGKILL from timeout)
└─ Unknown (timeout) → Cannot determine if skills installed
```

**Example scenarios:**

| Exit Code | Cause | Agent Behavior |
|-----------|-------|----------------|
| 0 | All skills installed | Agent executes normally |
| 1 | npx skills failed (network error) | Agent run aborted, error logged |
| 127 | npx command not found | Agent run aborted, error logged |
| 137 | Container timeout (SIGKILL) | Unknown skill status, metrics show timeout |
| 130 | SIGINT (Ctrl+C) | Agent interrupted, unknown skill status |

**Important notes:**
- Any non-zero exit code during skill installation marks the entire installation as failed
- Partial failures are not tracked—if one skill fails, all skills for that agent are marked as failed
- Timeout scenarios (exit code 137) cannot distinguish between successful and failed installations

### The [SKILL INSTALL] Log Prefix

All skill installation activities in agent logs are prefixed with `[SKILL INSTALL]` for easy identification and filtering.

**Log prefix formats:**

```
[SKILL INSTALL] Installing skills for agent '<agent-name>'
[SKILL INSTALL] Installing skill: <skill-source>
[SKILL INSTALL] Found N preexisting skill(s) that will skip npx installation
[SKILL INSTALL] Using preexisting skill: <name> (skipping npx installation)
[SKILL INSTALL STDERR] <error-message-from-npx>
```

**Example log output:**

```bash
# Successful skill installation
[2024-02-20 10:30:15] [SKILL INSTALL] Installing skills for agent 'security-scan'
[2024-02-20 10:30:15] [SKILL INSTALL] Installing skill: vercel-labs/agent-skills@security-audit
[2024-02-20 10:30:22] Starting Kilo Code CLI...

# Installation with preexisting skills
[2024-02-20 10:30:15] [SKILL INSTALL] Installing skills for agent 'ui-reviewer'
[2024-02-20 10:30:15] [SKILL INSTALL] Found 1 preexisting skill(s) that will skip npx installation
[2024-02-20 10:30:15] [SKILL INSTALL] Using preexisting skill: frontend-design (skipping npx installation)
[2024-02-20 10:30:16] Starting Kilo Code CLI...

# Failed skill installation
[2024-02-20 10:30:15] [SKILL INSTALL] Installing skills for agent 'bug-fixer'
[2024-02-20 10:30:15] [SKILL INSTALL] Installing skill: nonexistent-repo@invalid-skill
[2024-02-20 10:30:18] [SKILL INSTALL STDERR] error: repository 'nonexistent-repo' not found
[2024-02-20 10:30:18] [SKILL INSTALL ERROR] Command failed with exit code 1
[2024-02-20 10:30:18] Container exited with code 1
```

**Filtering logs for skill installation:**

```bash
# View only skill installation lines
switchboard logs security-scan | grep "\[SKILL INSTALL\]"

# View only skill installation errors
switchboard logs security-scan | grep "\[SKILL INSTALL ERROR\]"

# View stderr from skill installation
switchboard logs security-scan | grep "\[SKILL INSTALL STDERR\]"
```

### How Failures Appear in `switchboard logs` Output

Skill installation failures appear in agent logs with specific error patterns that are easy to identify.

**Typical failure log patterns:**

```bash
# Network failure example
[2024-02-20 14:22:10] [SKILL INSTALL] Installing skills for agent 'code-reviewer'
[2024-02-20 14:22:10] [SKILL INSTALL] Installing skill: vercel-labs/agent-skills@frontend-design
[2024-02-20 14:22:35] [SKILL INSTALL STDERR] npm ERR! network request failed
[2024-02-20 14:22:35] [SKILL INSTALL STDERR] npm ERR! code ENOTFOUND
[2024-02-20 14:22:35] [SKILL INSTALL STDERR] npm ERR! syscall getaddrinfo
[2024-02-20 14:22:35] [SKILL INSTALL ERROR] Command failed with exit code 1
[2024-02-20 14:22:35] Container exited with code 1
[2024-02-20 14:22:35] Agent 'code-reviewer' failed to install skills

# Invalid repository example
[2024-02-20 14:30:05] [SKILL INSTALL] Installing skills for agent 'security-scan'
[2024-02-20 14:30:05] [SKILL INSTALL] Installing skill: invalid-org/missing-repo@skill-name
[2024-02-20 14:30:12] [SKILL INSTALL STDERR] error: repository 'invalid-org/missing-repo' not found
[2024-02-20 14:30:12] [SKILL INSTALL ERROR] Command failed with exit code 1
[2024-02-20 14:30:12] Container exited with code 1
[2024-02-20 14:30:12] Agent 'security-scan' failed to install skills

# Missing npx example
[2024-02-20 14:40:00] [SKILL INSTALL] Installing skills for agent 'doc-updater'
[2024-02-20 14:40:00] [SKILL INSTALL] Installing skill: anthropic/skills@writing-helper
[2024-02-20 14:40:01] [SKILL INSTALL STDERR] /bin/sh: npx: not found
[2024-02-20 14:40:01] [SKILL INSTALL ERROR] Command failed with exit code 127
[2024-02-20 14:40:01] Container exited with code 127
[2024-02-20 14:40:01] Agent 'doc-updater' failed to install skills

# Timeout example
[2024-02-20 14:50:00] [SKILL INSTALL] Installing skills for agent 'heavy-task'
[2024-02-20 14:50:00] [SKILL INSTALL] Installing skill: vercel-labs/agent-skills@large-bundle
[2024-02-20 14:50:30] Agent timed out after 30m
[2024-02-20 14:50:30] Terminating container heavy-task-abc123 due to timeout
[2024-02-20 14:50:31] Container heavy-task-abc123 stopped (exit code: 137)
[2024-02-20 14:50:31] Unknown skill installation status (timeout)
```

**Identifying the failure type:**

| Exit Code | Log Pattern | Likely Cause |
|-----------|-------------|--------------|
| 1 | `Command failed with exit code 1` | npx skills failed (network, repo, etc.) |
| 127 | `npx: not found` | Node.js/npx missing in container |
| 137 | `timed out` | Installation exceeded agent timeout |
| 130 | `SIGINT` | Container interrupted manually |

**Viewing skill installation failures:**

```bash
# View all skill installation errors across agents
grep -r "\[SKILL INSTALL ERROR\]" .switchboard/logs/

# View failures for a specific agent
grep "\[SKILL INSTALL ERROR\]" .switchboard/logs/security-scan-*.log

# Count skill installation failures
grep -r "\[SKILL INSTALL ERROR\]" .switchboard/logs/ | wc -l

# View recent failures
grep "\[SKILL INSTALL ERROR\]" .switchboard/logs/*/*.log | tail -20
```

### Metrics Tracking for Skill Failures

Switchboard automatically tracks skill installation metrics across all agent runs, providing visibility into installation success rates and failure patterns.

**Tracked metrics:**

| Metric | Description | Updated When |
|--------|-------------|--------------|
| **Total Skills Installed** | Cumulative count of skills installed successfully | After each successful agent run with skills |
| **Total Skills Failed** | Cumulative count of skills that failed to install | After each failed skill installation |
| **Runs with Skill Failures** | Number of agent runs that had at least one skill installation failure | After each agent run with failed skills |
| **Avg Skill Install Time** | Average time (in seconds) to install a skill | After each agent run with skills |

**Viewing skill installation metrics:**

```bash
# View summary metrics for all agents
switchboard metrics

# View detailed metrics including skill statistics
switchboard metrics --detailed

# View metrics for a specific agent
switchboard metrics --agent security-scan --detailed

# Example output:
# security-scan
# ───────────────────────────────────────────────────────────────────────
#   Total Runs:              15
#   Successful Runs:         12
#   Failed Runs:             3
#   Success Rate:            80.0%
#   Total Duration:          45m 30s
#   Average Duration:        3m 2s
#   Total Skills Installed:  45
#   Total Skills Failed:     3
#   Runs with Skill Failures: 2
#   Avg Skill Install Time:  5.2s
```

**Metrics interpretation:**

- **Total Skills Installed**: Count of skills that successfully installed across all agent runs
- **Total Skills Failed**: Count of skills that failed to install (each failed skill increments this count)
- **Runs with Skill Failures**: Number of agent runs that experienced at least one skill installation failure
- **Avg Skill Install Time**: Average time spent installing skills (successful and failed)

**Example scenarios:**

```bash
# Scenario 1: All skills installed successfully
Agent runs 5 times, 3 skills each
Total Skills Installed: 15
Total Skills Failed: 0
Runs with Skill Failures: 0

# Scenario 2: Some runs fail to install skills
Agent runs 5 times, 3 skills each
- Runs 1-3: All skills install (9 installed, 0 failed)
- Run 4: All skills fail (0 installed, 3 failed)
- Run 5: All skills install (3 installed, 0 failed)
Total Skills Installed: 12
Total Skills Failed: 3
Runs with Skill Failures: 1

# Scenario 3: Multiple runs with failures
Agent runs 5 times, 2 skills each
- Run 1: Both skills install (2 installed, 0 failed)
- Run 2: Both skills fail (0 installed, 2 failed)
- Run 3: One skill fails, one succeeds (1 installed, 1 failed) [if partial detection were supported]
- Run 4: Both skills install (2 installed, 0 failed)
- Run 5: Both skills fail (0 installed, 2 failed)
Total Skills Installed: 4
Total Skills Failed: 4
Runs with Skill Failures: 2
```

**Note:** Current implementation treats skill installation as all-or-nothing. If any skill fails during installation, all skills for that agent run are counted as failed. Partial success detection is not currently supported.

**Using metrics to identify issues:**

```bash
# Identify agents with high skill failure rates
switchboard metrics --detailed | grep -A 10 "Runs with Skill Failures"

# Find agents experiencing recurring skill installation problems
switchboard metrics | awk '/^└─/ && /skill/ {print}'

# Monitor skill installation time trends
switchboard metrics --detailed | grep "Avg Skill Install Time"
```

---

## Open Questions

This section documents open questions and deferred features for the Skills feature. These are known limitations or design decisions that may be revisited in future releases.

### OQ-1: Skill install latency and agent timeouts

**Current Behavior:**

Skill installation can take significant time depending on network conditions, skill size, and npm package complexity. Currently, users must manually adjust agent timeouts to accommodate skill installation latency.

**Configuration Example:**

```toml
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review recent changes"
timeout = "30m"  # Increased from default to allow for skill installation
```

**Deferred Feature:**

- **Auto-adjustment feature**: Automatically extend timeouts when skill installation is detected
- GitHub issue needed to track this feature request

---

### OQ-2: Skill version pinning support

**Current Behavior:**

Currently, skills are installed without version pinning, always fetching the latest available version from the source.

```toml
[[agent]]
name = "code-reviewer"
skills = ["github:owner/skill-name"]  # No version control
```

**Deferred Feature:**

- **Version pinning**: Support for pinning skills to specific versions (e.g., `"github:owner/skill-name@v1.2.0"`)
- GitHub issue needed to track requirements and design for version pinning

---

### OQ-3: Skill caching across runs

**Current Behavior:**

Skills are installed fresh each time a container runs. This ensures consistency but increases startup latency.

**Rationale for Fresh Install:**

1. **Consistency**: Each run uses the latest skill versions, reducing compatibility issues
2. **Simplicity**: No need to manage cache invalidation or version conflicts
3. **Storage**: Avoids persistent storage complexity in ephemeral containers

**Deferred Feature:**

- **Caching layer**: Optional persistent cache for installed skills
- GitHub issue needed for caching feature request

---

### OQ-4: npx skills version pinning

**Current Behavior:**

Container skills are executed via `npx` without version pinning, always using the latest version.

```yaml
# Container skill format
skill:
  type: container
  image: node:20
  command: npx -y skill-name  # Always uses latest
```

**Trade-off:**

| Approach | Pros | Cons |
|----------|------|------|
| Latest (current) | Always up-to-date, no maintenance | Potential breaking changes |
| Pinned version | Predictable, stable | Requires manual updates, potential security issues |

**Deferred Feature:**

- **Version pinning for npx**: Support specifying versions for container skills
- GitHub issue needed for version pinning discussion

---

### OQ-5: Skill install failure policy

**Current Behavior:**

When any skill fails to install, the entire agent run is aborted. This is a strict all-or-nothing approach.

```toml
[[agent]]
name = "security-scanner"
skills = ["npm:security-tool"]  # If this fails, the agent won't run
```

**Current Implementation:**

- If any skill fails to install, the agent run fails with an error
- All skills for that agent run are counted as failed (no partial success)

**Deferred Feature:**

- **`skills_optional` flag**: Allow agents to continue running even if some skills fail to install
- Example: `skills_optional = true` would allow the agent to run with available skills
- GitHub issue needed for optional skills feature request

---

## CLI Commands

Switchboard provides 13 commands for managing your scheduled AI agents:

| Command | Description |
|---------|-------------|
| `switchboard up [--config <path>] [--detach]` | Build image (if needed) and start the scheduler. Runs in foreground by default; `--detach` backgrounds it |
| `switchboard run <agent-name> [--config <path>]` | Immediately execute a single agent (ignoring its schedule). Useful for testing |
| `switchboard build [--config <path>] [--no-cache]` | Build or rebuild the agent Docker image without starting the scheduler |
| `switchboard list [--config <path>]` | Print all configured agents, their schedules, and their prompts |
| `switchboard logs [<agent-name>] [--follow] [--tail <n>]` | View logs from agent runs. Optionally filter by agent name |
| `switchboard down` | Stop the scheduler and any running agent containers |
| `switchboard validate [--config <path>]` | Parse and validate the config file without running anything |
| `switchboard skills list [--search <query>]` | Browse available skills from skills.sh registry |
| `switchboard skills install <source> [--global]` | Install a skill from GitHub/npm/local |
| `switchboard skills installed [--global]` | List installed skills in project/global scope |
| `switchboard skills remove <name> [--global] [--yes]` | Remove an installed skill |
| `switchboard skills update [<skill-name>]` | Update installed skills to latest versions |
| `switchboard validate` | Validate skill configuration (includes skills field validation)

---

## Metrics

Switchboard automatically tracks and displays execution metrics for all configured agents, giving you insight into agent performance, reliability, and resource usage over time.

### Usage

View metrics using the `switchboard metrics` command:

```bash
# View summary table for all agents
switchboard metrics

# View detailed metrics for all agents
switchboard metrics --detailed

# View detailed metrics for a specific agent
switchboard metrics --detailed --agent my-agent

# Use a custom config file
switchboard metrics --config /path/to/switchboard.toml
```

**Table view** displays a compact summary with key metrics for each agent.

**Detailed view** shows comprehensive statistics including timestamps, queue wait time, timeout count, and recent run history.

### Metrics Tracked

The metrics system tracks the following information for each agent:

| Metric | Description |
|--------|-------------|
| **Total Runs** | Number of times the agent has been executed |
| **Successful Runs** | Number of runs that completed successfully |
| **Failed Runs** | Number of runs that terminated with an error |
| **Success Rate** | Percentage of successful runs out of total runs |
| **Total Duration** | Cumulative time spent running the agent across all executions |
| **Average Duration** | Mean execution time per run |
| **First Run** | Timestamp of the agent's first execution |
| **Last Run** | Timestamp of the agent's most recent execution |
| **Last Duration** | Execution time of the most recent run |
| **Queue Wait Time** | Time the agent spent waiting in the queue before starting |
| **Timeout Count** | Number of runs that exceeded the configured timeout |
| **Termination Signals** | Number of runs terminated by system signals |
| **Recent Runs** | History of the last 5 runs with timestamps, duration, status, and error messages |

### Storage

Metrics are stored persistently in JSON format at:

```
<log_dir>/metrics.json
```

where `<log_dir>` is the log directory configured in `switchboard.toml` (default: `.switchboard/logs`). The file is updated atomically after each agent run, ensuring data integrity even if multiple agents execute concurrently.

### Collection Behavior

Metrics collection follows a predictable lifecycle to ensure accurate tracking and persistence:

- **Collection timing**: Metrics are collected after each agent run completes, regardless of whether the run succeeded or failed
- **Persistence across restarts**: The `metrics.json` file is loaded on scheduler startup, preserving historical metrics across scheduler restarts and system reboots
- **Automatic saving**: Metrics are automatically saved to disk after each agent run, ensuring no data is lost
- **Resetting metrics**: To reset all metrics, delete the `metrics.json` file (e.g., `rm .switchboard/logs/metrics.json`); a new file will be created on the next agent run
- **Viewing metrics**: Use the `switchboard metrics` command to view current metrics (see the [Usage](#usage) section for options)

This design ensures that metrics provide a comprehensive, long-term view of agent performance while maintaining data integrity.

### Error Handling

The metrics system is designed to be resilient and never interrupt scheduler operation. Errors during metrics operations are logged but do not affect agent execution.

#### Corrupted Metrics File

If the metrics file (`<log_dir>/metrics.json`) becomes corrupted or contains invalid JSON:
- The corrupted file is automatically backed up to `metrics.json.backup.<timestamp>`
- A warning is logged to stderr with the backup file path
- The scheduler continues running (new empty metrics are created on next save)
- No agent runs are interrupted

**Recovery steps:**
1. Check the backup file: `ls -la <log_dir>/metrics.json.backup.*`
2. If a backup is valid, restore it: `cp <log_dir>/metrics.json.backup.<timestamp> <log_dir>/metrics.json`
3. If all backups are corrupted, delete the corrupted file and let the scheduler start fresh

#### Missing Metrics File

If the metrics file doesn't exist on the first run:
- No error is raised - this is expected behavior
- The first call to `MetricsStore::save()` creates a new `metrics.json` file
- No user intervention is required

#### Metrics Update Failures

If metrics update or save operations fail during scheduler execution:
- Errors are logged using the tracing logger but do not fail the scheduler
- The scheduler continues running and processing agents
- Agent runs are not interrupted by metrics failures
- This design ensures agent reliability even if the metrics subsystem has issues

#### Atomic Write Guarantees

`MetricsStore::save()` uses an atomic write pattern to prevent partial writes:
- Data is first written to a temporary file (`metrics.json.tmp`)
- Only if the write succeeds, the temporary file is renamed to `metrics.json`
- This prevents data corruption if the process crashes mid-write
- If a crash occurs during write, only the temporary file may be incomplete; the original `metrics.json` remains intact

### Status Indicators

The status column provides a quick visual indicator of agent health based on success rate and run count:

| Icon | Condition | Meaning |
|------|-----------|---------|
| ✓ | Success rate ≥ 95% and ≥3 runs | Agent is performing well |
| ⚠ | Success rate 50-95% and ≥3 runs | Agent has issues that need attention |
| ✗ | Success rate < 50% and ≥3 runs | Agent is unreliable, investigate immediately |
| - | Fewer than 3 runs | Insufficient data to assess |

---

## Requirements

- **Docker** daemon running and accessible (required for agent container execution)
- **Rust** 2021 edition or later (for building from source)
- **Git** (for cloning the repository)

---

## Development Setup

### Prerequisites

- Rust 2021 edition or later
- Docker daemon running and accessible
- Git for version control

### Building the Project

```bash
# Clone the repository
git clone https://github.com/yourusername/switchboard.git
cd switchboard

# Build in debug mode
cargo build

# Build in release mode
cargo build --release

# Run tests
cargo nextest run

# Run tests with coverage
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Development Commands

```bash
# Run the CLI in development
cargo run -- up --help
cargo run -- validate

# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check without building
cargo check
```

### Testing

The project uses a comprehensive testing strategy:

```bash
# Run all tests
cargo nextest run

# Run only unit tests
cargo nextest run --lib

# Run integration tests (requires Docker)
cargo nextest run --features integration

# Generate HTML coverage report
cargo llvm-cov --html --output-dir coverage/
```

### Coding Standards and Conventions

#### Rust Coding Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) for public APIs
- Prefer idiomatic Rust patterns over porting patterns from other languages
- Use meaningful variable and function names that clearly describe their purpose
- Leverage Rust's ownership system and borrow checker to ensure memory safety
- Avoid unwrapping `Option` and `Result` types without proper error handling
- Use pattern matching and destructuring to make code more readable
- Prefer iterators over imperative loops when possible
- Use `Option` and `Result` types instead of null values and exceptions
- Apply the builder pattern for complex struct initialization
- Keep functions focused and small (ideally under 50 lines)

#### Formatting Requirements

- Run `cargo fmt` before committing any code changes
- Ensure all code passes `rustfmt` checks without warnings
- Use the default `rustfmt` configuration unless project-specific overrides exist
- Format code automatically in your IDE (VS Code, IntelliJ IDEA, etc.) with appropriate extensions
- The CI pipeline will fail if formatting does not meet standards
- Run `cargo fmt -- --check` in CI to verify formatting without modifying files

#### Linting Requirements

- Run `cargo clippy` before committing to catch common mistakes and improve code quality
- Address all clippy warnings before pushing code
- Use `cargo clippy -- -W clippy::all` for the strictest linting
- Suppress clippy warnings only with a clear justification in a comment
- The CI pipeline runs clippy with warnings-as-errors configuration
- Pay special attention to performance-related lints (`clippy::perf`)
- Review complexity lints (`clippy::cognitive_complexity`) and refactor when needed

#### Code Review Guidelines

- All code must undergo code review before merging into the main branch
- At least one maintainer approval is required for PRs
- Address all review comments before requesting re-review
- Be responsive to review feedback within a reasonable timeframe
- Keep PRs small and focused to facilitate thorough reviews
- Reviewers should focus on correctness, style, and architectural consistency
- Use GitHub's review features to provide line-by-line feedback
- Be constructive and respectful in all review communications

#### Documentation Standards

- Document all public APIs with Rustdoc comments (`///`)
- Include examples in documentation for complex or non-obvious APIs
- Explain non-trivial logic with inline comments (`//`)
- Keep documentation in sync with code changes
- Use `#[doc]` attributes for module-level documentation
- Include parameter descriptions, return value information, and error conditions
- Add `# Examples` sections with runnable code samples where appropriate
- Run `cargo doc` to verify documentation builds without warnings
- Keep comments concise, accurate, and up-to-date

#### Testing Conventions

- Write unit tests for all public functions and methods
- Maintain code coverage above 80% for new code
- Use descriptive test names that clearly indicate what is being tested
- Follow the Arrange-Act-Assert (AAA) pattern in test structure
- Use `cargo nextest` for faster test execution
- Mock external dependencies (Docker, filesystem, network) in unit tests
- Write integration tests for critical workflows and cross-module interactions
- Include edge cases, error conditions, and boundary values in tests
- Make tests independent and deterministic (avoid reliance on shared state)

#### Commit Message Conventions

- Use [Conventional Commits](https://www.conventionalcommits.org/) format
- Format: `<type>(<scope>): <description>`
- Types: `feat` (new feature), `fix` (bug fix), `docs` (documentation), `style` (formatting), `refactor` (code restructuring), `test` (adding tests), `chore` (maintenance tasks), `perf` (performance improvements)
- Examples:
  - `feat(cli): add --verbose flag for detailed output`
  - `fix(scheduler): handle timezone conversion correctly`
  - `docs(readme): update installation instructions`
  - `test(metrics): add coverage for collector module`
- Write descriptions in the imperative mood ("add" not "added" or "adds")
- Keep the first line under 72 characters
- Add a body paragraph if needed to explain the "why" and "how"

#### Pull Request Guidelines

- PRs should be focused on a single concern or feature
- Include appropriate tests for new functionality
- Update documentation (README, API docs, inline comments) as needed
- Link to related issues in the PR description
- Ensure all CI checks pass before requesting review
- Squash commits into logical units before merging
- Use a clear, descriptive PR title following the commit message convention
- Provide context and motivation in the PR description
- Break large changes into smaller, reviewable PRs when possible

#### Naming Conventions

- Use `snake_case` for variables, functions, and module names
- Use `PascalCase` (or `UpperCamelCase`) for types, structs, enums, and traits
- Use `SCREAMING_SNAKE_CASE` for constants and static variables
- Use `kebab-case` for command-line arguments and file names
- Prefix private helper functions with `_` if they're not intended for external use
- Use descriptive names that convey purpose without abbreviations
- Avoid single-letter variable names except in loop iterators
- Use `is_`, `has_`, `can_` prefixes for boolean-returning functions
- Example:
  - Variable: `agent_container_id`
  - Function: `validate_configuration()`
  - Type: `AgentConfig`
  - Constant: `MAX_RETRIES`
  - Module: `docker_management`

---

## Code Coverage

Code coverage measures how much of the codebase is executed by tests. It's essential for maintaining code quality, identifying untested code paths, and ensuring reliability as the project evolves.

### Installation

Install cargo-llvm-cov to enable coverage analysis:

```bash
cargo install cargo-llvm-cov
```

### Running Tests with Coverage

To run tests with coverage analysis and generate an lcov report (for CI/CD integration):

```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Generating HTML Report

For local development, generate an interactive HTML coverage report:

```bash
cargo llvm-cov --html --output-dir coverage/
```

This creates a detailed HTML report in the `coverage/` directory that you can open in your browser to explore coverage by file and line.

### Viewing Coverage Results

Open the HTML report using your platform's default browser:

```bash
# macOS
open coverage/index.html

# Linux
xdg-open coverage/index.html

# Windows
start coverage/index.html
```

### Coverage Targets

The project aims to meet these coverage thresholds:

| Metric Type | Minimum | Target |
|-------------|---------|--------|
| Overall line coverage | 70% | 85% |
| Overall branch coverage | 60% | 75% |
| New code (per PR) | 80% | 90% |

These targets are based on the criticality of each module and its testability. See [`PRD.md`](PRD.md:462-495) for detailed requirements.

### Coverage Exclusions

**Note**: `cargo-llvm-cov 0.6` with stable Rust does not support attribute-based code exclusion (e.g., `#[cfg(not(tarpaulin_include))]` or `#[coverage(off)]`). The only available exclusion mechanism requires unstable nightly features (`#![feature(coverage_attribute)]`).

As a result, we accept the following in coverage metrics:
- Generated code (e.g., derive macros)
- Unreachable code paths (e.g., error handling branches)
- The `main()` function

This is standard practice and does not affect the overall quality goals.

### Per-Module Expectations

Different modules have different coverage targets based on testability and complexity:

| Module | Expected Coverage | Notes |
|--------|-------------------|-------|
| CLI and commands | 80-90% | Highly testable unit tests |
| Scheduler | 75-85% | Core logic, requires careful testing |
| Docker integration | 70-80% | Limited by integration test constraints |
| Metrics collection | 70-85% | Moderate testability |
| Logging utilities | 60-75% | Lower priority for comprehensive coverage |

See [`PRD.md`](PRD.md:462-495) for complete module-specific requirements.

### CI Integration

Coverage reports are automatically uploaded to [Codecov](https://codecov.io/) on:
- Every push to the `main` branch
- Every pull request

The codecov badge at the top of this README displays the current coverage status.

---

## Architecture Overview

This section describes the high-level architecture of switchboard, including component architecture, data flow, container execution model, and logging structure.

### Component Architecture

Switchboard is organized into four main modules, each with specific responsibilities:

- **CLI Module** ([`src/cli/mod.rs`](src/cli/mod.rs)):
  - **Purpose**: Parses command-line arguments using clap, orchestrates command execution, manages scheduler lifecycle, and handles PID file management. Provides the user interface and delegates tasks to appropriate handlers.
  - **Key Structures**: `Cli` (main CLI structure), `Commands` enum (variants: `Up`, `Run`, `Build`, `List`, `Logs`, `Metrics`, `Down`, `Validate`)
  - **Key Functions**: `run()` (dispatch logic), `run_up()`, `run_run()`, `check_image_exists()`, `is_process_running()`
  - **Dependencies**: clap, bollard, tokio

- **Scheduler Module** ([`src/scheduler/mod.rs`](src/scheduler/mod.rs)):
  - **Purpose**: Evaluates cron expressions, triggers agent executions, manages task scheduling and lifecycle, and handles overlapping executions (Skip/Queue modes). Coordinates when and how tasks are executed based on configured schedules.
  - **Key Structures**: `Scheduler`, `RunStatus`, `QueuedRun`, `ScheduledAgent`
  - **Key Functions**: `new()`, `register_agent()`, `start()`, `stop()`, `execute_agent()`, `process_queued_run()`
  - **Dependencies**: chrono, chrono-tz, tokio, tokio-cron-scheduler, uuid

- **Docker Module** ([`src/docker/mod.rs`](src/docker/mod.rs)):
  - **Purpose**: Builds Docker images, manages container lifecycle via Docker Engine API, and handles container creation, execution, stream management, and timeout. Manages all interactions with Docker for running Kilo Code instances.
  - **Key Structures**: `DockerClient`, `DockerError`, `AgentExecutionResult`, `ContainerConfig`, `StreamConfig`
  - **Key Functions**: `new()`, `check_available()`, `build_agent_image()`, `run_agent()`, `attach_and_stream_logs()`, `wait_for_exit()`
  - **Dependencies**: bollard, tokio, bytes, futures, flate2, tar, anyhow

- **Logger Module** ([`src/logger/mod.rs`](src/logger/mod.rs)):
  - **Purpose**: Captures and aggregates logs from all agent runs, manages log file management and rotation, and provides terminal interleaved output with agent-name prefixes. Manages log collection from containers with both terminal and file output.
  - **Key Structures**: `Logger`, `FileWriter`, `TerminalWriter`
  - **Key Functions**: `new()`, `write_terminal_output()`, `write_agent_log()`, directory creation, log rotation, timestamp generation
  - **Dependencies**: chrono, thiserror

### Data Flow and Interaction Between Modules

Switchboard's architecture follows a clear data flow through its modules:

1. **CLI Parsing**: The CLI module parses user commands and delegates to the appropriate command handlers in the Commands module.

2. **Command Execution**: Commands interact with the Scheduler module to request task scheduling based on user configuration.

3. **Task Scheduling**: The Scheduler coordinates execution timing and manages the lifecycle of Kilo Code instances, ensuring tasks are executed according to their schedules.

4. **Container Execution**: The Docker module handles all container operations—creating, running, and managing Kilo Code instances based on Scheduler requests.

5. **Log Streaming**: The Docker module streams real-time logs from running containers through the Logger module, which handles both terminal output and persistent file storage.

6. **Metrics Collection**: The Metrics module collects execution metrics throughout the lifecycle and stores them for later retrieval and analysis.

This modular design ensures clear separation of concerns while enabling efficient data flow from user input to container execution and back.

### Container Execution Model

Switchboard's Docker module executes Kilo Code containers through a structured process:

1. **Container Creation**: The Docker module creates containers using the configured Kilo Code image, initializing each instance with appropriate settings and resource allocations.

2. **Volume Mounting**: The workspace directory is mounted into containers as a volume, allowing Kilo Code instances to access project files and produce outputs within the shared workspace.

3. **Environment Configuration**: Environment variables are passed to containers to configure Kilo Code behavior, including prompts, timeouts, and execution parameters as defined in the configuration.

4. **Stream Capture**: Container stdout and stderr streams are captured in real-time using Docker's attach mechanism, enabling immediate log streaming to both terminal output and persistent storage.

5. **Timeout Handling**: A timeout mechanism monitors container execution duration. When a container exceeds its configured timeout, it is terminated gracefully to prevent resource exhaustion and ensure predictable execution times.

6. **Log File Writing**: Captured stream output is written to log files in configured storage locations, maintaining a persistent record of each execution session for later analysis and debugging.

This execution model ensures reliable, isolated, and monitored container lifecycles while maintaining seamless integration with the rest of Switchboard's architecture.

### Log File Structure

Switchboard maintains a comprehensive logging system for all container execution sessions:

1. **Logs Directory**: Log files are stored in a dedicated logs directory, providing centralized access to execution records and ensuring organized file management across all task runs.

2. **Unique File Naming**: Each execution run generates a log file with a unique identifier, typically incorporating timestamps and task metadata to prevent conflicts and enable precise log file identification.

3. **Stream Capture**: Log files capture both stdout and stderr streams from container execution, preserving the complete output from Kilo Code instances for comprehensive analysis and debugging.

4. **Format with Timestamps and Stream Indicators**: Log entries include timestamps to track the progression of execution events and stream indicators to distinguish between stdout (standard output) and stderr (error) streams, facilitating efficient log analysis and troubleshooting.

5. **Debugging and Audit Trails**: These persistent log files serve as essential resources for debugging execution issues, auditing task history, and reviewing the detailed output of previous container runs without requiring re-execution.

This structured logging approach ensures that all execution sessions are thoroughly documented and easily accessible for post-mortem analysis, system monitoring, and continuous improvement of the Switchboard workflow.

## Project Structure

```
switchboard/
├── Cargo.toml                      # Project manifest and dependencies
├── Dockerfile                      # Agent container image definition
├── README.md                       # This file
├── ARCHITECTURE.md                 # Detailed architecture documentation
├── PRD.md                          # Product Requirements Document
├── TODO.md                         # Current sprint tasks
├── COMPLETED.md                    # Completed work items
├── BACKLOG.md                      # Future work and enhancements
├── BLOCKERS.md                     # Current and resolved blockers
│
├── src/
│   ├── main.rs                     # Binary entry point
│   ├── lib.rs                      # Library module exports
│   │
│   ├── cli/
│   │   └── mod.rs                  # CLI command definitions and handlers
│   │
│   ├── config/
│   │   └── mod.rs                  # Configuration parsing and validation
│   │
│   ├── scheduler/
│   │   └── mod.rs                  # Cron scheduler (stub)
│   │
│   ├── docker/
│   │   └── mod.rs                  # Docker client (stub)
│   │
│   └── logger/
│       └── mod.rs                  # Logging utilities (stub)
│
└── tests/
    └── common.rs                   # Common test utilities
```

---

## Configuration Reference

### switchboard.toml Schema

```toml
version = "0.1.0"

# Global defaults (optional)
[settings]
image_name = "switchboard-agent"        # Docker image name
image_tag = "latest"                # Docker image tag
log_dir = ".switchboard/logs"           # Log output directory
workspace_path = "."                # Path to mount into container
timezone = "America/New_York"       # Timezone for cron evaluation

# Define your agents
[[agent]]
name = "agent-name"
schedule = "0 */6 * * *"            # Cron expression
prompt = "Your inline prompt here"
# OR
prompt_file = "prompts/agent.md"     # Path to prompt file
readonly = false                    # Read-only mount (default: false)
timeout = "30m"                     # Max runtime (default: "30m")
env = { KEY = "value" }             # Additional environment variables

[[agent]]
name = "another-agent"
schedule = "0 2 * * 1"              # Mondays at 2 AM
prompt_file = "prompts/weekly.md"
readonly = true
timeout = "1h"
```

### [settings] - Global Settings

The `[settings]` section configures global settings that apply to all agents. All fields are optional.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `image_name` | `String` | `"switchboard-agent"` | Docker image name for agent containers |
| `image_tag` | `String` | `"latest"` | Docker image tag version |
| `log_dir` | `String` | `".switchboard/logs"` | Directory for storing log files |
| `workspace_path` | `String` | `"."` | Path to mount into agent containers as workspace |
| `timezone` | `String` | `"system"` | Timezone for cron schedule evaluation (use IANA format like `"America/New_York"`; `"system"` uses system timezone) |
| `overlap_mode_str` | `String` | `"skip"` | String representation of overlap mode (validated as `"skip"` or `"queue"`) |

**Notes:**

- **Timezone format**: Use IANA timezone identifiers (e.g., `"America/New_York"`, `"Europe/London"`, `"Asia/Tokyo"`). Set to `"system"` to use the system's timezone.
- **Overlap mode**: This is a global default that can be overridden per agent. Use `"skip"` to skip overlapping executions or `"queue"` to queue them for later execution.

### [[agent]] - Agent Configuration

The `[[agent]]` section configures individual agents. At least one agent is required.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | `String` | `""` | Unique identifier for the agent (required, cannot be empty) |
| `prompt` | `Option<String>` | `None` | Inline prompt text (exactly one of `prompt` or `prompt_file` must be provided) |
| `prompt_file` | `Option<String>` | `None` | Path to prompt file relative to config directory (exactly one of `prompt` or `prompt_file` must be provided) |
| `schedule` | `String` | `""` | Cron expression for agent execution (5-field Unix cron format) |
| `env` | `Option<HashMap<String, String>>` | `Some(HashMap::new())` | Additional environment variables as KEY=VALUE pairs |
| `readonly` | `Option<bool>` | `None` | Whether the agent runs in read-only mode (no filesystem writes) |
| `timeout` | `Option<String>` | `Some("30m")` | Maximum execution duration (format: `"Ns"`, `"Nm"`, `"Nh"` where N is a number) |
| `overlap_mode` | `Option<OverlapMode>` | `None` | Agent-level override for overlap mode (`Skip` or `Queue`) |
| `max_queue_size` | `Option<usize>` | `None` | Queue size limit for Queue mode (default: 3) |
| `skills` | `Option<Vec<String>>` | `None` | List of skills to install inside the agent container at startup (format: "owner/repo" or "owner/repo@skill-name") |

#### Validation Rules

- **`name`**: Must be non-empty and unique across all agents
- **`prompt` / `prompt_file`**: Exactly one of these fields must be provided
  - If using `prompt_file`, the file must exist at the specified path relative to the config directory
- **`timeout`**: Must be a valid format with a positive number
- **`schedule`**: Must be a valid 5-field Unix cron format

#### Timeout Format Reference

| Format | Example | Result |
|--------|---------|--------|
| `"Ns"` | `"30s"` | 30 seconds |
| `"Nm"` | `"5m"` | 5 minutes (300 seconds) |
| `"Nh"` | `"1h"` | 1 hour (3600 seconds) |

**Note**: The `overlap_mode` field can be used to override the global overlap mode setting (`overlap_mode_str` in `[settings]`) on a per-agent basis.

### Cron Expression Format

The `schedule` field uses the standard 5-field Unix cron format:

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6, Sunday = 0)
│ │ │ │ │
* * * * *
```

Each field can contain: `*` (any value), `N` (specific value), `N-M` (range), `*/N` (every N), `N,M,O` (list).

| Expression | Description |
|------------|-------------|
| `0 */6 * * *` | Every 6 hours at minute 0 |
| `0 9 * * *` | Daily at 9:00 AM |
| `0 2 * * 1` | Every Monday at 2:00 AM |
| `*/30 * * * *` | Every 30 minutes |
| `0 0 * * 0` | Every Sunday at midnight |
| `15 4 * * *` | Daily at 4:15 AM |
| `0 0 1 * *` | On the 1st of every month at midnight |
| `0 */2 * * *` | Every 2 hours at minute 0 |

| Special Character | Description | Example |
|-------------------|-------------|---------|
| `*` | Any value | `* * * * *` (every minute) |
| `-` | Range of values | `0 9-17 * * *` (hours 9-17) |
| `/` | Step value (every N) | `*/15 * * * *` (every 15 minutes) |
| `,` | List of values | `0 9,12,18 * * *` (at 9, 12, and 18) |

**Note**: Day of week numbering uses 0 = Sunday, 1 = Monday, ..., 6 = Saturday.

### Overlap Modes

Overlap modes control how the scheduler handles scheduled runs when an agent is already executing. You can configure this globally in `[settings]` or override per agent in `[[agent]]`.

| Mode | Alias | Description |
|------|--------|-------------|
| `Skip` | `"skip"` | When an agent is already running, skip new runs and log a warning (default per PRD §9) |
| `Queue` | `"queue"` | Add new runs to a queue and execute them sequentially after the current run completes |

#### Skip Mode (Default)

Skip mode prevents concurrent runs of the same agent. If a scheduled run starts while the agent is already executing, it's skipped and a warning is logged. Use this for:
- Stateful operations where concurrent execution could cause conflicts
- Agents that modify files or external state
- Scenarios where resource constraints require strict serialization

Example configuration:
```toml
[settings]
overlap_mode_str = "skip"  # This is the default

[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review recent changes"
# Uses skip mode from global settings
```

#### Queue Mode

Queue mode runs are queued and executed one at a time. The queue has a maximum size (default: 3, configurable via `max_queue_size`). When the queue is full, additional runs are skipped. Use this when you want to ensure all runs execute eventually but want to prevent unbounded queue growth.

**Queue behavior:**
- When a scheduled run triggers while the agent is running, it's added to the queue
- Queued runs execute sequentially in order after the current run completes
- When `max_queue_size` is reached, new runs are skipped with a warning
- The queue is per-agent (each agent maintains its own queue)

Example configuration:
```toml
[settings]
overlap_mode_str = "queue"
max_queue_size = 3  # Default is 3

[[agent]]
name = "doc-updater"
schedule = "0 */4 * * *"
prompt = "Update documentation"
# Uses queue mode from global settings with max_queue_size = 3
```

Per-agent override:
```toml
[settings]
overlap_mode_str = "skip"  # Global default

[[agent]]
name = "data-processor"
schedule = "*/15 * * * *"
prompt = "Process new data"
overlap_mode = "Queue"      # Override: use queue mode for this agent
max_queue_size = 5          # Optional: override max queue size
```

**Note:** `max_queue_size` only applies to Queue mode. The default value is 3, which means up to 3 runs can be queued while the current run is executing. Additional runs beyond this limit are skipped with a warning.

#### Configuration Hierarchy

Overlap mode configuration follows this hierarchy:
1. **Global default**: `settings.overlap_mode_str` (defaults to `"skip"`)
2. **Per-agent override**: `agent.overlap_mode` (overrides the global setting if specified)

If an agent specifies `overlap_mode`, it takes precedence over the global `settings.overlap_mode_str`. Otherwise, the agent inherits the global setting.

### Cron Expression Reference

Switchboard uses standard 5-field cron expressions:

```
┌───────────── minute (0 - 59)
│ ┌───────────── hour (0 - 23)
│ │ ┌───────────── day of month (1 - 31)
│ │ │ ┌───────────── month (1 - 12)
│ │ │ │ ┌───────────── day of week (0 - 6, Sunday = 0)
│ │ │ │ │
* * * * *
```

**Examples:**

| Expression | Description |
|------------|-------------|
| `0 */6 * * *` | Every 6 hours at minute 0 |
| `0 9 * * *` | Daily at 9:00 AM |
| `0 2 * * 1` | Every Monday at 2:00 AM |
| `*/30 * * * *` | Every 30 minutes |
| `0 0 * * 0` | Every Sunday at midnight |

For complete configuration details, see [`PRD.md §6`](PRD.md#6-configuration-file).

---

## CLI Command Reference

Switchboard provides a comprehensive CLI for managing agent scheduling, execution, and monitoring. The following commands are available:

### `switchboard up`

Build agent image and start the scheduler daemon.

**Syntax:**
```bash
switchboard up [OPTIONS]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |
| `--detach` | `-d` | Run in background | `false` |

**Usage examples:**
```bash
# Start scheduler in foreground
switchboard up

# Start scheduler in background
switchboard up --detach

# Use custom configuration file
switchboard up --config /path/to/custom.toml --detach
```

---

### `switchboard run`

Immediately execute a single agent (bypasses scheduler).

**Syntax:**
```bash
switchboard run [OPTIONS] <AGENT_NAME>
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |

**Positional:**
| Argument | Description |
|----------|-------------|
| `AGENT_NAME` | Name of the agent to execute (required) |

**Usage examples:**
```bash
# Execute an agent immediately
switchboard run my-agent

# Execute agent with custom config
switchboard run code-reviewer --config /path/to/config.toml
```

---

### `switchboard build`

Build Docker images for agents defined in the configuration file.

**Syntax:**
```bash
switchboard build [OPTIONS]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |
| `--no-cache` | | Build without using Docker cache | `false` |

**Usage examples:**
```bash
# Build using default config
switchboard build

# Build without cache
switchboard build --no-cache

# Build with custom config
switchboard build --config /path/to/custom.toml
```

---

### `switchboard list`

Print all configured agents with their schedules and prompts.

**Syntax:**
```bash
switchboard list [OPTIONS]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |

**Usage examples:**
```bash
# List all agents with default config
switchboard list

# List agents with custom config
switchboard list --config /path/to/custom.toml
```

---

### `switchboard logs`

View logs from agent runs or the scheduler.

**Syntax:**
```bash
switchboard logs [OPTIONS] [AGENT_NAME]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |
| `--follow` | `-f` | Stream logs as they are generated | `false` |
| `--tail LINES` | `-t LINES` | Show the last N lines | `50` |

**Positional:**
| Argument | Description |
|----------|-------------|
| `AGENT_NAME` | Name of the agent to view logs for (optional, defaults to scheduler logs) |

**Usage examples:**
```bash
# View scheduler logs
switchboard logs

# View specific agent logs
switchboard logs my-agent

# Follow logs in real-time
switchboard logs my-agent --follow

# View last 100 lines of logs
switchboard logs my-agent --tail 100

# Follow and tail combined
switchboard logs my-agent --follow --tail 100
```

---

### `switchboard down`

Stop the scheduler daemon and any running agent containers.

**Syntax:**
```bash
switchboard down [OPTIONS]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |
| `--cleanup` | `-c` | Remove all associated resources (logs, PID files, etc.) | `false` |

**Usage examples:**
```bash
# Stop scheduler and containers
switchboard down

# Stop and clean up all resources
switchboard down --cleanup

# Stop with custom config
switchboard down --config /path/to/custom.toml
```

---

### `switchboard validate`

Validate the configuration file (syntax, cron schedules, and agent definitions).

**Syntax:**
```bash
switchboard validate [OPTIONS]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |

**Usage examples:**
```bash
# Validate default configuration
switchboard validate

# Validate custom configuration
switchboard validate --config /path/to/custom.toml
```

---

### `switchboard metrics`

Display agent execution metrics and statistics.

**Syntax:**
```bash
switchboard metrics [OPTIONS]
```

**Arguments:**
| Argument | Short | Description | Default |
|----------|-------|-------------|---------|
| `--config PATH` | `-c PATH` | Path to configuration file | `./switchboard.toml` |
| `--detailed` | `-d` | Show detailed metrics view | `false` |
| `--agent NAME` | | Show metrics for a specific agent only | `null` |

**Usage examples:**
```bash
# Display summary metrics
switchboard metrics

# Display detailed metrics
switchboard metrics --detailed

# Show metrics for a specific agent
switchboard metrics --agent my-agent

# Detailed view for specific agent
switchboard metrics --detailed --agent my-agent
```

---

### Global Options

The following options are available across all commands:

| Option | Short | Description |
|--------|-------|-------------|
| `--config PATH` | `-c PATH` | Path to configuration file (default: `./switchboard.toml`) |
| `--help` | `-h` | Display help message for the command |
| `--version` | `-V` | Display version information |

**Usage examples:**
```bash
# Show version
switchboard --version

# Show help for specific command
switchboard up --help
switchboard logs --help

# Use custom config with any command
switchboard --config /custom/path/switchboard.toml list
switchboard -c /custom/path/switchboard.toml validate
```

---

## Project Status

### Current Implementation (Sprint 0)

| Module | Status | Completion |
|--------|--------|------------|
| **Config Parser** | ✅ Fully Implemented | 95% |
| **CLI Framework** | ✅ Framework Complete | 60% |
| **Docker Client** | 🚧 Stub Only | 0% |
| **Scheduler** | 🚧 Stub Only | 0% |
| **Logger** | 🚧 Stub Only | 0% |

### What's Working Now

- ✅ Rust project structure with all dependencies configured
- ✅ CLI command definitions with clap-based argument parsing
- ✅ Configuration module with comprehensive TOML parsing and validation
- ✅ Dockerfile for agent containers based on `node:22-slim` with Kilo Code CLI
- ✅ Basic command handlers (stub implementations)
- ✅ 40+ unit tests for the config module

### In Progress

- 🚧 Docker client implementation (Sprint 1)
- 🚧 Logger module implementation (Sprint 1)
- 🚧 Cron scheduler implementation (Sprint 2)
- 🚧 CLI command handler implementations (Sprints 1-3)

### Planned

- 📋 Full integration test suite
- 📋 CI/CD pipeline with coverage reporting
- 📋 Documentation and examples
- 📋 crates.io publishing

### Blockers

🎉 **No active blockers!** The project is moving forward smoothly.

---

## Roadmap

### v0.1.0 (Current - Work in Progress)

- [x] Project setup and scaffolding
- [x] CLI command framework
- [x] Configuration parsing and validation
- [ ] Docker client implementation
- [ ] Logger module implementation
- [ ] Cron scheduler implementation
- [ ] Integration test suite
- [ ] Documentation

### Future Enhancements

- 🌐 Web UI / Dashboard for monitoring
- 🔗 Webhook triggers (git push, PR events)
- 🔄 Agent chaining (output feeds into next prompt)
- ☁️ Remote execution on Docker hosts or Kubernetes
- 🔐 Secret management integration (Vault, AWS Secrets Manager)
- 🔄 Config hot-reload
- 🔔 Notifications (Slack, email, webhooks)
- 🐳 Per-agent Dockerfile overrides
- 📦 Queue mode for overlapping runs

---

## Contributing

Contributions are welcome! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch**: `git checkout -b feature/amazing-feature`
3. **Make your changes** with proper tests
4. **Run tests**: `cargo nextest run`
5. **Run linter**: `cargo clippy -- -D warnings`
6. **Format code**: `cargo fmt`
7. **Commit your changes**: `git commit -m "Add amazing feature"`
8. **Push to branch**: `git push origin feature/amazing-feature`
9. **Open a Pull Request**

### Development Guidelines

- Follow Rust naming conventions and idiomatic style
- Write tests for all new functionality (target: 80%+ coverage)
- Update documentation for any user-facing changes
- Ensure all CI checks pass before submitting PR

---

## Resources

- **Project Documentation**
  - [Product Requirements Document](PRD.md) - Detailed requirements and feature specifications
  - [Architecture Documentation](ARCHITECTURE.md) - System design and technical architecture
  - [Installation Troubleshooting Guide](docs/INSTALLATION_TROUBLESHOOTING.md) - Common installation issues and solutions
  - [Platform Compatibility](docs/PLATFORM_COMPATIBILITY.md) - Supported platforms and environments
  - [crates.io Publishing Guide](docs/CRATES_IO_PUBLISHING.md) - Guide for publishing to crates.io
  - [Changelog](CHANGELOG.md) - Version history and release notes

- **External Documentation**
  - [Official Rust Documentation](https://doc.rust-lang.org/) - Comprehensive Rust language documentation
  - [Docker Documentation](https://docs.docker.com/) - Container platform documentation
  - [Kilo Code CLI Documentation](https://www.npmjs.com/package/@kilocode/cli) - AI coding agent runner documentation

---

## License

This project is licensed under the MIT License - see the LICENSE file for details.

---

## Acknowledgments

- [Kilo Code CLI](https://www.npmjs.com/package/@kilocode/cli) — The AI coding agent runner
- [Docker](https://www.docker.com/) — Container platform for isolated execution
- [Rust](https://www.rust-lang.org/) — The language that makes this possible
- All contributors who help make Switchboard better

---

## Support

- 📧 Email: [your-email@example.com]
- 🐛 Issues: [GitHub Issues](https://github.com/yourusername/switchboard/issues)
- 💬 Discussions: [GitHub Discussions](https://github.com/yourusername/switchboard/discussions)

---

<div align="center">

**Made with ❤️ by the Switchboard team**

If you find this project useful, please consider giving it a ⭐ on GitHub!

</div>
