# Scheduler Up/Down API Implementation Plan

## Overview

This plan outlines the implementation of API endpoints that provide the equivalent functionality to `switchboard up` and `switchboard down` CLI commands. Currently, the REST API does not expose these endpoints (intentionally, per the original REST API plan), but they are needed for full programmatic control of the scheduler.

---

## Table of Contents

1. [Current State Analysis](#1-current-state-analysis)
2. [Proposed API Endpoints](#2-proposed-api-endpoints)
3. [Implementation Approach](#3-implementation-approach)
4. [API Handler Implementation](#4-api-handler-implementation)
5. [Router Updates](#5-router-updates)
6. [Module Documentation Updates](#6-module-documentation-updates)
7. [User Documentation Updates](#7-user-documentation-updates)
8. [Testing Strategy](#8-testing-strategy)

---

## 1. Current State Analysis

### CLI Commands

| Command | File | Functionality |
|---------|------|---------------|
| `switchboard up` | [`src/cli/commands/up.rs`](src/cli/commands/up.rs) | Starts scheduler, registers agents, creates PID file at `.switchboard/scheduler.pid` |
| `switchboard restart` | [`src/cli/commands/restart.rs`](src/cli/commands/restart.rs) | Stops scheduler (sends SIGTERM via `kill`) and restarts |
| `switchboard down` | [`src/cli/mod.rs:1003`](src/cli/mod.rs) | Stops scheduler and containers |

### Current API Endpoints

| Endpoint | Status | Notes |
|----------|--------|-------|
| `POST /api/v1/gateway/up` | Placeholder | For Discord gateway, not scheduler |
| `POST /api/v1/gateway/down` | Placeholder | For Discord gateway, not scheduler |
| `GET /api/v1/status` | Partial | Returns `running: false` (TODO) |
| `POST /api/v1/shutdown` | Placeholder | For API server shutdown, not scheduler |

### Architecture Notes

- Scheduler runs as a separate process with PID stored in `.switchboard/scheduler.pid`
- API server runs in its own process (separate from scheduler)
- The API server needs to spawn/manage the scheduler process to implement up/down

---

## 2. Proposed API Endpoints

### New Endpoints

| CLI Equivalent | HTTP Method | Endpoint | Description |
|----------------|-------------|----------|-------------|
| `switchboard up` | POST | `/api/v1/scheduler/up` | Start the scheduler |
| `switchboard down` | POST | `/api/v1/scheduler/down` | Stop the scheduler |
| `switchboard restart` | POST | `/api/v1/scheduler/restart` | Restart the scheduler |
| `switchboard status` | GET | `/api/v1/scheduler/status` | Get scheduler status (enhanced) |

### Request/Response Formats

#### POST /api/v1/scheduler/up

**Request Body (optional):**
```json
{
  "detach": false
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "started": true,
    "pid": 12345,
    "instance_id": "switchboard-default"
  },
  "message": "Scheduler started successfully"
}
```

#### POST /api/v1/scheduler/down

**Request Body (optional):**
```json
{
  "force": false,
  "timeout": 30
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "stopped": true,
    "previous_pid": 12345
  },
  "message": "Scheduler stopped successfully"
}
```

#### GET /api/v1/scheduler/status

**Response:**
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
  }
}
```

---

## 3. Implementation Approach

### Process Management Strategy

The API server will manage the scheduler as a child process using the same patterns as the CLI:

1. **Start (up)**: Spawn scheduler as child process, write PID to file
2. **Stop (down)**: Read PID from file, send SIGTERM, clean up
3. **Status**: Check if process with PID is running

### Key Design Decisions

1. **Separate Process**: Scheduler runs in separate process (not in same event loop as API)
2. **PID File**: Use existing `.switchboard/scheduler.pid` mechanism for compatibility
3. **Detached Mode**: Default behavior matches CLI `switchboard up` (blocking)
4. **Error Handling**: Return appropriate HTTP status codes for common errors

### Implementation Location

New file: `src/api/handlers/scheduler.rs`

This follows the existing pattern where each resource has its own handler file.

---

## 4. API Handler Implementation

### New File: src/api/handlers/scheduler.rs

```rust
//! Scheduler API handlers.
//!
//! This module provides HTTP handlers for scheduler management endpoints.

use crate::api::error::{ApiError, ApiResult};
use crate::api::state::ApiState;
use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Scheduler start request.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SchedulerStartRequest {
    /// Run scheduler in detached mode.
    #[serde(default)]
    pub detach: bool,
}

/// Scheduler start response.
#[derive(Serialize, Deserialize, Debug)]
pub struct SchedulerStartResponse {
    pub started: bool,
    pub pid: Option<u32>,
    pub instance_id: String,
    pub message: String,
}

/// Scheduler stop request.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SchedulerStopRequest {
    /// Force kill the scheduler.
    #[serde(default)]
    pub force: bool,
    /// Timeout in seconds before force kill.
    #[serde(default = "default_timeout")]
    pub timeout: u32,
}

fn default_timeout() -> u32 {
    30
}

/// Scheduler stop response.
#[derive(Serialize, Deserialize, Debug)]
pub struct SchedulerStopResponse {
    pub stopped: bool,
    pub previous_pid: Option<u32>,
    pub message: String,
}

/// Scheduler status response.
#[derive(Serialize, Deserialize, Debug)]
pub struct SchedulerStatusResponse {
    pub running: bool,
    pub pid: Option<u32>,
    pub instance_id: String,
    pub uptime_seconds: Option<u64>,
    pub agents_registered: Option<usize>,
    pub started_at: Option<String>,
}

// ============================================================================
// Handlers
// ============================================================================

/// Start the scheduler.
///
/// Equivalent to `switchboard up` CLI command.
pub async fn scheduler_up(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<SchedulerStartRequest>,
) -> ApiResult<Json<ApiResponse<SchedulerStartResponse>>> {
    // 1. Check if scheduler is already running
    // 2. Load switchboard config if not already loaded
    // 3. Spawn scheduler process (similar to CLI run_up)
    // 4. Write PID file
    // 5. Return response with PID
}

/// Stop the scheduler.
///
/// Equivalent to `switchboard down` CLI command.
pub async fn scheduler_down(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<SchedulerStopRequest>,
) -> ApiResult<Json<ApiResponse<SchedulerStopResponse>>> {
    // 1. Read PID from file
    // 2. Check if process is running
    // 3. Send SIGTERM (or SIGKILL if force)
    // 4. Wait for graceful shutdown
    // 5. Clean up PID file
    // 6. Return response
}

/// Get scheduler status.
///
/// Returns current scheduler status.
pub async fn scheduler_status(
    State(state): State<Arc<ApiState>>,
) -> ApiResult<Json<ApiResponse<SchedulerStatusResponse>>> {
    // 1. Read PID from file
    // 2. Check if process is running
    // 3. Get uptime if running
    // 4. Return status
}

/// Restart the scheduler.
///
/// Equivalent to `switchboard restart` CLI command.
pub async fn scheduler_restart(
    State(state): State<Arc<ApiState>>,
    Json(request): Json<SchedulerStartRequest>,
) -> ApiResult<Json<ApiResponse<SchedulerStartResponse>>> {
    // 1. Stop scheduler (call scheduler_down logic)
    // 2. Start scheduler (call scheduler_up logic)
    // 3. Return response
}
```

---

## 5. Router Updates

### Update src/api/router.rs

Add new routes:

```rust
// Scheduler endpoints
.route("/api/v1/scheduler/up", post(scheduler_up))
.route("/api/v1/scheduler/down", post(scheduler_down))
.route("/api/v1/scheduler/status", get(scheduler_status))
.route("/api/v1/scheduler/restart", post(scheduler_restart))
```

### Update src/api/handlers/mod.rs

Export new handlers:

```rust
pub use crate::api::handlers::scheduler::{
    scheduler_up,
    scheduler_down,
    scheduler_status,
    scheduler_restart,
};
```

---

## 6. Module Documentation Updates

### Update src/api/mod.rs

Add scheduler endpoints to the module documentation:

```rust
//! - `scheduler` - Scheduler management at `/api/v1/scheduler/*`
//!   - POST /api/v1/scheduler/up - Start scheduler
//!   - POST /api/v1/scheduler/down - Stop scheduler
//!   - POST /api/v1/scheduler/restart - Restart scheduler
//!   - GET /api/v1/scheduler/status - Get scheduler status
```

---

## 7. User Documentation Updates

### 7.1 Update docs/cli-reference.md

Add note about API equivalents:

```markdown
### up

Build the agent Docker image (if needed) and start the scheduler...

**API Equivalent:**
```bash
curl -X POST http://localhost:18500/api/v1/scheduler/up
```
```

### 7.2 Create docs/api-reference.md (or update existing)

Add new section for scheduler endpoints:

```markdown
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

**Response:**
```json
{
  "success": true,
  "data": {
    "started": true,
    "pid": 12345,
    "instance_id": "switchboard-default"
  }
}
```

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

### Get Scheduler Status

**Endpoint:** `GET /api/v1/scheduler/status`

Get current scheduler status.

### Restart Scheduler

**Endpoint:** `POST /api/v1/scheduler/restart`

Restart the scheduler.
```

### 7.3 Update docs/INDEX.md

Add API reference to the index.

---

## 8. Testing Strategy

### Unit Tests

1. Test handler request/response serialization
2. Test PID file reading/writing
3. Test process status checking

### Integration Tests

1. Test scheduler start via API
2. Test scheduler stop via API
3. Test scheduler status endpoint
4. Test scheduler restart endpoint
5. Test error cases (already running, not running, etc.)

### Test Locations

- Unit tests in `src/api/handlers/scheduler.rs`
- Integration tests in `tests/` directory (new file: `tests/scheduler_api.rs`)

---

## Implementation Order

1. **Create scheduler handlers** (`src/api/handlers/scheduler.rs`)
2. **Update handlers mod.rs** to export new handlers
3. **Update router.rs** to add new routes
4. **Update api/mod.rs** documentation
5. **Create/update user documentation**
6. **Add tests**

---

## Summary

This implementation adds the missing scheduler management endpoints to the REST API, providing full parity with the CLI commands:

- `POST /api/v1/scheduler/up` - Equivalent to `switchboard up`
- `POST /api/v1/scheduler/down` - Equivalent to `switchboard down`
- `POST /api/v1/scheduler/restart` - Equivalent to `switchboard restart`
- `GET /api/v1/scheduler/status` - Enhanced status endpoint

The implementation follows existing patterns in the codebase and maintains compatibility with the CLI's PID file mechanism.
