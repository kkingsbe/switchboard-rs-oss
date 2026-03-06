# Skills Feature Documentation

Skills are reusable agent configuration packages that extend Switchboard's capabilities. They provide procedural knowledge, tools, and prompts that agents can use to perform specialized tasks.

## What are Skills?

Skills are declarative knowledge packages that define how an AI agent should approach specific tasks. Each skill contains:

- **Metadata** - Name, version, description, and author information
- **Procedural Knowledge** - Instructions and best practices for the agent
- **Tool Definitions** - Custom tools the agent can use
- **Prompts** - Template prompts tailored to the skill's domain

### Use Cases

Skills are commonly used for:

- **Code Review** - Specialized review guidelines for different languages and frameworks
- **Testing** - Test generation patterns and testing best practices
- **Documentation** - Documentation generation and maintenance procedures
- **Security Auditing** - Security vulnerability detection patterns
- **Refactoring** - Code modernization and improvement patterns
- **Frontend Design** - UI/UX guidelines and component standards

## Configuration

### Defining Skills in switchboard.toml

Skills are declared in the `skills` field of each agent configuration:

```toml
[[agent]]
name = "ui-reviewer"
schedule = "*/30 * * * *"
prompt = "Review UI components for design consistency."

# Skills for this agent
skills = [
    "vercel-labs/agent-skills@frontend-design",
]
```

### Skill Source Format

Skills can be specified in several formats:

| Format | Description | Example |
|--------|-------------|---------|
| `owner/repo@skill-name` | Specific skill from a repository | `vercel-labs/agent-skills@frontend-design` |
| `owner/repo` | All skills from a repository | `anthropics/skills` |
| `https://github.com/owner/repo` | Full GitHub URL | `https://github.com/owner/repo` |
| `npm-package-name` | npm package | `@kilocode/react-atomic-refactoring` |

### Example Configuration

```toml
# Example: Agent with multiple skills
[[agent]]
name = "comprehensive-reviewer"
schedule = "0 */2 * * *"
prompt = "Perform a comprehensive code review."

skills = [
    "anthropics/skills@security-audit",
    "vercel-labs/agent-skills@code-quality",
]

timeout = "1h"
readonly = true
```

## CLI Commands

Switchboard provides the `switchboard skills` subcommand for managing skills.

### List Available Skills

Search and browse skills from the skills.sh registry:

```bash
# List skills matching a search query
switchboard skills list --search docker

# With custom result limit
switchboard skills list --search "code review" --limit 20

# Default: shows popular AI-related skills
switchboard skills list
```

### Install a Skill

Install a skill to your project's `./skills/` directory:

```bash
# Install a specific skill
switchboard skills install owner/repo@skill-name

# Install all skills from a repository
switchboard skills install owner/repo

# Install globally (available to all projects)
switchboard skills install --global owner/repo@skill-name

# Skip confirmation prompt
switchboard skills install --yes owner/repo@skill-name
```

### List Installed Skills

View currently installed skills:

```bash
# List all installed skills
switchboard skills installed

# Show only global skills
switchboard skills installed --global
```

### Update a Skill

Update installed skills to their latest versions:

```bash
# Update all installed skills
switchboard skills update

# Update a specific skill
switchboard skills update frontend-design
```

### Remove a Skill

Remove an installed skill:

```bash
# Remove a skill (with confirmation prompt)
switchboard skills remove frontend-design

# Remove without confirmation
switchboard skills remove --yes frontend-design

# Remove from global skills directory
switchboard skills remove --global skill-name
```

## Skill Manifest (SKILL.md)

Skills are defined using a `SKILL.md` file with YAML frontmatter. This file must be at the root of the skill package.

### Required Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Unique identifier for the skill (lowercase, hyphens) |

### Optional Fields

| Field | Type | Description |
|-------|------|-------------|
| `description` | string | Human-readable description of the skill |
| `version` | string | Semantic version (e.g., "1.0.0") |
| `authors` | array | List of author names/emails |
| `dependencies` | array | Other skills this skill depends on |
| `compatible_agents` | array | Agent types that can use this skill |
| `source` | string | Source URL (GitHub, npm, etc.) |

### Agent Type Values

Common agent types that can use skills:

- `architect` - Planning and architecture design
- `code` - Code implementation and modification
- `ask` - Question answering and analysis
- `debug` - Troubleshooting and debugging
- `orchestrator` - Task coordination
- `task-discovery` - Scope discovery
- `html-reporter` - Report generation
- `codebase-scan` - Code analysis
- `interviewer` - Interview assistance

### Example SKILL.md

```markdown
---
name: frontend-design
description: Frontend development best practices and component guidelines
version: 1.0.0
authors: ["Vercel Labs <team@vercel.com>"]
dependencies: []
compatible_agents: ["code", "architect", "ask"]
source: https://github.com/vercel-labs/agent-skills
---

# Frontend Design Skill

This skill provides guidelines and patterns for building production-grade
frontend interfaces using modern frameworks.

## Component Structure

Follow atomic design principles:
- Atoms: Basic UI elements (buttons, inputs)
- Molecules: Simple component groups
- Organisms: Complex UI sections
- Templates: Page layouts
- Pages: Full applications
```

## Skill Lockfile

When skills are installed, Switchboard generates a `skills.lock.json` file that tracks installed skills:

```json
{
  "version": 1,
  "skills": {
    "rust-best-practices": {
      "source": "apollographql/skills",
      "sourceType": "github",
      "computedHash": "3d80db37cef0198fe21f6f10363bbe0a7cb3f36afab49d0f3270fad4abc2a6d9"
    }
  }
}
```

This lockfile ensures reproducible builds by pinning skill versions.

## Common Issues

### npx Not Found

**Error:**
```
Error: npx is required for this command. Install Node.js from https://nodejs.org
```

**Cause:** Node.js is not installed on the host machine.

**Solution:** Install Node.js from [https://nodejs.org](https://nodejs.org) (LTS version recommended).

### Installation Failures

**Error:**
```
Error: Destination already exists: ./skills/frontend-design
```

**Cause:** The skill is already installed.

**Solution:** Use the `--yes` flag to overwrite, or remove the existing skill first:
```bash
switchboard skills remove frontend-design
switchboard skills install owner/repo@frontend-design
```

### Skill Not Found

**Error:**
```
Error: Skill 'owner/repo@nonexistent-skill' not found
```

**Cause:** The specified skill doesn't exist in the repository.

**Solution:** Verify the skill name is correct by browsing available skills:
```bash
switchboard skills list --search <keyword>
```

### Network Issues

**Error:**
```
Failed to search skills: Network unavailable
```

**Cause:** No internet connection or firewall blocking network requests.

**Solution:**
- Check your internet connection
- Verify firewall/proxy settings allow connections to npm registries
- Try again later if there's a temporary outage

### Malformed SKILL.md

**Error:**
```
Malformed SKILL.md for 'skill-name': invalid yaml frontmatter
```

**Cause:** The skill's SKILL.md file has invalid YAML syntax.

**Solution:** This is typically a bug in the skill package. Report the issue to the skill maintainer or try a different version.

### Permission Denied

**Error:**
```
Failed to execute npx skills add: Permission denied
```

**Cause:** Insufficient permissions to write to the skills directory.

**Solution:**
- Check directory permissions
- Ensure the `./skills/` directory is writable
- Try running with appropriate permissions

## Requirements

### For CLI Commands (Host Machine)

- Node.js and npx installed
- Internet connection for accessing the skills registry

### For Agent Execution (Container)

- Skills are installed inside the container at runtime
- The container base image includes Node.js (`node:22-slim`)
- No host-side installation required for agent execution

## Best Practices

1. **Pin specific versions** - Use `@skill-name` format for reproducible behavior
2. **Keep skills updated** - Run `switchboard skills update` periodically
3. **Review skill sources** - Only install skills from trusted sources
4. **Use appropriate timeouts** - Complex skills may need longer timeouts
5. **Monitor skill installations** - Check logs if agents fail to start

## Workflow Skills Integration

When you install a workflow that requires skills, they are automatically installed to your project's `./skills/` directory.

### How It Works

1. When you run `switchboard workflows install <workflow-name>`, Switchboard:
   - Downloads the workflow from the registry
   - Parses the workflow's `manifest.toml`
   - Extracts required skills from `[defaults].skills` and `[[agents]].skills`
   - Installs each skill that isn't already present
   - Updates the skills lockfile

2. When you run `switchboard workflows update`, Switchboard:
   - Updates the workflow files
   - Updates all required skills to their latest versions

### Required Skills

Skills can be specified at two levels in `manifest.toml`:

#### Default Skills (applies to all agents)

```toml
[defaults]
skills = ["owner/repo@skill-name"]
```

#### Per-Agent Skills

```toml
[[agents]]
name = "architect"
prompt_file = "ARCHITECT.md"
skills = ["owner/repo@agent-skill"]
```

## Validation

Running `switchboard workflows validate <workflow-name>` will check if all required skills are installed and warn you if any are missing.

## Troubleshooting

### Skill Installation Fails

If a skill fails to install during workflow installation, the workflow installation will be aborted. To resolve:

1. Install Node.js from https://nodejs.org
2. Try installing the workflow again: `switchboard workflows install <workflow-name>`

### Missing Skills Warning

If you see warnings about missing skills:

```bash
# Install the missing skill manually
switchboard skills install owner/repo@skill-name

# Or let the workflow install handle it
switchboard workflows install workflow-name
```
