# Silent Timeout Test Coverage Plan

## Goal
Reach 80% test coverage for the silent timeout feature.

## Current Coverage
- **parse_silent_timeout()**: 0% - No tests
- **LogTimestampTracker**: 0% - No tests
- **effective_silent_timeout()**: 0% - No tests
- **spawn_silent_timeout_monitor()**: 0% - No tests (async, requires mocking)

## Target Functions for Testing

### 1. parse_silent_timeout() - src/docker/run/run.rs:525
**Lines to cover**: 525-560 (~36 lines)

| Test Case | Input | Expected Output |
|-----------|-------|-----------------|
| Disabled with "0" | "0" | Ok(None) |
| Disabled with "0s" | "0s" | Ok(None) |
| Seconds | "30s" | Ok(Some(30 seconds)) |
| Minutes | "5m" | Ok(Some(300 seconds)) |
| Hours | "1h" | Ok(Some(3600 seconds)) |
| Hours large | "2h" | Ok(Some(7200 seconds)) |
| Whitespace handling | " 5m " | Ok(Some(300 seconds)) |
| Invalid unit | "5x" | Err(DockerError) |
| Invalid value | "abc" | Err(DockerError) |
| Empty string | "" | Err(DockerError) |
| Zero value with unit | "0s" | Ok(None) |

### 2. LogTimestampTracker - src/docker/run/streams.rs:19
**Lines to cover**: 24-52 (~29 lines)

| Test Case | Method | Scenario |
|-----------|--------|----------|
| New tracker initialized | new() | Should set last_log_time to current time |
| Update resets timestamp | update() | After update, seconds_since_last_log should be ~0 |
| Seconds since last log | seconds_since_last_log() | Immediately after creation should be ~0 |
| Timeout NOT exceeded | is_silent_timeout_exceeded() | Short timeout, recent log = false |
| Timeout exceeded | is_silent_timeout_exceeded() | Long timeout, old log = true |
| Default implementation | Default::default() | Should be equivalent to new() |

### 3. effective_silent_timeout() - src/config/mod.rs:987
**Lines to cover**: 987-1002 (~16 lines)

| Test Case | Agent Setting | Global Setting | Expected |
|-----------|---------------|----------------|----------|
| Agent override | Some("10m") | Some("5m") | "10m" |
| Global fallback | None | Some("5m") | "5m" |
| Default fallback | None | None | "5m" |
| Agent "0" (disabled) | Some("0") | Some("5m") | "0" |
| Global "0" (disabled) | None | Some("0") | "0" |

### 4. spawn_silent_timeout_monitor() - src/docker/run/run.rs:580
**Lines to cover**: 580-657 (~78 lines)
- Requires async testing with mocks
- Consider testing with MockDockerClient

| Test Case | Scenario |
|-----------|----------|
| Cancel before timeout | cancel_flag set, no kill |
| Timeout triggers kill | exceeds timeout, kills container |
| Kill failure handled | kill_container returns error |
| Logger write on timeout | logger receives timeout message |

## Test File Locations

Option A: Add to existing test modules
- `src/docker/run/run.rs` - Add #[cfg(test)] module
- `src/docker/run/streams.rs` - Add #[cfg(test)] module  
- `src/config/mod.rs` - Add #[cfg(test)] module

Option B: Add to `tests/` directory
- `tests/silent_timeout_tests.rs` - New file

## Implementation Approach

1. **Week 1**: Unit tests for parse_silent_timeout and LogTimestampTracker
2. **Week 2**: Unit tests for effective_silent_timeout  
3. **Week 3**: Async tests for spawn_silent_timeout_monitor (requires mock setup)
4. **Week 4**: Integration test and coverage verification

## Coverage Targets

| Module | Current | Target |
|--------|---------|--------|
| parse_silent_timeout | 0% | 100% |
| LogTimestampTracker | 0% | 100% |
| effective_silent_timeout | 0% | 100% |
| spawn_silent_timeout_monitor | 0% | 60% (async complexity) |
| **Overall** | **0%** | **80%** |

## Notes
- Use `#[tokio::test]` for async tests
- Mock `DockerClientTrait` for spawn_silent_timeout_monitor tests
- Use `Duration` mocking or frozen time for LogTimestampTracker tests
- Consider using `time` crate or similar for deterministic testing
