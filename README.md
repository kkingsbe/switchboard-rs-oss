# Switchboard

<p align="center">
  <b>Cron for AI coding agents</b><br>
  Schedule and run AI coding agents with Docker isolation, cron-based scheduling, and Discord integration.
</p>

<p align="center">

[![Build](https://github.com/switchboard-ai/switchboard/actions/workflows/build.yml/badge.svg)](https://github.com/switchboard-ai/switchboard/actions/workflows/build.yml)
[![Crates.io](https://img.shields.io/crates/v/switchboard.svg)](https://crates.io/crates/switchboard)
[![Discord](https://img.shields.io/discord/123456789?label=Discord)](https://discord.gg/switchboard)

</p>

---

## tl;dr

```bash
# 1. Install
cargo install switchboard

# 2. Create config (switchboard.toml)
echo '[settings]
image_name = "kilosynth/prompter:latest"

[[agent]]
name = "daily-coder"
schedule = "0 9 * * *"
prompt = "Review PRs and summarize changes."' > switchboard.toml

# 3. Run
switchboard up
```

---

## What is Switchboard?

Switchboard is a tool for **scheduling and running AI coding agents** on your own infrastructure. It combines the simplicity of cron with the power of containerized AI agents.

### Why use Switchboard?

- **Scheduled Automation** — Run AI agents on cron schedules (hourly, daily, weekly)
- **Docker Isolation** — Each agent runs in an isolated container with its own workspace
- **Discord Integration** — Conversational AI bot for your Discord server
- **Skills System** — Reusable, versioned agent configurations

### What it does

Switchboard reads a TOML configuration file, schedules AI agent runs based on cron expressions, and executes them inside Docker containers. Each agent receives a prompt and optional skills, then executes within the configured timeout and overlap constraints.

---

## Quick Start

### Step 1: Ensure Docker is running

> ⚠️ **Prerequisite**: Docker must be running

```bash
docker --version
```

### Step 2: Install Switchboard

```bash
cargo install switchboard
```

### Step 3: Create a configuration file

Create `switchboard.toml` in your project:

```toml
[settings]
image_name = "kilosynth/prompter:latest"

[[agent]]
name = "daily-coder"
schedule = "0 9 * * *"
prompt = "Review any open PRs and summarize the changes."
```

### Step 4: Validate your configuration

> ✅ **Tip**: Use `switchboard validate` before running

```bash
switchboard validate
```

### Step 5: Start the scheduler

```bash
switchboard up
```

The scheduler will run in the foreground. Press `Ctrl+C` to stop.

---

## Installation

### Prerequisites

| Dependency | Version | Purpose |
|------------|---------|---------|
| **Docker** | 20.10+ | Container runtime for agent isolation |
| **Rust** | 1.70+ | Required for `cargo install` |

### Install from Crates.io

```bash
cargo install switchboard
```

### Install from Source

```bash
git clone https://github.com/switchboard-ai/switchboard.git
cd switchboard
cargo install --path .
```

### Verify Installation

```bash
switchboard --version
```

---

## Core Concepts

### Agents

An **agent** is a scheduled AI task defined in your configuration. Each agent has:

- A unique `name`
- A `schedule` (cron expression)
- A `prompt` or `prompt_file`

### Schedules

Schedules use **6-field cron expressions**:

```
second minute hour day month weekday
```

Examples:
- `"0 0 * * * *"` — Every hour on the hour
- `"0 */15 * * * *"` — Every 15 minutes
- `"0 9 * * 1-5"` — 9:00 AM on weekdays

### Docker Isolation

Each agent runs inside an isolated Docker container with:

- Its own filesystem workspace (mounted from your project)
- Environment variables you specify
- Configurable timeout and overlap handling

### Overlap Modes

- **`skip`** (default) — Skip new runs if agent is already running
- **`queue`** — Queue new runs for sequential execution after current run completes

---

## Configuration

### Minimal Example

```toml
[settings]
image_name = "kilosynth/prompter:latest"

[[agent]]
name = "my-agent"
schedule = "0 9 * * *"
prompt = "Your task description here."
```

### Full Example

```toml
[settings]
image_name = "kilosynth/prompter:latest"
image_tag = "latest"
timezone = "America/New_York"
overlap_mode = "skip"
log_dir = ".switchboard/logs"

[[agent]]
name = "comprehensive-agent"
schedule = "*/15 * * * *"
prompt = "Review recent changes for issues."
env = { API_KEY = "${MY_API_KEY}" }
readonly = false
timeout = "1h"
skills = ["frontend-design", "security-audit"]
```

### Key Configuration Fields

| Section | Field | Required | Description |
|---------|-------|----------|-------------|
| `settings` | `image_name` | No | Docker image for agents (default: "switchboard-agent") |
| `settings` | `timezone` | No | IANA timezone (default: "system") |
| `settings` | `overlap_mode` | No | "skip" or "queue" (default: "skip") |
| `agent` | `name` | Yes | Unique agent identifier |
| `agent` | `schedule` | Yes | Cron expression |
| `agent` | `prompt` | Yes* | Inline prompt text |
| `agent` | `prompt_file` | Yes* | Path to prompt file |
| `agent` | `timeout` | No | Max execution time (default: "30m") |
| `agent` | `skills` | No | List of skills to install |
| `agent` | `env` | No | Environment variables |

*Either `prompt` or `prompt_file` is required, not both.

> 📝 **Full Configuration Reference**: See [docs/configuration.md](docs/configuration.md)

---

## CLI Reference

| Command | Description |
|---------|-------------|
| `switchboard up` | Start the scheduler (runs agents on schedule) |
| `switchboard run <name>` | Run a specific agent immediately |
| `switchboard build` | Build the Docker image |
| `switchboard validate` | Validate configuration file |
| `switchboard logs` | View agent logs |
| `switchboard metrics` | View execution metrics |
| `switchboard list` | List all configured agents |
| `switchboard skills` | Manage skills |
| `switchboard --help` | Show all commands |

### Common Usage

```bash
# Validate config before running
switchboard validate

# Run a specific agent now
switchboard run my-agent

# View logs for an agent
switchboard logs my-agent

# Check execution metrics
switchboard metrics
```

---

## Features

### Skills

**Skills** are reusable agent configurations stored in the `skills/` directory. They provide:

- Versioned, reproducible agent capabilities
- Shared prompts and instructions across agents
- Easy updates without changing main configuration

Example: A `rust-best-practices` skill can be used by multiple agents to enforce coding standards.

> 📝 **Skills Documentation**: See [docs/skills.md](docs/skills.md)

### Discord Concierge

The **Discord Concierge** is an AI-powered bot that lives in your Discord server. It provides:

- Conversational interface to your agent system
- Natural language queries about project status
- Bug and task filing capabilities
- Relay of agent outputs to your team

```toml
[discord]
enabled = true
token_env = "${DISCORD_TOKEN}"
channel_id = "YOUR_CHANNEL_ID"

[discord.llm]
provider = "openrouter"
api_key_env = "${OPENROUTER_API_KEY}"
model = "anthropic/claude-sonnet-4"
```

> 📝 **Discord Documentation**: See [docs/discord.md](docs/discord.md)

---

## Documentation

| Guide | Description |
|-------|-------------|
| [docs/installation.md](docs/installation.md) | Detailed installation and prerequisites |
| [docs/configuration.md](docs/configuration.md) | Complete TOML configuration reference |
| [docs/skills.md](docs/skills.md) | Creating and using skills |
| [docs/discord.md](docs/discord.md) | Discord bot setup and usage |
| [docs/troubleshooting.md](docs/troubleshooting.md) | Common issues and solutions |

---

## License

MIT
