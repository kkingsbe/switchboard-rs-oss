# Troubleshooting Guide

This guide covers common issues you may encounter when using Switchboard and how to resolve them.

## Docker Issues

### Docker Daemon Not Running

**Symptom:**
```
Error: Docker daemon is not running
```

**Cause:** Docker Desktop or Docker daemon is not started.

**Solution:**

1. **macOS:** Open Docker Desktop application or run:
   ```bash
   open -a Docker
   ```

2. **Linux:** Start the Docker service:
   ```bash
   sudo systemctl start docker
   sudo systemctl status docker  # Verify it's running
   ```

3. **Windows:** Ensure Docker Desktop is running (check system tray icon).

---

### Docker Permission Denied

**Symptom:**
```
permission denied while trying to connect to the Docker daemon socket
```

**Cause:** Your user doesn't have permission to access Docker.

**Solution:**

1. Add your user to the docker group:
   ```bash
   sudo usermod -aG docker $USER
   ```

2. Log out and back in, or run:
   ```bash
   newgrp docker
   ```

3. Alternatively, run Switchboard with sudo (not recommended for production).

---

### Docker Out of Disk Space

**Symptom:**
```
Error: No space left on device
```

**Cause:** Docker's disk space is exhausted.

**Solution:**

1. Clean up unused Docker resources:
   ```bash
   docker system prune -a
   docker volume prune
   ```

2. Remove unused images:
   ```bash
   docker rmi $(docker images -q -f dangling=true)
   ```

3. Check Docker desktop settings to increase disk allocation.

---

## Configuration Issues

### Config File Not Found

**Symptom:**
```
Error: config file not found: switchboard.toml
```

**Cause:** No configuration file in the current directory or default locations.

**Solution:**

1. Create a configuration file:
   ```bash
   cp switchboard.sample.toml switchboard.toml
   ```

2. Or specify a custom config path:
   ```bash
   switchboard --config /path/to/config.toml run
   ```

---

### Config Validation Errors

**Symptom:**
```
Error: Invalid configuration at line X
```

**Cause:** Syntax error or invalid value in TOML configuration.

**Solution:**

1. Validate your configuration:
   ```bash
   switchboard validate
   ```

2. Common issues:
   - Invalid cron syntax (use standard cron format: `* * * * *`)
   - Missing required fields
   - Invalid TOML data types
   - Malformed list/array syntax

3. Example valid cron schedule:
   ```toml
   [[schedule]]
   name = "daily-report"
   cron = "0 9 * * *"  # Run at 9:00 AM daily
   ```

---

### Invalid Cron Expression

**Symptom:**
```
Error: invalid cron expression
```

**Cause:** The cron expression doesn't follow standard format.

**Solution:** Use standard 5-field cron format:

| Field | Allowed Values |
|-------|----------------|
| Minute | 0-59 |
| Hour | 0-23 |
| Day of Month | 1-31 |
| Month | 1-12 |
| Day of Week | 0-6 (0 = Sunday) |

Examples:
- `* * * * *` - Every minute
- `0 * * * *` - Every hour at minute 0
- `0 9 * * 1-5` - Weekdays at 9:00 AM
- `0 0 1 * *` - First day of every month at midnight

---

## Skill Installation Issues

### Skill Installation Failures

**Symptom:**
```
Error: failed to install skill: [skill-name]
```

**Cause:** Network issues, invalid skill definition, or Docker problems.

**Solution:**

1. Check network connectivity:
   ```bash
   ping registry.example.com
   ```

2. Verify the skill definition in your config:
   ```toml
   [[skill]]
   name = "my-skill"
   source = "docker-image:tag"  # or local path
   ```

3. Check Docker can pull images:
   ```bash
   docker pull <image-name>
   ```

4. View detailed logs:
   ```bash
   switchboard logs --skill <skill-name>
   ```

---

### Network Connectivity Issues

**Symptom:**
```
Error: connection timeout
Error: could not resolve host
```

**Cause:** No internet connection or DNS issues.

**Solution:**

1. Check your internet connection:
   ```bash
   ping google.com
   ```

2. Verify DNS resolution:
   ```bash
   nslookup registry.example.com
   ```

3. Check proxy settings if behind a corporate firewall:
   ```bash
   export HTTP_PROXY=http://proxy:8080
   export HTTPS_PROXY=http://proxy:8080
   ```

4. For Docker, configure proxy in Docker Desktop settings.

---

### Container Skill Install Failure

**Symptom:**
```
Error: failed to install container skill
```

**Cause:** The skill's container image failed to build or run.

**Solution:**

1. Verify the skill definition:
   ```toml
   [[skill]]
   name = "my-skill"
   source = "docker"
   image = "skill-image:latest"
   command = ["python", "main.py"]
   ```

2. Check Docker can build/run the image manually:
   ```bash
   docker run skill-image:latest
   ```

3. Check skill logs for specific errors:
   ```bash
   switchboard logs --skill my-skill
   ```

---

## Discord Integration Issues

### Discord Bot Won't Start

**Symptom:**
```
Error: Discord bot failed to connect
```

**Cause:** Invalid bot token or missing permissions.

**Solution:**

1. Verify your bot token in `.env`:
   ```
   DISCORD_BOT_TOKEN=your_token_here
   ```

2. Check the bot has required intents:
   - `GUILD_MESSAGES`
   - `MESSAGE_CONTENT`

3. Re-invite the bot with correct permissions:
   - Send Messages
   - Read Message History
   - Use Application Commands

---

### Discord Message Response Not Working

**Symptom:** Bot doesn't respond to messages.

**Cause:** Missing configuration or wrong channel permissions.

**Solution:**

1. Verify Discord config:
   ```toml
   [discord]
   enabled = true
   ```

2. Check the bot has permissions in the channel:
   - Send Messages
   - Read Message History

3. Verify the bot was invited with correct scopes (`bot`, `applications.commands`).

---

## General Issues

### High Memory Usage

**Symptom:** Switchboard is using excessive memory.

**Solution:**

1. Reduce concurrent skill executions in config:
   ```toml
   [scheduler]
   max_concurrent = 2
   ```

2. Limit skill resources:
   ```toml
   [[skill]]
   name = "heavy-skill"
   memory_limit = "512m"  # Limit to 512MB
   ```

---

### Logs Not Appearing

**Symptom:** No logs are being written.

**Cause:** Logging not configured or log level too high.

**Solution:**

1. Enable logging in config:
   ```toml
   [logging]
   level = "debug"
   file = "./logs/switchboard.log"
   ```

2. Check log file permissions:
   ```bash
   ls -la logs/
   ```

---

## Getting More Help

If you're still experiencing issues:

1. Run with verbose logging:
   ```bash
   RUST_LOG=debug switchboard run
   ```

2. Check the logs directory:
   ```bash
   ls -la logs/
   ```

3. Search existing issues on GitHub
4. Ask on the Discord community
