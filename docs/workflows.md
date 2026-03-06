# Workflows Feature Documentation

Workflows are pre-built agent configurations that can be installed from a registry and applied to your Switchboard project. They provide ready-made multi-agent setups for common development workflows.

## Overview

The Workflows feature allows you to:

- Browse available workflows from a central registry
- Install complete agent configurations with one command
- Update workflows to get the latest improvements
- Generate `switchboard.toml` configurations from workflow manifests

## Use Cases

### Multi-Agent Development Teams
Deploy coordinated agent teams with defined roles (architect, developer, code reviewer, scrum master) that work together on your project.

### Standardized Processes
Ensure consistent development practices across teams by using predefined workflow templates.

### Quick Project Setup
Get started quickly with a pre-configured agent setup instead of configuring agents manually.

## CLI Commands

Switchboard provides 7 subcommands for workflow management:

### List Available Workflows

```bash
switchboard workflows list
```

Search for specific workflows:
```bash
switchboard workflows list --search docker
switchboard workflows list --search "code review"
```

Limit results:
```bash
switchboard workflows list --limit 20
```

### Install a Workflow

```bash
switchboard workflows install workflow-name
```

Skip confirmation:
```bash
switchboard workflows install --yes workflow-name
```

Workflows are installed to the project-level `.switchboard/workflows/` directory.

### List Installed Workflows

```bash
switchboard workflows installed
```

Shows workflow name, description, version, and source for each installed workflow.

### Update Workflows

Update all installed workflows:
```bash
switchboard workflows update
```

Update a specific workflow:
```bash
switchboard workflows update workflow-name
```

### Remove a Workflow

```bash
switchboard workflows remove workflow-name
```

Skip confirmation:
```bash
switchboard workflows remove --yes workflow-name
```

### Validate a Workflow

```bash
switchboard workflows validate workflow-name
```

Validates the workflow's `manifest.toml` file exists and is properly formatted. Checks:
- Referenced prompt files exist
- Cron schedules are valid
- `overlap_mode` values are valid

### Apply a Workflow

Generate `switchboard.toml` configuration from a workflow's manifest:
```bash
switchboard workflows apply workflow-name
```

Additional options:
```bash
# Append to existing config instead of creating new
switchboard workflows apply workflow-name --append

# Preview what would be generated
switchboard workflows apply workflow-name --dry-run

# Custom output file
switchboard workflows apply workflow-name --output my-config.toml

# Agent name prefix to avoid conflicts
switchboard workflows apply workflow-name --prefix myteam
```

## Configuration

### Workflow Directory Structure

Installed workflows are stored in `.switchboard/workflows/<workflow-name>/`:

```
.switchboard/workflows/
└── bmad/
    ├── manifest.toml       # Workflow definition
    └── prompts/
        ├── ARCHITECT.md
        ├── CODE_REVIEWER.md
        ├── DEV_PARALLEL.md
        └── SCRUM_MASTER.md
```

### Manifest Format

The `manifest.toml` defines the workflow:

```toml
name = "workflow-name"
description = "Description of what this workflow does"
version = "1.0.0"

[defaults]
schedule = "0 9 * * *"    # Default cron schedule
timeout = "30m"           # Default timeout
readonly = false
overlap_mode = "skip"    # skip | queue | parallel
skills = ["owner/repo@skill-name"]  # Skills applied to all agents

[[prompts]]
name = "PROMPT.md"
description = "Prompt description"
role = "architect"

[[agents]]
name = "agent-name"
prompt_file = "PROMPT.md"
schedule = "*/30 * * * *"
timeout = "30m"
env = { KEY = "value" }
skills = ["owner/repo@agent-skill"]  # Agent-specific skills
```

## Skills Integration

Workflows can declare required skills that are automatically installed when the workflow is installed. This ensures all agents have the capabilities they need to execute their tasks.

### How It Works

1. When you install a workflow with `switchboard workflows install <name>`, any skills defined in the manifest are automatically installed
2. Skills are installed to your project's `./skills/` directory
3. When updating workflows with `switchboard workflows update`, skills are also updated to their latest versions

### Declaring Skills

Skills can be specified at two levels:

**Default Skills** (applies to all agents in the workflow):
```toml
[defaults]
skills = ["vercel-labs/agent-skills@frontend-design"]
```

**Per-Agent Skills** (overrides defaults for specific agents):
```toml
[[agents]]
name = "code-reviewer"
prompt_file = "CODE_REVIEW.md"
skills = ["vercel-labs/agent-skills@security-audit"]
```

### Validation

The `switchboard workflows validate` command checks if all required skills are installed and warns you about any missing skills.

### Troubleshooting

**Missing skills after installation:**
```bash
# Manually install the missing skill
switchboard skills install owner/repo@skill-name

# Or reinstall the workflow
switchboard workflows remove workflow-name
switchboard workflows install workflow-name
```

## Registry

Workflows are stored in the [switchboard-workflows](https://github.com/kkingsbe/switchboard-workflows) repository. The registry is fetched from GitHub when you run `list` or `install` commands.

## Examples

### Install and Apply BMAD Workflow

```bash
# List available workflows
switchboard workflows list

# Install the BMAD workflow
switchboard workflows install bmad

# View installed workflows
switchboard workflows installed

# Validate before applying
switchboard workflows validate bmad

# Apply to generate switchboard.toml
switchboard workflows apply bmad

# Start the agents
switchboard up
```

### Custom Workflow Setup

```bash
# Apply with custom prefix to avoid conflicts
switchboard workflows apply bmad --prefix myteam

# Append to existing configuration
switchboard workflows apply another-workflow --append
```

## Troubleshooting

### "Workflow not found"
- Check the workflow name is correct: `switchboard workflows list`
- Ensure you have network access to GitHub

### "Prompt file not found" during validate
- The workflow may be corrupted. Try reinstalling: `switchboard workflows remove <name> && switchboard workflows install <name>`

### "Agent name conflict" when applying
- Use the `--prefix` flag to avoid conflicts: `switchboard workflows apply <name> --prefix myprefix`

### Permission denied errors
- Ensure the `.switchboard/workflows/` directory is writable
- Check file permissions in your project directory

## See Also

- [Configuration](configuration.md) - Switchboard configuration file format
- [Skills](skills.md) - Extensible agent capabilities
- [Troubleshooting](troubleshooting.md) - General troubleshooting guide
