# CLI Command Reference

Complete reference for Switchboard CLI commands.

## Synopsis

```bash
switchboard [GLOBAL OPTIONS] <command> [command options] [arguments...]
```

## Global Flags

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-h, --help` | Show help information |
| `--version` | Show version information |

## Commands

### run

Immediately execute a single agent without scheduling.

```bash
switchboard run [agent-name]
```

**Options:**
- `agent-name` - Name of the agent to run (required)

**Examples:**
```bash
# Run a specific agent
switchboard run my-agent

# Run with verbose output
switchboard run my-agent --verbose
```

---

### build

Build or rebuild the agent Docker image.

```bash
switchboard build [options]
```

**Options:**
- `--no-cache` - Build without using Docker cache
- `--pull` - Always pull base images

**Examples:**
```bash
# Standard build
switchboard build

# Fresh build without cache
switchboard build --no-cache
```

---

### list

Print all configured agents, their schedules, and prompts.

```bash
switchboard list [options]
```

**Options:**
- `--format json` - Output in JSON format
- `--format table` - Output in table format (default)

**Examples:**
```bash
# List all agents
switchboard list

# JSON output for scripting
switchboard list --format json
```

---

### logs

View logs from agent runs.

```bash
switchboard logs [options] [agent-name]
```

**Options:**
- `-f, --follow` - Stream logs in real-time
- `--tail N` - Show last N lines
- `--since duration` - Show logs since duration (e.g., 1h, 30m)
- `--until duration` - Show logs until duration

**Examples:**
```bash
# View last 100 lines of logs
switchboard logs --tail 100

# Follow logs in real-time
switchboard logs -f

# Logs from the last hour
switchboard logs --since 1h

# Specific agent logs
switchboard logs my-agent --tail 50
```

---

### metrics

Display agent execution metrics.

```bash
switchboard metrics [options] [agent-name]
```

**Options:**
- `--period duration` - Metrics time period (e.g., 24h, 7d)
- `--format json` - Output in JSON format

**Examples:**
```bash
# All agent metrics
switchboard metrics

# Metrics for specific agent
switchboard metrics my-agent

# Last 7 days
switchboard metrics --period 7d
```

---

### down

Stop the scheduler and any running agent containers.

```bash
switchboard down [options]
```

**Options:**
- `--rmi` - Also remove Docker images
- `-v, --volumes` - Remove named volumes

**Examples:**
```bash
# Stop scheduler
switchboard down

# Stop and remove images
switchboard down --rmi
```

---

### status

Check scheduler health and status.

```bash
switchboard status
```

**Output includes:**
- Scheduler running state
- Number of active agents
- Last run times
- Docker container status

---

### validate

Parse and validate the configuration file.

```bash
switchboard validate [options] [config-file]
```

**Options:**
- `--strict` - Enable strict validation

**Examples:**
```bash
# Validate default config
switchboard validate

# Validate specific file
switchboard validate my-config.toml

# Strict validation
switchboard validate --strict
```

---

### skills

Manage Kilo skills. See [Skills Documentation](skills.md) for details.

```bash
switchboard skills [command]
```

**Subcommands:**
- `list` - List available skills
- `install` - Install a skill
- `installed` - List installed skills
- `update` - Update installed skills
- `remove` - Remove a skill

---

### workflows

Manage Switchboard workflows. See [Workflows Documentation](workflows.md) for details.

```bash
switchboard workflows [command]
```

**Subcommands:**
- `list` - List available workflows
- `install` - Install a workflow
- `installed` - List installed workflows
- `update` - Update workflows
- `remove` - Remove a workflow
- `validate` - Validate a workflow
- `apply` - Apply workflow to generate config

---

### gateway

Start the Discord Gateway service (requires `gateway` feature).

```bash
switchboard gateway [options]
```

**Options:**
- `--host address` - Server host (default: "0.0.0.0")
- `--port port` - Server port (default: 8080)

**Examples:**
```bash
# Default gateway
switchboard gateway

# Custom port
switchboard gateway --port 3000
```

---

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Docker error |
| 4 | Network error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SWITCHBOARD_CONFIG` | Path to config file |
| `SWITCHBOARD_DIR` | Path to .switchboard directory |
| `DOCKER_HOST` | Docker daemon socket |

## See Also

- [Configuration](configuration.md) - Configuration file format
- [Installation](installation.md) - Installation instructions
- [Troubleshooting](troubleshooting.md) - Common issues
