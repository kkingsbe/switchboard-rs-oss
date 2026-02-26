# Troubleshooting

This guide helps you diagnose and fix common runtime issues when using switchboard.

## About This Guide

This guide focuses on **runtime issues** that occur after switchboard is installed and configured. For installation-related issues (Rust/Cargo setup, Docker installation, permission problems, etc.), please see the [Installation Troubleshooting Guide](INSTALLATION_TROUBLESHOOTING.md).

---

## Quick Checklist

Before diving into specific issues, verify the following:

- [ ] Docker daemon is running (`docker ps` succeeds)
- [ ] Your user has Docker permissions (no sudo required for `docker ps`)
- [ ] `.gilocode` directory exists with `config.json` and `api_keys.json`
- [ ] `switchboard.toml` configuration file is present and valid
- [ ] Workspace path specified in config exists and is accessible
- [ ] Sufficient disk space available for containers and logs

---

## Runtime Issues

### Docker Daemon Errors

#### Docker daemon not running

**Symptom:**
```
Error: Docker daemon is not running
Error: Failed to connect to Docker daemon
Error: Cannot connect to the Docker daemon at unix:///var/run/docker.sock
```

**Cause:** The Docker daemon is not started or not responding.

**Resolution:**

**Linux:**
```bash
sudo systemctl start docker
sudo systemctl enable docker  # Optional: Start on boot
```

**macOS:**
- Open Docker Desktop from Applications
- Wait for the Docker icon in the menu bar to indicate it's running (steady, not animating)
- Verify with `docker ps`

**Windows (WSL):**
- Open Docker Desktop
- Go to Settings → Resources → WSL Integration
- Enable WSL 2 integration

---

#### Docker permission denied

**Symptom:**
```
permission denied while trying to connect to the Docker daemon socket
```

**Cause:** Your user is not in the docker group.

**Resolution:**

Add your user to the docker group:

```bash
sudo usermod -aG docker $USER
```

Log out and log back in for changes to take effect, or run:

```bash
newgrp docker
```

**Verify permissions:**
```bash
docker ps  # Should work without sudo
```

---

#### Docker connection refused

**Symptom:**
```
Error: Failed to connect to Docker daemon
Error: Connection refused
```

**Cause:** Docker daemon is running but not accepting connections, or there's a network issue.

**Resolution:**

1. Check if Docker is responding:
   ```bash
   docker info
   ```

2. If `docker info` fails, restart Docker:

   **Linux:**
   ```bash
   sudo systemctl restart docker
   ```

   **macOS/Windows:**
   - Restart Docker Desktop

3. Check Docker daemon logs (Linux):
   ```bash
   sudo journalctl -u docker.service -n 50
   ```

4. Verify no firewall is blocking Docker communication

---

### Agent Container Failures

#### Container creation failed

**Symptom:**
```
Failed to create container 'agent-name': {error}
```

**Cause:** Invalid container configuration, resource limits, or image issues.

**Resolution:**

1. Verify the Docker image exists:
   ```bash
   docker images | grep switchboard-agent
   ```

2. If image doesn't exist, build it:
   ```bash
   switchboard build
   ```

3. Check for resource constraints:
   ```bash
   docker system df
   ```

4. Validate configuration with:
   ```bash
   switchboard validate
   ```

5. Check Docker daemon logs for detailed error information

---

#### Container start failed

**Symptom:**
```
Failed to start container '{container_id}': {error}
```

**Cause:** Missing dependencies, invalid environment variables, or resource conflicts.

**Resolution:**

1. Inspect the container for configuration issues:
   ```bash
   docker inspect {container_id}
   ```

2. Check container logs for startup errors:
   ```bash
   docker logs {container_id}
   ```

3. Verify workspace path exists and is accessible:
   ```bash
   ls -la {workspace_path}
   ```

4. Verify `.kilocode` directory exists and contains required files:
   ```bash
   ls -la ~/.kilocode
   cat ~/.kilocode/config.json
   ```

5. Check for port conflicts if using custom network configurations

---

#### Container stop failed

**Symptom:**
```
Failed to stop container {id}: {error}
```

**Cause:** Container may be unresponsive, in a bad state, or Docker daemon issues.

**Resolution:**

1. Check container status:
   ```bash
   docker ps -a | grep {container_id}
   ```

2. Try killing the container instead of stopping:
   ```bash
   docker kill {container_id}
   ```

3. If still failing, remove the container forcibly:
   ```bash
   docker rm -f {container_id}
   ```

4. Verify Docker daemon is healthy:
   ```bash
   docker info
   ```

---

### Scheduler Runtime Issues

#### Scheduler already running

**Symptom:**
```
Scheduler is already running (PID: {pid}). Use 'switchboard list' to see active agents or 'switchboard down' to stop it first
```

**Cause:** A scheduler process is already running for this project.

**Resolution:**

1. Check running agents:
   ```bash
   switchboard list
   ```

2. If you want to restart the scheduler, stop it first:
   ```bash
   switchboard down
   ```

3. Verify no stale PID file exists:
   ```bash
   ls -la .switchboard/pid
   ```

4. If scheduler crashed and PID file remains, remove it:
   ```bash
   rm .switchboard/pid
   ```

---

#### Scheduler start failed

**Symptom:**
```
Failed to start scheduler: {error}
```

**Cause:** Configuration errors, missing dependencies, or system resource issues.

**Resolution:**

1. Validate configuration:
   ```bash
   switchboard validate
   ```

2. Check Docker availability:
   ```bash
   docker ps
   ```

3. Verify `.gilocode` directory setup:
   ```bash
   ls -la ~/.kilocode/config.json
   ls -la ~/.kilocode/api_keys.json
   ```

4. Check for port conflicts or resource limitations:
   ```bash
   free -h  # Memory
   df -h    # Disk space
   ```

5. Review scheduler logs for detailed error information:
   ```bash
   switchboard logs --follow
   ```

---

#### Agent registration failed

**Symptom:**
```
Failed to register agent '{name}': {error}
```

**Cause:** Invalid cron expression, timezone issues, or configuration errors.

**Resolution:**

1. Validate configuration:
   ```bash
   switchboard validate
   ```

2. Check cron expression format:
   - Must be 5 fields: minute hour day month weekday
   - Valid examples: `0 */6 * * *`, `*/30 * * * *`, `0 2 * * 1`

3. Verify timezone is valid IANA format:
   - Valid: `America/New_York`, `Europe/London`, `Asia/Tokyo`
   - Invalid: `EST`, `UTC-5`, `PST8PDT`

4. Check for duplicate agent names in configuration

---

#### Agent execution failed

**Symptom:**
```
Failed to run agent: {error}
```

**Cause:** Docker connection issues, missing image, or container runtime errors.

**Resolution:**

1. Verify Docker is running:
   ```bash
   docker ps
   ```

2. Check if agent image exists:
   ```bash
   docker images | grep switchboard-agent
   ```

3. If image doesn't exist, build it:
   ```bash
   switchboard build
   ```

4. View agent logs for detailed error:
   ```bash
   switchboard logs {agent_name}
   ```

5. Inspect failed container:
   ```bash
   docker ps -a | grep switchboard-agent
   docker logs {container_id}
   ```

---

## Configuration Issues

### TOML Syntax Errors

**Symptom:**
```
Error parsing switchboard.toml: Failed to read file: {error}
Error parsing switchboard.toml:line {line}, col {col}: {message}
```

**Cause:** Malformed TOML syntax in configuration file.

**Resolution:**

Common syntax mistakes and fixes:

1. **Missing quotes around strings:**
   ```toml
   # ❌ Invalid
   name = my-agent
   schedule = 0 */6 * * *

   # ✅ Valid
   name = "my-agent"
   schedule = "0 */6 * * *"
   ```

2. **Invalid list syntax:**
   ```toml
   # ❌ Invalid
   env = KEY=value

   # ✅ Valid (use table format for single items)
   [env]
   KEY = "value"

   # ✅ Valid (for multiple env vars, use array of objects - not supported yet)
   ```

3. **Unterminated strings or tables:**
   ```toml
   # ❌ Invalid (missing closing bracket)
   [[agent]
   name = "my-agent"

   # ✅ Valid
   [[agent]]
   name = "my-agent"
   ```

4. **Invalid boolean values:**
   ```toml
   # ❌ Invalid
   readonly = "true"

   # ✅ Valid
   readonly = true
   ```

5. **Validate your config:**
   ```bash
   switchboard validate
   ```

---

### Cron Expression Errors

#### Invalid field count

**Symptom:**
```
Validation error in field 'schedule': Invalid cron expression '{schedule}': expected 5 fields (minute hour day month weekday), got {count}. Example: '0 */6 * * *' (runs every 6 hours)
```

**Cause:** Cron expression has wrong number of fields.

**Resolution:**

Cron expressions must have exactly 5 fields: `minute hour day month weekday`

```toml
# ❌ Invalid (6 fields)
schedule = "0 * * * * *"

# ❌ Invalid (4 fields)
schedule = "0 * * *"

# ✅ Valid (5 fields)
schedule = "0 */6 * * *"      # Every 6 hours
schedule = "*/30 * * * *"      # Every 30 minutes
schedule = "0 2 * * *"         # Daily at 2 AM
schedule = "0 2 * * 1"         # Every Monday at 2 AM
schedule = "@daily"            # Once per day (also supported)
```

---

#### Invalid field range

**Symptom:**
```
Validation error in field 'schedule': Invalid cron expression '{schedule}': {error}
```

**Cause:** Field value outside valid range.

**Resolution:**

Valid ranges for each field:
- Minute: 0-59
- Hour: 0-23
- Day: 1-31
- Month: 1-12
- Weekday: 0-6 (0 = Sunday, 6 = Saturday)

```toml
# ❌ Invalid (minute must be 0-59)
schedule = "0-60 * * * *"

# ❌ Invalid (hour must be 0-23)
schedule = "* * * * 24"

# ✅ Valid
schedule = "0 */6 * * *"      # Every 6 hours
schedule = "*/30 * * * *"      # Every 30 minutes
schedule = "0 0 * * 0"         # Every Sunday at midnight
```

---

#### Parse failure

**Symptom:**
```
Validation error in field 'schedule': Invalid cron expression '{schedule}': {error}
```

**Cause:** Cron expression contains invalid characters or syntax.

**Resolution:**

Common cron syntax errors:

1. **Invalid special characters:**
   ```toml
   # ❌ Invalid
   schedule = "0 @hourly * * *"

   # ✅ Valid
   schedule = "@hourly"
   schedule = "0 * * * *"
   ```

2. **Invalid range notation:**
   ```toml
   # ❌ Invalid
   schedule = "60-90 * * * *"

   # ✅ Valid
   schedule = "0-30 * * * *"
   schedule = "*/15 * * * *"
   ```

3. **Mixed formats:**
   ```toml
   # ❌ Invalid
   schedule = "0-10/5 * * * *"

   # ✅ Valid
   schedule = "0,5,10 * * * *"
   schedule = "0-10 * * * *"
   ```

---

### Prompt File Errors

#### Prompt file not found

**Symptom:**
```
Prompt file '{prompt_file}' not found for agent '{agent_name}'
Error: ✗ Prompt file not found: {file}
```

**Cause:** Referenced prompt file doesn't exist at specified path.

**Resolution:**

1. Verify the file exists:
   ```bash
   ls -la prompts/my-prompt.md
   ```

2. Check if path is relative to config file or absolute:
   ```bash
   # If using relative path (relative to switchboard.toml location)
   [[agent]]
   name = "my-agent"
   prompt_file = "prompts/my-prompt.md"

   # If using absolute path
   [[agent]]
   name = "my-agent"
   prompt_file = "/home/user/my-project/prompts/my-prompt.md"
   ```

3. Create the prompt file if it doesn't exist:
   ```bash
   mkdir -p prompts
   echo "Your prompt content here" > prompts/my-prompt.md
   ```

---

#### Neither prompt nor prompt_file specified

**Symptom:**
```
Validation error in agent '{name}': Agent '{name}' must have either 'prompt' (inline text) or 'prompt_file' (path to file) specified
```

**Cause:** Agent configuration is missing both `prompt` and `prompt_file` fields.

**Resolution:**

You must specify exactly one of `prompt` or `prompt_file`:

```toml
# ✅ Option 1: Use inline prompt
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review the recent code changes and suggest improvements."

# ✅ Option 2: Use prompt file
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt_file = "prompts/review-code.md"

# ❌ Invalid: Neither specified
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
```

---

#### Both prompt and prompt_file specified

**Symptom:**
```
Validation error in agent '{name}': Agent '{name}' must have exactly one of 'prompt' or 'prompt_file' specified, not both
```

**Cause:** Agent configuration includes both `prompt` and `prompt_file` fields.

**Resolution:**

Choose exactly one approach:

```toml
# ✅ Option 1: Use inline prompt only
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review the recent code changes and suggest improvements."

# ✅ Option 2: Use prompt file only
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt_file = "prompts/review-code.md"

# ❌ Invalid: Both specified
[[agent]]
name = "code-reviewer"
schedule = "0 */6 * * *"
prompt = "Review the recent code changes and suggest improvements."
prompt_file = "prompts/review-code.md"
```

---

#### Invalid prompt file path

**Symptom:**
```
Validation error in agent '{name}' field 'prompt_file': Agent '{name}' has invalid prompt_file path: '{path}'. Check that the file path is absolute or relative to the config file
```

**Cause:** Path contains invalid characters or is malformed.

**Resolution:**

1. Use valid path characters:
   ```toml
   # ❌ Invalid (contains invalid characters)
   prompt_file = "prompts/review-code?.md"
   prompt_file = "prompts/review-code*.md"

   # ✅ Valid
   prompt_file = "prompts/review-code.md"
   prompt_file = "/home/user/prompts/review-code.md"
   ```

2. Escape spaces or use quotes:
   ```toml
   # ❌ Invalid (unescaped space)
   prompt_file = prompts/my prompt.md

   # ✅ Valid
   prompt_file = "prompts/my prompt.md"
   prompt_file = 'prompts/my prompt.md'
   ```

3. Verify file exists after fixing path:
   ```bash
   test -f "prompts/review-code.md" && echo "File exists" || echo "File not found"
   ```

---

### Timeout Configuration Errors

#### Invalid timeout format

**Symptom:**
```
Validation error in field 'timeout': Invalid timeout value: '{value}'. Valid formats: '30s' (30 seconds), '5m' (5 minutes), '1h' (1 hour). {error}
```

**Cause:** Timeout value doesn't match the expected format (Ns, Nm, Nh).

**Resolution:**

Timeout values must be in the format: `<number><unit>` where unit is `s` (seconds), `m` (minutes), or `h` (hours):

```toml
# ❌ Invalid
timeout = "30"
timeout = "30x"
timeout = "30min"
timeout = "30:00"

# ✅ Valid
timeout = "30s"   # 30 seconds
timeout = "5m"    # 5 minutes
timeout = "1h"    # 1 hour
timeout = "90s"   # 90 seconds
timeout = "2h30m" # Not supported - use one unit
```

---

#### Timeout value zero

**Symptom:**
```
Validation error in field 'timeout': Timeout value must be greater than 0. Use a positive value like '10s' or '5m'
```

**Cause:** Timeout value is 0, which would immediately terminate the container.

**Resolution:**

Use a positive timeout value:

```toml
# ❌ Invalid
timeout = "0s"
timeout = "0m"

# ✅ Valid
timeout = "10s"   # Minimum 10 seconds recommended
timeout = "30s"
timeout = "5m"
timeout = "1h"
```

---

#### Invalid timeout unit

**Symptom:**
```
Invalid timeout unit: '{unit}' (use s, m, or h)
```

**Cause:** Timeout unit is not one of the supported values.

**Resolution:**

Only the following units are supported:
- `s` - seconds
- `m` - minutes
- `h` - hours

```toml
# ❌ Invalid
timeout = "30sec"
timeout = "30min"
timeout = "30hr"

# ✅ Valid
timeout = "30s"
timeout = "30m"
timeout = "1h"
```

---

### Overlap Mode Errors

**Symptom:**
```
Validation error in field 'overlap_mode': Invalid overlap_mode '{mode}'. Must be one of: Skip (default, skip if already running), Queue (queue up to max_queue_size runs)
```

**Cause:** Overlap mode value is not a valid option.

**Resolution:**

Overlap mode must be either `Skip` or `Queue`:

```toml
# ❌ Invalid
overlap_mode = "wait"
overlap_mode = "concurrent"
overlap_mode = "skipqueue"

# ✅ Valid
overlap_mode = "Skip"    # Default: Skip new run if agent is already running
overlap_mode = "Queue"   # Queue up to max_queue_size runs to execute sequentially

# Can also set globally in [settings]
[settings]
overlap_mode_str = "Skip"  # or "Queue"
```

**Understanding overlap modes:**

- **Skip**: If an agent is already running when a scheduled time arrives, the new run is skipped and a warning is logged. This is the default behavior.

- **Queue**: If an agent is already running, the new run is added to a queue (up to `max_queue_size`). Runs execute sequentially after the current run completes.

---

### Timezone Errors

**Symptom:**
```
Validation error in field 'timezone': Invalid timezone '{tz}'. Use IANA timezone format (e.g., 'America/New_York', 'Europe/London', 'Asia/Tokyo'). See: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
```

**Cause:** Timezone value is not a valid IANA timezone identifier.

**Resolution:**

Use IANA timezone format:

```toml
# ❌ Invalid
timezone = "EST"
timezone = "UTC-5"
timezone = "PST8PDT"
timezone = "GMT"

# ✅ Valid (IANA timezone format)
timezone = "America/New_York"
timezone = "America/Los_Angeles"
timezone = "Europe/London"
timezone = "Asia/Tokyo"
timezone = "Australia/Sydney"
timezone = "UTC"
```

**Common IANA timezones:**

| Region | Timezone |
|--------|----------|
| US Eastern | America/New_York |
| US Pacific | America/Los_Angeles |
| UK | Europe/London |
| Central Europe | Europe/Paris |
| Japan | Asia/Tokyo |
| Australia (East) | Australia/Sydney |

For a complete list, see: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones

---

### Queue Size Errors

**Symptom:**
```
Validation error in field 'max_queue_size': Invalid value for 'max_queue_size': 0. Queue size must be a positive integer. Valid range: Minimum: 1, Maximum: 100
```

**Cause:** Queue size is invalid (zero, negative, or too large).

**Resolution:**

Queue size must be a positive integer between 1 and 100:

```toml
# ❌ Invalid
max_queue_size = 0
max_queue_size = -5
max_queue_size = 1000

# ✅ Valid
max_queue_size = 1
max_queue_size = 3      # Default
max_queue_size = 10
max_queue_size = 100    # Maximum
```

---

### Workspace Path Errors

**Symptom:**
```
Error: ✗ Workspace path '{path}' does not exist or is not a directory. Check your switchboard.toml configuration or create the directory.
```

**Cause:** Workspace path specified in configuration doesn't exist or is not a directory.

**Resolution:**

1. Check the configured workspace path:
   ```toml
   [settings]
   workspace_path = "./workspace"  # Must exist as a directory
   ```

2. Verify the path exists:
   ```bash
   ls -la ./workspace
   ```

3. If it doesn't exist, create it:
   ```bash
   mkdir -p ./workspace
   ```

4. Ensure it's a directory, not a file:
   ```bash
   test -d ./workspace && echo "Is directory" || echo "Not a directory"
   ```

5. Check permissions:
   ```bash
   ls -ld ./workspace
   ```

---

## Skills Issues

This section covers common issues related to skill installation, configuration, and usage.

---

### npx not found

**Symptom:**
```
Error: npx not found during skill installation
Error: Failed to execute npx: No such file or directory
```

**Cause:** Node.js/npx is not installed or not in the system PATH.

**Resolution:**

1. Install Node.js (which includes npx):
   ```bash
   # Linux (using NodeSource)
   curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
   sudo apt-get install -y nodejs

   # macOS (using Homebrew)
   brew install node

   # Windows (using Chocolatey)
   choco install nodejs
   ```

2. Verify npx is installed:
   ```bash
   npx --version
   ```

3. If Node.js is installed but npx is not found, try:
   ```bash
   npm install -g npx
   ```

4. Ensure Node.js is in your PATH:
   ```bash
   export PATH="$PATH:/usr/local/bin"  # Add to ~/.bashrc or ~/.zshrc
   ```

---

### Invalid skill source format

**Symptom:**
```
Error: Invalid skill source format: '{invalid_source}'
Error: Skill source must be in format 'owner/repo' or 'owner/repo@skill-name'
```

**Cause:** The skill source specified in switchboard.toml doesn't match expected format.

**Resolution:**

1. Check your skill configuration in switchboard.toml:
   ```toml
   # ❌ Invalid formats
   skill_source = "github.com/owner/repo"
   skill_source = "https://github.com/owner/repo"
   skill_source = "owner-repo"

   # ✅ Valid formats
   skill_source = "owner/repo"
   skill_source = "owner/repo@skill-name"
   skill_source = "kilocode/skill-example"
   skill_source = "kilocode/skill-example@code-review"
   ```

2. Validate your config:
   ```bash
   switchboard validate
   ```

---

### Skill installation failures

**Symptom:**
```
Error: Skill installation failed with exit code {code}
Warning: Failed to install skill from '{source}'
```

**Cause:** The skill installation process failed, possibly due to missing dependencies, permission issues, or skill-specific errors.

**Resolution:**

1. Check the skill installation logs:
   ```bash
   switchboard skills install --verbose
   switchboard logs --skill {skill_name}
   ```

2. Common causes and fixes:
   - **Missing dependencies:** Ensure npx and git are installed
   - **Permission issues:** Check file permissions in skills directory
   - **Repository access:** Verify the skill repository is public or you have access

3. Try reinstalling the skill:
   ```bash
   switchboard skills uninstall {skill_name}
   switchboard skills install {skill_source}
   ```

4. Check for skill-specific requirements in the skill's documentation

---

### Duplicate skill entries

**Symptom:**
```
Warning: Duplicate skill entry '{skill_name}' found in switchboard.toml
Error: Multiple skills with the same name defined
```

**Cause:** The same skill is defined multiple times in switchboard.toml.

**Resolution:**

1. Check switchboard.toml for duplicate skill entries:
   ```toml
   # ❌ Duplicate - will cause warning
   [[skill]]
   name = "code-review"
   source = "owner/repo1"

   [[skill]]
   name = "code-review"  # Duplicate!
   source = "owner/repo2"

   # ✅ Valid - unique names
   [[skill]]
   name = "code-review"
   source = "owner/repo1"

   [[skill]]
   name = "security-scan"
   source = "owner/repo2"
   ```

2. Deduplicate your config:
   - Remove duplicate [[skill]] entries
   - Or rename skills to have unique names

3. Validate the config:
   ```bash
   switchboard validate
   ```

---

### Network issues during skill installation

**Symptom:**
```
Error: Network timeout while fetching skill from '{source}'
Error: Connection refused: Could not connect to GitHub
Error: Failed to clone repository: Operation timed out
```

**Cause:** Network connectivity issues when trying to download/install skills from remote repositories.

**Resolution:**

1. Check your internet connection:
   ```bash
   curl -I https://github.com
   ping github.com
   ```

2. Verify Git is configured properly:
   ```bash
   git config --list
   git remote -v
   ```

3. Check for firewall/proxy issues:
   - Corporate firewalls may block GitHub access
   - Proxy settings may need configuration

4. Configure Git proxy if needed:
   ```bash
   git config --global http.proxy http://proxy.example.com:8080
   git config --global https.proxy http://proxy.example.com:8080
   ```

5. Try again with increased timeout:
   ```bash
   GIT_TIMEOUT=60 switchboard skills install {source}
   ```

---

### Malformed SKILL.md warnings

**Symptom:**
```
Warning: Malformed SKILL.md for skill '{name}': Invalid frontmatter
Warning: SKILL.md is missing required fields: name, version
```

**Cause:** The SKILL.md file in the skill repository has invalid or missing frontmatter fields.

**Resolution:**

1. Check the SKILL.md format in the skill repository:
   ```markdown
   ---
   name: "my-skill"
   version: "1.0.0"
   description: "A skill that does something useful"
   author: "owner"
   ---

   # Skill Content
   Your skill documentation here...
   ```

2. Required frontmatter fields:
   - `name`: Skill name (required)
   - `version`: Semantic version (required)

3. Optional frontmatter fields:
   - `description`: What the skill does
   - `author`: Who created the skill
   - `homepage`: Link to skill documentation
   - `repository`: Source code location

4. If you're a skill author, fix the SKILL.md:
   ```markdown
   ---
   name: "example-skill"
   version: "1.0.0"
   description: "Example skill description"
   author: "your-username"
   ---
   ```

5. Validate your skill structure:
   ```bash
   switchboard skills validate
   ```

---

## Scheduler/Metrics Issues

### Cron Schedule Problems

#### Wrong timezone

**Symptom:**
```
Warning: Agent '{name}' scheduled in wrong timezone
Agent not running at expected time
```

**Cause:** System timezone differs from configured timezone, or timezone is not set correctly.

**Resolution:**

1. Check configured timezone:
   ```toml
   [settings]
   timezone = "America/New_York"
   ```

2. Verify system timezone:
   ```bash
   # Linux/macOS
   date
   timedatectl  # Linux only

   # Check TZ environment variable
   echo $TZ
   ```

3. Use `switchboard list` to see next run times with timezone information:
   ```bash
   switchboard list
   ```

4. If using `timezone = "system"`, ensure your system timezone is correctly set

5. Test scheduling with a temporary agent to verify timezone:

   ```toml
   [[agent]]
   name = "timezone-test"
   schedule = "*/1 * * * *"  # Every minute
   prompt = "Testing timezone"
   timezone = "America/New_York"
   ```

---

#### Next run calculation failed

**Symptom:**
```
Invalid cron schedule '{schedule}': {error}
Failed to calculate next run time for agent '{name}'
```

**Cause:** Cron expression is invalid or incompatible with scheduler.

**Resolution:**

1. Validate cron expression:
   ```bash
   switchboard validate
   ```

2. Verify cron format (5 fields: minute hour day month weekday):
   ```toml
   # ✅ Valid
   schedule = "0 */6 * * *"
   schedule = "*/30 * * * *"
   schedule = "0 2 * * 1"

   # ❌ Invalid (special macros not all supported)
   schedule = "@yearly"
   schedule = "@annually"
   ```

3. Check for edge cases:
   ```toml
   # ❌ Invalid (February 30 doesn't exist)
   schedule = "0 0 30 2 *"

   # ❌ Invalid (hour 25 doesn't exist)
   schedule = "0 25 * * *"
   ```

4. Use `switchboard list` to see calculated next run times:
   ```bash
   switchboard list
   ```

---

#### Job registration failed

**Symptom:**
```
Failed to register agent '{name}': {error}
```

**Cause:** Scheduler unable to register cron job, possibly due to duplicate job IDs or scheduler internal errors.

**Resolution:**

1. Check for duplicate agent names in configuration:
   ```bash
   grep "name = " switchboard.toml
   ```

2. Validate configuration:
   ```bash
   switchboard validate
   ```

3. Restart scheduler if it's in a bad state:
   ```bash
   switchboard down
   switchboard up
   ```

4. Check for stale PID files:
   ```bash
   ls -la .switchboard/pid
   ```

5. If scheduler crashed, remove PID file and restart:
   ```bash
   rm .switchboard/pid
   switchboard up
   ```

---

### Overlap Handling Issues

#### Skipped runs (Skip mode)

**Symptom:**
```
Warning: Agent '{name}' is already running (container_id: {id}), overlap_mode=skip, skipping new run
```

**Cause:** Agent is already running and overlap_mode is set to `Skip`.

**Resolution:**

This is expected behavior when using `overlap_mode = "Skip"`. If you want to queue runs instead, change to `Queue` mode:

```toml
[[agent]]
name = "my-agent"
overlap_mode = "Queue"    # Queue runs instead of skipping
max_queue_size = 5        # Optional: increase queue size
```

Or adjust the schedule so runs don't overlap:

```toml
[[agent]]
name = "my-agent"
schedule = "0 */12 * * *"  # Every 12 hours instead of every 6
timeout = "1h"             # Shorter timeout to reduce overlap risk
```

---

#### Queue full (Queue mode)

**Symptom:**
```
Warning: Agent '{name}' queue is full (max {size}), skipping scheduled run
```

**Cause:** Agent's run queue is full (max_queue_size reached) and overlap_mode is `Queue`.

**Resolution:**

1. Increase the queue size:

   ```toml
   [[agent]]
   name = "my-agent"
   overlap_mode = "Queue"
   max_queue_size = 10     # Increase from default of 3
   ```

2. Reduce frequency of scheduled runs:

   ```toml
   [[agent]]
   name = "my-agent"
   schedule = "0 */12 * * *"  # Every 12 hours instead of more frequent
   ```

3. Reduce agent execution time:
   - Shorten timeout if possible
   - Optimize prompts to be more concise
   - Use readonly mode for read-only tasks

4. Check for stuck runs preventing queue progress:
   ```bash
   docker ps | grep switchboard-agent
   switchboard logs
   ```

---

#### Queued run execution failed

**Symptom:**
```
Error executing queued run for agent '{name}': {error}
```

**Cause:** Error occurred while executing a queued run, possibly due to Docker issues or configuration changes.

**Resolution:**

1. Check agent logs for detailed error:
   ```bash
   switchboard logs {agent_name}
   ```

2. Verify Docker is running and healthy:
   ```bash
   docker ps
   docker info
   ```

3. Check if configuration changed since run was queued:
   ```bash
   git diff switchboard.toml  # If using version control
   ```

4. Restart scheduler to clear bad state:
   ```bash
   switchboard down
   switchboard up
   ```

5. Inspect failed container if it still exists:
   ```bash
   docker ps -a | grep switchboard-agent
   docker logs {container_id}
   ```

---

#### Configuration not found for queued run

**Symptom:**
```
Warning: Agent configuration not found for queued run: '{name}'
```

**Cause:** Agent was removed from configuration while a run was queued, or configuration file was modified.

**Resolution:**

1. Check if agent exists in current configuration:
   ```bash
   grep -A 10 "name = \"${agent_name}\"" switchboard.toml
   ```

2. If agent was intentionally removed, ignore this warning (it's just cleaning up)

3. If agent should exist, check for configuration errors:
   ```bash
   switchboard validate
   ```

4. Restart scheduler to reload configuration:
   ```bash
   switchboard down
   switchboard up
   ```

---

### Metrics Collection Issues

#### Metrics update failed

**Symptom:**
```
Failed to update metrics with {detail}: {error}
```

**Cause:** Unable to write metrics data, possibly due to file system issues or concurrent access.

**Resolution:**

1. Check if metrics directory exists:
   ```bash
   ls -la .switchboard/metrics
   ```

2. Create metrics directory if missing:
   ```bash
   mkdir -p .switchboard/metrics
   ```

3. Check disk space:
   ```bash
   df -h
   ```

4. Check file permissions:
   ```bash
   ls -la .switchboard/
   ```

5. If metrics file is corrupted, delete and let it regenerate:
   ```bash
   rm .switchboard/metrics/*.json
   ```

---

#### Corrupted metrics file

**Symptom:**
```
Failed to load metrics: {error}
switchboard metrics shows no data or errors
```

**Cause:** Metrics JSON file is corrupted or invalid.

**Resolution:**

1. Check metrics file contents:
   ```bash
   cat .switchboard/metrics/*.json
   ```

2. Validate JSON syntax:
   ```bash
   jq . .switchboard/metrics/*.json
   ```

3. If file is corrupted, delete it:
   ```bash
   rm .switchboard/metrics/*.json
   ```

4. Metrics will be regenerated automatically on next run

5. To manually reset all metrics:
   ```bash
   rm -rf .switchboard/metrics
   mkdir -p .switchboard/metrics
   ```

---

#### Missing metrics file

**Symptom:**
```
Failed to load metrics: {error}
switchboard metrics shows "No metrics data available"
```

**Cause:** Metrics file hasn't been created yet or was deleted.

**Resolution:**

1. Check if metrics directory exists:
   ```bash
   ls -la .switchboard/metrics
   ```

2. If missing, create directory:
   ```bash
   mkdir -p .switchboard/metrics
   ```

3. Run an agent to generate metrics:
   ```bash
   switchboard run {agent_name}
   ```

4. Start scheduler to begin collecting metrics:
   ```bash
   switchboard up
   ```

5. Verify metrics are being collected:
   ```bash
   switchboard metrics
   ```

---

#### Concurrent writes

**Symptom:**
```
Failed to update metrics: {error}
Metrics data appears inconsistent or missing runs
```

**Cause:** Multiple scheduler instances or processes trying to write metrics simultaneously.

**Resolution:**

1. Check for multiple scheduler instances:
   ```bash
   ps aux | grep switchboard
   ```

2. Stop all running schedulers:
   ```bash
   switchboard down
   ```

3. Check for stale PID files:
   ```bash
   ls -la .switchboard/pid
   ```

4. Remove stale PID file if scheduler is not actually running:
   ```bash
   rm .switchboard/pid
   ```

5. Start single scheduler instance:
   ```bash
   switchboard up
   ```

---

## Performance Issues

### Resource Limitations

#### Container timeout (exit code 137)

**Symptom:**
```
Container exited with code 137
Agent run timed out and was killed
```

**Cause:** Agent execution exceeded the configured timeout limit and was forcibly terminated.

**Resolution:**

Exit code 137 indicates the container was killed (SIGKILL). This typically happens when:

1. **Timeout exceeded:**
   - Increase timeout value:
     ```toml
     [[agent]]
     name = "my-agent"
     timeout = "2h"  # Increase from 30m or 1h
     ```

2. **Agent is too slow for the task:**
   - Optimize the prompt to be more concise
   - Break large tasks into smaller, focused agents
   - Use readonly mode for read-only tasks

3. **Container is doing too much work:**
   - Review agent logs to see what it's doing:
     ```bash
     switchboard logs {agent_name}
     ```
   - Consider splitting the workload

4. **External dependencies are slow:**
   - Check network connectivity
   - Verify external APIs are responding

---

#### High memory usage

**Symptom:**
```
System running low on memory
Container consuming excessive memory
Agent runs fail due to OOM (Out of Memory)
```

**Cause:** Agent is processing large amounts of data or has memory leaks.

**Resolution:**

1. Check container memory usage:
   ```bash
   docker stats
   ```

2. Inspect memory limits:
   ```bash
   docker inspect {container_id} | grep -i memory
   ```

3. Set memory limits in Docker if not already set:
   ```bash
   docker run --memory="2g" --memory-swap="2g" ...
   ```

4. Reduce agent workload:
   - Process smaller batches of data
   - Use filters to limit scope
   - Split large tasks into multiple agents

5. Use readonly mode when possible:
   ```toml
   [[agent]]
   name = "read-only-agent"
   readonly = true
   ```

---

#### Disk exhaustion

**Symptom:**
```
No space left on device
Logs cannot be written
Docker image pulls fail
```

**Cause:** Disk space exhausted by containers, images, or logs.

**Resolution:**

1. Check disk space:
   ```bash
   df -h
   ```

2. Check Docker disk usage:
   ```bash
   docker system df
   ```

3. Clean up unused Docker resources:
   ```bash
   # Remove stopped containers
   docker container prune

   # Remove unused images
   docker image prune -a

   # Remove unused volumes
   docker volume prune

   # Remove all unused resources
   docker system prune -a --volumes
   ```

4. Clean up log files:
   ```bash
   # Check log directory size
   du -sh .switchboard/logs

   # Rotate or compress old logs
   find .switchboard/logs -name "*.log" -mtime +7 -delete
   ```

5. Configure log rotation if running long-term:
   - The logger module supports rotation
   - Set appropriate retention policies

---

#### CPU throttling

**Symptom:**
```
Agent runs are slower than expected
High CPU usage from containers
System becomes unresponsive
```

**Cause:** Multiple agents running simultaneously, CPU-intensive tasks, or insufficient CPU resources.

**Resolution:**

1. Check CPU usage:
   ```bash
   top
   htop
   docker stats
   ```

2. Set CPU limits on containers:
   ```bash
   docker run --cpus="2.0" ...
   docker run --cpus="0.5" ...
   ```

3. Stagger agent schedules to reduce concurrent load:
   ```toml
   [[agent]]
   name = "agent-1"
   schedule = "0 0 * * *"    # Midnight

   [[agent]]
   name = "agent-2"
   schedule = "0 2 * * *"    # 2 AM (2 hours later)

   [[agent]]
   name = "agent-3"
   schedule = "0 4 * * *"    # 4 AM (another 2 hours later)
   ```

4. Use overlap_mode to limit concurrent runs:
   ```toml
   [settings]
   overlap_mode_str = "Skip"  # Don't queue, just skip if already running
   ```

5. Reduce agent workload or optimize prompts

---

### Timeout-Related Problems

#### Consistent timeouts

**Symptom:**
```
Agent consistently times out on every run
All runs exit with code 137
```

**Cause:** Timeout value is too short for the agent's workload.

**Resolution:**

1. Review agent logs to see what's happening:
   ```bash
   switchboard logs {agent_name}
   ```

2. Increase timeout value:
   ```toml
   [[agent]]
   name = "slow-agent"
   timeout = "4h"  # Increase from 1h or 2h
   ```

3. Break large tasks into smaller agents:
   ```toml
   # Instead of one large agent
   [[agent]]
   name = "full-audit"
   schedule = "0 2 * * 0"
   prompt = "Do everything..."

   # Split into focused agents
   [[agent]]
   name = "security-scan"
   schedule = "0 2 * * 1"
   prompt = "Scan for security vulnerabilities..."

   [[agent]]
   name = "code-review"
   schedule = "0 2 * * 2"
   prompt = "Review recent code changes..."
   ```

4. Use readonly mode when possible:
   ```toml
   [[agent]]
   name = "read-only-agent"
   readonly = true
   ```

---

#### Graceful termination issues

**Symptom:**
```
Container doesn't terminate cleanly
Data loss during shutdown
Incomplete work after timeout
```

**Cause:** Agent doesn't handle SIGTERM signals properly, or timeout is too short for graceful shutdown.

**Resolution:**

1. Check container logs for shutdown behavior:
   ```bash
   switchboard logs {agent_name} --tail 100
   ```

2. Increase timeout to allow graceful shutdown:
   ```toml
   [[agent]]
   name = "my-agent"
   timeout = "2h"  # Give more time for clean shutdown
   ```

3. The Docker timeout mechanism sends SIGTERM first, then SIGKILL after 10 seconds
   - If agent needs more time, consider implementing signal handling in the agent
   - Use the timeout value as the total budget, with 10 seconds for SIGTERM → SIGKILL transition

4. For critical workloads, use checkpoints or save intermediate progress

---

### Log File Growth

**Symptom:**
```
Log files consuming large amounts of disk space
.switchboard/logs directory growing continuously
```

**Cause:** Logs are accumulating over time without rotation or cleanup.

**Resolution:**

1. Check log directory size:
   ```bash
   du -sh .switchboard/logs
   du -sh .gastode/logs/*/  # Per-agent breakdown
   ```

2. View log files and their sizes:
   ```bash
   ls -lh .switchboard/logs/*/
   ```

3. Implement log rotation:

   The logger module supports log rotation. Configure retention:

   ```toml
   [settings]
   log_dir = ".switchboard/logs"
   # Log rotation is handled by the logger module
   # Old logs are rotated automatically
   ```

4. Clean up old logs manually if needed:
   ```bash
   # Remove logs older than 7 days
   find .switchboard/logs -name "*.log" -mtime +7 -delete

   # Compress logs older than 3 days
   find .switchboard/logs -name "*.log" -mtime +3 -exec gzip {} \;
   ```

5. Use `switchboard logs --tail` to limit output:
   ```bash
   switchboard logs --tail 50  # Show last 50 lines only
   ```

6. Consider using log aggregation tools for production deployments

---

## Operational Issues

### .kilocode Directory Problems

#### Missing .kilocode directory

**Symptom:**
```
Error: .kilocode directory not found
Error: Failed to locate Kilo Code configuration
Error: Missing .kilocode/config.json
```

**Cause:** The `.kilocode` directory doesn't exist in the expected location.

**Resolution:**

1. Check for `.kilocode` directory:
   ```bash
   # Check home directory
   ls -la ~/.kilocode

   # Check current project directory
   ls -la .kilocode

   # Find all .kilocode directories
   find ~ -type d -name ".kilocode" 2>/dev/null
   ```

2. Create the directory structure:
   ```bash
   # In home directory (recommended)
   mkdir -p ~/.kilocode/logs

   # Or in project directory
   mkdir -p .kilocode/logs
   ```

3. Create required configuration files:

   `~/.kilocode/config.json`:
   ```json
   {
     "version": "1.0.0",
     "default_provider": "anthropic"
   }
   ```

   `~/.kilocode/api_keys.json`:
   ```json
   {
     "anthropic": "your-anthropic-api-key-here",
     "openai": "your-openai-api-key-here"
   }
   ```

4. Add `.kilocode/` to `.gitignore`:
   ```bash
   echo ".kilocode/" >> .gitignore
   ```

---

#### Missing API keys

**Symptom:**
```
Error: Missing API keys in .kilocode/api_keys.json
Agent fails to authenticate with AI service
```

**Cause:** API keys are not configured in `.kilocode/api_keys.json`.

**Resolution:**

1. Check if API keys file exists:
   ```bash
   ls -la ~/.kilocode/api_keys.json
   ```

2. Create or update the API keys file:

   `~/.kilocode/api_keys.json`:
   ```json
   {
     "anthropic": "your-anthropic-api-key-here",
     "openai": "your-openai-api-key-here"
   }
   ```

3. Obtain API keys:
   - Anthropic: https://console.anthropic.com/
   - OpenAI: https://platform.openai.com/api-keys

4. Secure the file:
   ```bash
   chmod 600 ~/.kilocode/api_keys.json
   ```

5. Never commit API keys to version control:
   ```bash
   echo ".kilocode/api_keys.json" >> .gitignore
   ```

---

#### Missing config.json

**Symptom:**
```
Error: Missing .kilocode/config.json
Configuration file not found
```

**Cause:** The `config.json` file in `.kilocode` doesn't exist.

**Resolution:**

1. Check if config file exists:
   ```bash
   ls -la ~/.kilocode/config.json
   ```

2. Create the configuration file:

   `~/.kilocode/config.json`:
   ```json
   {
     "version": "1.0.0",
     "default_provider": "anthropic",
     "timeout": 300,
     "max_retries": 3
   }
   ```

3. Verify JSON is valid:
   ```bash
   jq . ~/.kilocode/config.json
   ```

4. Check file permissions:
   ```bash
   ls -la ~/.kilocode/
   ```

---

### Image Build Issues

#### Build fails

**Symptom:**
```
Failed to build agent image: {error}
Build command exits with error
```

**Cause:** Dockerfile errors, missing dependencies, network issues, or resource constraints.

**Resolution:**

1. Check the Dockerfile:
   ```bash
   cat Dockerfile
   ```

2. Try building with verbose output:
   ```bash
   switchboard build --no-cache
   ```

3. Check Docker daemon is running:
   ```bash
   docker ps
   ```

4. Verify disk space:
   ```bash
   df -h
   ```

5. Check for network connectivity (if pulling base images):
   ```bash
   ping -c 3 registry-1.docker.io
   ```

6. Review build logs for specific error messages:
   ```bash
   # The build command outputs logs in real-time
   # Look for specific error messages
   ```

7. If using custom Dockerfile, verify:
   - Base image is accessible
   - All dependencies are available
   - Syntax is correct
   - No typos in commands

---

#### Image not found

**Symptom:**
```
Failed to find image: {image_name}:{image_tag}
Error: No such image: switchboard-agent:latest
```

**Cause:** Docker image hasn't been built yet or was deleted.

**Resolution:**

1. Check available images:
   ```bash
   docker images | grep switchboard-agent
   ```

2. Build the image:
   ```bash
   switchboard build
   ```

3. Verify image was built:
   ```bash
   docker images | grep switchboard-agent
   ```

4. If build fails, see "Build fails" section above

5. If image was deleted, rebuild it:
   ```bash
   switchboard build
   ```

---

#### Base image pull fails

**Symptom:**
```
Failed to pull base image: {error}
Error pulling image from registry
```

**Cause:** Network issues, authentication problems, or registry unavailable.

**Resolution:**

1. Check network connectivity:
   ```bash
   ping -c 3 registry-1.docker.io
   ```

2. Try pulling the base image manually:
   ```bash
   docker pull ubuntu:latest
   ```

3. Check if you're behind a proxy or firewall:
   - Configure Docker proxy settings
   - Check firewall rules allow Docker registry access

4. Verify Docker daemon is running:
   ```bash
   docker ps
   ```

5. Check disk space:
   ```bash
   df -h
   ```

6. If using private registry, authenticate:
   ```bash
   docker login your-registry.com
   ```

7. If using a different base image, update Dockerfile

---

### Log Viewing Problems

#### No logs found

**Symptom:**
```
No logs found for agent '{agent_name}'
Log directory is empty
```

**Cause:** Agent hasn't run yet, logs were deleted, or logs are in a different location.

**Resolution:**

1. Check if log directory exists:
   ```bash
   ls -la .switchboard/logs
   ```

2. Check log files:
   ```bash
   find .switchboard/logs -name "*.log"
   ```

3. Run an agent to generate logs:
   ```bash
   switchboard run {agent_name}
   ```

4. Start scheduler to begin collecting logs:
   ```bash
   switchboard up
   ```

5. Check log directory configuration:
   ```toml
   [settings]
   log_dir = ".switchboard/logs"
   ```

6. Verify log file permissions:
   ```bash
   ls -la .switchboard/logs/*/
   ```

---

#### Logs not updating

**Symptom:**
```
switchboard logs --follow shows no new output
Log file is stale
```

**Cause:** Agent is not running, log file rotation occurred, or scheduler is not functioning.

**Resolution:**

1. Check if scheduler is running:
   ```bash
   ps aux | grep switchboard
   ```

2. Check if agent containers are running:
   ```bash
   docker ps | grep switchboard-agent
   ```

3. Check if new log files exist:
   ```bash
   ls -lt .switchboard/logs/*/ | head -20
   ```

4. Restart scheduler:
   ```bash
   switchboard down
   switchboard up
   ```

5. Check Docker logs directly:
   ```bash
   docker logs {container_id} --follow
   ```

---

### Command Usage Errors

#### Invalid command syntax

**Symptom:**
```
error: unexpected argument '{arg}' found
error: required argument '{arg}' was not provided
```

**Cause:** Incorrect command-line arguments or flags.

**Resolution:**

1. Get help for a command:
   ```bash
   switchboard --help
   switchboard {command} --help
   ```

2. Common command examples:

   ```bash
   # Validate configuration
   switchboard validate
   switchboard validate --config /path/to/config.toml

   # Build image
   switchboard build
   switchboard build --no-cache
   switchboard build --config /path/to/config.toml

   # Run agent
   switchboard run my-agent
   switchboard run my-agent --config /path/to/config.toml

   # Start scheduler
   switchboard up
   switchboard up --detach

   # Stop scheduler
   switchboard down
   switchboard down --cleanup

   # List agents
   switchboard list

   # View logs
   switchboard logs
   switchboard logs my-agent
   switchboard logs --follow
   switchboard logs --tail 100
   switchboard logs my-agent --follow --tail 50

   # View metrics
   switchboard metrics
   switchboard metrics --detailed
   switchboard metrics --agent my-agent
   ```

3. Ensure agent name matches configuration:
   ```bash
   grep "name = " switchboard.toml
   ```

---

#### Config file not found

**Symptom:**
```
Error: Configuration file not found: {path}
```

**Cause:** Configuration file doesn't exist at specified path or default location.

**Resolution:**

1. Check default location:
   ```bash
   ls -la ./switchboard.toml
   ```

2. Specify config file explicitly:
   ```bash
   switchboard --config /path/to/switchboard.toml validate
   ```

3. Create configuration file if missing:
   ```bash
   # Copy sample config
   cp switchboard.sample.toml switchboard.toml

   # Or create manually
   cat > switchboard.toml << EOF
   version = "0.1.0"

   [[agent]]
   name = "my-agent"
   schedule = "0 */6 * * *"
   prompt = "Hello, world!"
   EOF
   ```

4. Validate configuration:
   ```bash
   switchboard validate
   ```

---

## Debugging Guidance

### Debugging Tools

#### switchboard validate

Validate configuration without starting the scheduler:

```bash
# Validate default config file
switchboard validate

# Validate custom config file
switchboard validate --config /path/to/switchboard.toml
```

**Use for:**
- Checking TOML syntax
- Verifying cron expressions
- Validating timeout formats
- Checking prompt file paths
- Verifying agent names are unique

---

#### switchboard list

Display all configured agents with next run times:

```bash
# List all agents
switchboard list

# Output shows:
# - Agent name
# - Schedule
# - Overlap mode
# - Next run time
# - Current run status
```

**Use for:**
- Seeing agent schedules
- Checking next run times
- Verifying agent configuration
- Monitoring scheduler state

---

#### switchboard logs

View logs from agent runs:

```bash
# View all agent logs (last 50 lines)
switchboard logs

# View logs for specific agent
switchboard logs {agent_name}

# Follow logs in real-time
switchboard logs --follow
switchboard logs {agent_name} --follow

# Show last N lines
switchboard logs --tail 100
switchboard logs {agent_name} --tail 200

# Combine options
switchboard logs {agent_name} --follow --tail 50
```

**Use for:**
- Debugging agent failures
- Monitoring agent progress
- Checking timeout behavior
- Reviewing execution history

---

#### switchboard metrics

Display agent execution metrics:

```bash
# Show summary metrics
switchboard metrics

# Show detailed metrics
switchboard metrics --detailed

# Show metrics for specific agent
switchboard metrics --agent {agent_name}

# Combine options
switchboard metrics --agent my-agent --detailed
```

**Use for:**
- Tracking agent performance
- Identifying slow runs
- Monitoring success/failure rates
- Analyzing execution patterns

---

#### docker ps

List running containers:

```bash
# List all running containers
docker ps

# List all containers (including stopped)
docker ps -a

# Show container IDs and names
docker ps --format "table {{.ID}}\t{{.Names}}\t{{.Status}}"
```

**Use for:**
- Checking if agent containers are running
- Verifying container names
- Monitoring container status
- Finding container IDs for inspection

---

#### docker logs

View container logs:

```bash
# View container logs
docker logs {container_id}

# Follow logs in real-time
docker logs --follow {container_id}

# Show last N lines
docker logs --tail 100 {container_id}

# Show timestamps
docker logs --timestamps {container_id}

# Since specific time
docker logs --since 2024-02-15T10:00:00 {container_id}
```

**Use for:**
- Debugging container startup issues
- Viewing agent output
- Checking for errors
- Monitoring execution

---

#### docker inspect

Inspect container details:

```bash
# Inspect container
docker inspect {container_id}

# Show specific fields
docker inspect --format='{{.State.Status}}' {container_id}
docker inspect --format='{{.Config.Image}}' {container_id}
docker inspect --format='{{range .Mounts}}{{.Source}} -> {{.Destination}}{{"\n"}}{{end}}' {container_id}

# Show all labels
docker inspect --format='{{json .Config.Labels}}' {container_id} | jq
```

**Use for:**
- Checking container configuration
- Viewing mounted volumes
- Inspecting environment variables
- Understanding container state

---

### Log File Locations

Logs are stored in the configured log directory (default: `.switchboard/logs/`):

```
.switchboard/
├── logs/
│   ├── agent-1/
│   │   ├── 2024-02-15T10-00-00.123456Z.log
│   │   ├── 2024-02-15T16-00-00.234567Z.log
│   │   └── ...
│   ├── agent-2/
│   │   ├── 2024-02-15T11-00-00.345678Z.log
│   │   └── ...
│   └── ...
├── metrics/
│   └── metrics.json
└── pid
```

**Log file naming:** `{timestamp}.log`

**Log format:**
- Timestamps in ISO 8601 UTC format
- Each agent has its own subdirectory
- Logs are rotated automatically

**Viewing logs:**
```bash
# List all log files
find .switchboard/logs -name "*.log"

# View most recent log for an agent
ls -t .switchboard/logs/{agent_name}/ | head -1 | xargs -I {} cat .switchboard/logs/{agent_name}/{}

# Tail a specific log file
tail -f .switchboard/logs/{agent_name}/2024-02-15T10-00-00.123456Z.log
```

---

### Common Debugging Workflows

#### Workflow 1: Agent not running at expected time

1. Check agent configuration:
   ```bash
   switchboard validate
   ```

2. Verify agent is scheduled:
   ```bash
   switchboard list
   ```

3. Check scheduler is running:
   ```bash
   ps aux | grep switchboard
   ```

4. Check for scheduler errors in logs:
   ```bash
   switchboard logs --follow
   ```

5. Verify Docker daemon is running:
   ```bash
   docker ps
   ```

6. Try running agent manually:
   ```bash
   switchboard run {agent_name}
   ```

---

#### Workflow 2: Agent failing consistently

1. View agent logs:
   ```bash
   switchboard logs {agent_name} --tail 100
   ```

2. Check container logs:
   ```bash
   docker ps -a | grep switchboard-agent
   docker logs {container_id}
   ```

3. Inspect container:
   ```bash
   docker inspect {container_id}
   ```

4. Check agent metrics:
   ```bash
   switchboard metrics --agent {agent_name} --detailed
   ```

5. Run agent manually to reproduce:
   ```bash
   switchboard run {agent_name}
   ```

---

#### Workflow 3: Scheduler not starting

1. Check configuration:
   ```bash
   switchboard validate
   ```

2. Check Docker availability:
   ```bash
   docker ps
   docker info
   ```

3. Verify `.kilocode` directory:
   ```bash
   ls -la ~/.kilocode/config.json
   ls -la ~/.kilocode/api_keys.json
   ```

4. Check for existing scheduler:
   ```bash
   ps aux | grep switchboard
   ```

5. Check for stale PID file:
   ```bash
   ls -la .switchboard/pid
   ```

6. View error logs:
   ```bash
   switchboard logs --tail 50
   ```

---

#### Workflow 4: Container timeout issues

1. Check agent logs for timeout:
   ```bash
   switchboard logs {agent_name} | grep -i timeout
   ```

2. Check exit codes (137 = killed):
   ```bash
   docker ps -a | grep switchboard-agent
   ```

3. Review timeout configuration:
   ```bash
   grep timeout switchboard.toml
   ```

4. Increase timeout:
   ```toml
   [[agent]]
   name = "my-agent"
   timeout = "2h"  # Increase
   ```

5. Check container resource usage:
   ```bash
   docker stats
   ```

---

### Exit Code Reference

Common Docker container exit codes:

| Exit Code | Meaning | Typical Cause |
|-----------|---------|---------------|
| 0 | Success | Container completed successfully |
| 1 | Application error | Agent encountered an error |
| 125 | Docker daemon error | Docker daemon issue |
| 126 | Container command not executable | Command cannot be invoked |
| 127 | Container command not found | Command not found in PATH |
| 137 | Container killed (SIGKILL) | Timeout or manual kill |
| 139 | Segmentation fault | Application crash |
| 143 | Container stopped (SIGTERM) | Graceful termination |

**Common scenarios:**

- **Exit code 137**: Container was killed, typically due to timeout or memory limit
  - Check timeout configuration
  - Review agent workload
  - Check for resource constraints

- **Exit code 125**: Docker daemon issue
  - Check Docker daemon is running
  - Verify Docker installation
  - Check Docker logs

- **Exit code 126/127**: Container command error
  - Verify Dockerfile is correct
  - Check base image
  - Verify command syntax

---

### Known Issues

#### Partial Shutdown Behavior

**Issue:** When stopping containers fails in `switchboard down`, the function continues instead of aborting, potentially leaving some containers running.

**Impact:** Medium - Users may experience inconsistent system state where `switchboard down` appears to succeed but some containers remain running.

**Workaround:**
1. Check running containers:
   ```bash
   docker ps | grep switchboard-agent
   ```

2. Manually stop any remaining containers:
   ```bash
   docker stop {container_id}
   ```

3. Use `docker kill` if stop fails:
   ```bash
   docker kill -f {container_id}
   ```

**Reference:** See [`BUGS.md`](../BUGS.md) BUG-009

---

#### Static Mutable References in Tests

**Issue:** Creating shared references to mutable static `GLOBAL_LOG_DIR` triggers Rust 2024 compatibility warnings.

**Impact:** Low - Tests work but may become errors in future Rust editions.

**Workaround:** No immediate action required. This is a code quality issue for future compatibility.

**Reference:** See [`BUGS.md`](../BUGS.md) BUG-006, BUG-008

---

#### Unwrap Warnings

**Issue:** Several instances of unnecessary `.unwrap()` calls after `.is_some()` checks are flagged by clippy.

**Impact:** Low - Code works correctly but prevents compilation with `-D warnings` flag.

**Workaround:** No runtime impact. These will be fixed in future releases.

**Reference:** See [`BUGS.md`](../BUGS.md) BUG-001 through BUG-005

---

### When to Seek Help

If you've tried the troubleshooting steps above and still can't resolve your issue:

1. **Gather diagnostic information:**
   ```bash
   # Version information
   switchboard --version

   # Configuration
   cat switchboard.toml

   # Validation output
   switchboard validate

   # Scheduler status
   switchboard list

   # Recent logs
   switchboard logs --tail 100

   # Docker information
   docker --version
   docker ps -a | grep switchboard-agent
   ```

2. **Search existing issues:**
   - Check the [GitHub Issues](https://github.com/your-org/switchboard/issues)
   - Search for similar error messages

3. **Create a new issue with:**
   - switchboard version
   - Operating system and version
   - Rust version (`rustc --version`)
   - Docker version (`docker --version`)
   - Full error message
   - Steps to reproduce
   - What you've already tried
   - Diagnostic information from step 1

4. **Consider these resources:**
   - [README.md](../README.md) - Getting started and basic troubleshooting
   - [Installation Troubleshooting Guide](INSTALLATION_TROUBLESHOOTING.md) - Installation issues
   - [Architecture Documentation](../ARCHITECTURE.md) - Technical details
   - [Platform Compatibility Guide](PLATFORM_COMPATIBILITY.md) - Known platform issues

---

## Additional Resources

- **Configuration Reference:** See [`README.md`](../README.md) for detailed configuration options
- **Error Handling:** See [`docs/error_handling_audit.md`](error_handling_audit.md) for detailed error documentation
- **Platform Issues:** See [`docs/PLATFORM_COMPATIBILITY.md`](PLATFORM_COMPATIBILITY.md) for known platform-specific issues
- **Known Bugs:** See [`BUGS.md`](../BUGS.md) for current bug tracking

---

**Still having issues?** Please file a bug report on [GitHub Issues](https://github.com/your-org/switchboard/issues) with the diagnostic information gathered from the debugging workflows above.
