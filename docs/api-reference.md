# API Reference

Complete reference for the Switchboard REST API.

> **Last updated:** 2026-03-07

## Base URL

```
http://localhost:8080/api/v1
```

## Response Format

All responses follow a consistent JSON structure:

```json
{
  "success": true,
  "data": { ... },
  "message": null
}
```

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether the request succeeded |
| `data` | object | Response payload (null on error) |
| `message` | string | Error message if success is false |

## Error Responses

| Status Code | Description |
|-------------|-------------|
| 400 | Bad Request - Invalid input |
| 404 | Not Found - Resource not found |
| 409 | Conflict - Resource already exists |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

---

## Scheduler Management

### Start Scheduler

**Endpoint:** `POST /api/v1/scheduler/up`

Start the scheduler to run all configured agents on their cron schedules.

**Request:**
```json
{
  "detach": false
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `detach` | boolean | No | Run scheduler in detached mode (background). Default: false |

> **Note:** Foreground mode (`detach: false`) is not supported via API. Use `detach: true` or the CLI command.

**Response (200):**
```json
{
  "success": true,
  "data": {
    "started": true,
    "pid": 12345,
    "instance_id": "switchboard-default",
    "message": "Scheduler started successfully with PID: 12345"
  },
  "message": null
}
```

**Response (409 - Already running):**
```json
{
  "success": true,
  "data": {
    "started": false,
    "pid": 12345,
    "instance_id": "switchboard-default",
    "message": "Scheduler is already running with PID: 12345"
  },
  "message": null
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/v1/scheduler/up \
  -H "Content-Type: application/json" \
  -d '{"detach": true}'
```

---

### Stop Scheduler

**Endpoint:** `POST /api/v1/scheduler/down`

Stop the scheduler and any running agent containers.

**Request:**
```json
{
  "force": false,
  "timeout": 30
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `force` | boolean | No | Force kill the scheduler. Default: false |
| `timeout` | number | No | Timeout in seconds before force kill. Default: 30 |

**Response (200):**
```json
{
  "success": true,
  "data": {
    "stopped": true,
    "previous_pid": 12345,
    "message": "Scheduler stopped successfully (PID: 12345)"
  },
  "message": null
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/v1/scheduler/down \
  -H "Content-Type: application/json" \
  -d '{"force": false, "timeout": 30}'
```

---

### Get Scheduler Status

**Endpoint:** `GET /api/v1/scheduler/status`

Get current scheduler status.

**Response (200):**
```json
{
  "success": true,
  "data": {
    "running": true,
    "pid": 12345,
    "instance_id": "switchboard-default",
    "uptime_seconds": 3600,
    "agents_registered": 5,
    "started_at": "2026-03-07T10:00:00Z"
  },
  "message": null
}
```

**Response (not running):**
```json
{
  "success": true,
  "data": {
    "running": false,
    "pid": null,
    "instance_id": "switchboard-default",
    "uptime_seconds": null,
    "agents_registered": null,
    "started_at": null
  },
  "message": null
}
```

**Example:**
```bash
curl http://localhost:8080/api/v1/scheduler/status
```

---

### Restart Scheduler

**Endpoint:** `POST /api/v1/scheduler/restart`

Restart the scheduler (stop then start).

**Request:**
```json
{
  "detach": true,
  "stop_timeout": 30
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `detach` | boolean | No | Run scheduler in detached mode after restart. Default: true |
| `stop_timeout` | number | No | Timeout in seconds before force kill during stop. Default: 30 |

**Response (200):**
```json
{
  "success": true,
  "data": {
    "restarted": true,
    "previous_pid": 12345,
    "new_pid": 12346,
    "instance_id": "switchboard-default",
    "message": "Scheduler restarted successfully (previous PID: 12345, new PID: 12346)"
  },
  "message": null
}
```

**Example:**
```bash
curl -X POST http://localhost:8080/api/v1/scheduler/restart \
  -H "Content-Type: application/json" \
  -d '{"detach": true, "stop_timeout": 30}'
```

---

## Skills Management

### List Available Skills

**Endpoint:** `GET /api/v1/skills`

List all skills available from the registry.

**Response:**
```json
{
  "success": true,
  "data": {
    "skills": [
      {
        "name": "rust-engineer",
        "description": "Rust engineering skill",
        "version": "1.0.0"
      }
    ]
  }
}
```

### Install Skill

**Endpoint:** `POST /api/v1/skills`

Install a skill from the registry or a remote source.

**Request:**
```json
{
  "source": "github:owner/skill-name"
}
```

### List Installed Skills

**Endpoint:** `GET /api/v1/skills/installed`

### Update Skill

**Endpoint:** `PUT /api/v1/skills/:skill_name`

### Remove Skill

**Endpoint:** `DELETE /api/v1/skills/:skill_name`

---

## Workflows Management

### List Available Workflows

**Endpoint:** `GET /api/v1/workflows`

### Install Workflow

**Endpoint:** `POST /api/v1/workflows`

### List Installed Workflows

**Endpoint:** `GET /api/v1/workflows/installed`

### Update Workflow

**Endpoint:** `PUT /api/v1/workflows/:workflow_name`

### Remove Workflow

**Endpoint:** `DELETE /api/v1/workflows/:workflow_name`

### Validate Workflow

**Endpoint:** `POST /api/v1/workflows/validate`

### Apply Workflow

**Endpoint:** `POST /api/v1/workflows/apply`

---

## Gateway Management

> Requires `gateway` feature.

### Start Gateway

**Endpoint:** `POST /api/v1/gateway/up`

### Get Gateway Status

**Endpoint:** `GET /api/v1/gateway/status`

### Stop Gateway

**Endpoint:** `POST /api/v1/gateway/down`

---

## Health & Status

### Health Check

**Endpoint:** `GET /health`

### Configuration Validation

**Endpoint:** `POST /api/v1/validate`

### Shutdown

**Endpoint:** `POST /api/v1/shutdown`

---

## See Also

- [CLI Reference](cli-reference.md) - CLI command documentation
- [Configuration](configuration.md) - Configuration file format
