# Switchboard

<p align="center">
  <b>Cron for AI coding agents</b><br>
  Schedule and run AI coding agents with Docker isolation, cron-based scheduling, and Discord integration.
</p>

<p align="center">

[![Build](https://github.com/kkingsbe/switchboard-rs-oss/actions/workflows/build.yml/badge.svg)](https://github.com/kkingsbe/switchboard-rs-oss/actions/workflows/build.yml)

[![Discord](https://img.shields.io/discord/123456789?label=Discord)](https://discord.gg/x6S59ASxGa)

</p>

---

## tl;dr

```bash
# 1. Install
cargo install --path .

# 2. Build Docker image
switchboard build

# 3. Create config (switchboard.toml)
echo '[settings]
image_name = "switchboard-agent"

[[agent]]
name = "daily-coder"
schedule = "0 9 * * *"
prompt = "Review PRs and summarize changes."' > switchboard.toml

# 4. Run
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

## How It Works

Switchboard is built specifically for **Kilo Code**, an AI coding agent platform that provides API access to powerful coding models. Kilo Code acts as the execution engine, handling the AI model interactions while Switchboard handles scheduling, orchestration, and infrastructure.

### What is Kilo Code?

Kilo Code is an AI coding agent platform that offers API-based access to large language models optimized for coding tasks. It provides a unified interface to various AI models, handling authentication, rate limiting, and API abstraction so you can focus on writing prompts rather than managing infrastructure.

### Token-Based Access

Kilo Code uses a token-based authentication system. Users must obtain a Kilo Code token to use the platform. This token is configured in `.kilocode/cli/config.json` and authenticates your requests to the Kilo Code API. The token controls access to the AI models and tracks usage for the account it's associated with.

### Available Models

Switchboard currently supports two models that have been tested and optimized for coding tasks:

- **`z-ai/glm-4.7`** (default) — A high-performance model well-suited for code generation and analysis
- **`minimax/minimax-m2.5`** — An alternative model with strong reasoning capabilities

These specific models are supported because they have been evaluated for coding tasks and integrated with Kilo Code's infrastructure. Additional models may be added as they become available and tested.

---

## Quick Start

### Step 1: Ensure Docker is running

> ⚠️ **Prerequisite**: Docker must be running

```bash
docker --version
```

### Step 2: Install Switchboard

```bash
cargo install --path .
```

### Step 3: Build Docker image

```bash
switchboard build
```

### Step 4: Create a configuration file

Create `switchboard.toml` in your project:

```toml
[settings]
image_name = "kilosynth/prompter:latest"

[[agent]]
name = "daily-coder"
schedule = "0 9 * * *"
prompt = "Review any open PRs and summarize the changes."
```

### Step 5: Validate your configuration

> ✅ **Tip**: Use `switchboard validate` before running

```bash
switchboard validate
```

### Step 6: Start the scheduler

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
| **Kilo Code Token** | - | API token for AI agent execution |

> ⚠️ **Note**: You must obtain a Kilo Code token to run AI agents. After cloning, copy `.kilocode/cli/config.example.json` to `.kilocode/cli/config.json` and fill in your token and model choice. See [Kilo Code Configuration](#kilo-code-configuration) for details.

### Kilo Code Configuration

Switchboard requires a Kilo Code token to execute AI agents. After cloning the repository:

1. Copy the example config file:
   ```bash
   cp .kilocode/cli/config.example.json .kilocode/cli/config.json
   ```

2. Edit `.kilocode/cli/config.json` and fill in your token:
   ```json
   {
     "id": "default",
     "provider": "kilocode",
     "kilocodeToken": "YOUR_TOKEN_HERE",
     "kilocodeModel": "z-ai/glm-4.7",
     "kilocodePosthogApiKey": ""
   }
   ```

3. Available models include:
   - `z-ai/glm-4.7` (default)
   - `minimax/minimax-m2.5`

### Install from Source

```bash
git clone https://github.com/kkingsbe/switchboard-rs-oss.git
cd switchboard

# Configure Kilo Code token
cp .kilocode/cli/config.example.json .kilocode/cli/config.json
# Edit config.json and add your kilocodeToken and optionally change kilocodeModel

cargo install --path .
```

### Build Docker Image

```bash
switchboard build
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

Schedules use **5-field cron expressions** (Unix cron format):

```
minute hour day month weekday
```

Examples:
- `"0 * * * *"` — Every hour on the hour
- `"*/15 * * * *"` — Every 15 minutes
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
