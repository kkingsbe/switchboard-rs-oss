# Reliability Tests Documentation

This document describes the reliability and stress tests for skills installation in the Switchboard project.

## Overview

The reliability tests verify the stability and correctness of skill installation operations under various stress conditions. These tests focus on the script generation phase, which is the preparation step before actual Docker container-based skill installation.

## Test File Location

- **Test implementation**: `tests/skills_reliability.rs`
- **Common test utilities**: `tests/performance_common.rs`

## What Reliability Tests Verify

### 1. Sequential Multiple Skills Installation (`test_sequential_multiple_skills_installation`)

Tests installing 10 skills in sequence and verifies:
- Each script generation completes without error
- Each generated script is valid and non-empty
- No resource degradation occurs across multiple sequential calls

**Test approach**: Generates entrypoint scripts for 10 different skills one after another, checking each succeeds.

### 2. Same Skill Reinstall Multiple Times (`test_same_skill_reinstall_multiple_times`)

Tests idempotency by installing the same skill 3 times:
- Each reinstall succeeds without errors
- No duplicate entries or corruption occurs
- Script output is consistent across reinstalls

**Test approach**: Generates entrypoint scripts for the same skill repeatedly and verifies consistency.

### 3. Concurrent Installation Simulation (`test_concurrent_installation_simulation`)

Simulates concurrent installations by generating multiple scripts in rapid succession:
- Tests that `generate_entrypoint_script` can be called multiple times safely
- Documents current limitation: the function is a pure function with no shared state
- Notes that actual skill installation requires coordination at the application level

**Test approach**: Generates scripts for 3 different skill sets simultaneously.

### 4. Resource Leak Detection (`test_resource_leak_detection_file_handles`)

Verifies no resource leaks occur during repeated operations:
- Memory usage remains stable (measured via execution time)
- Each call completes successfully
- No errors occur during 20 repeated operations
- Checks for time degradation between first and last iterations

**Test approach**: Generates scripts 20 times and compares execution time averages.

### 5. Large Skill Count Reliability (`test_large_skill_count_reliability`)

Stress tests script generation with a large number of skills (50):
- Verifies all skills are included in the generated script
- Checks completion time remains reasonable (< 1 second)

**Test approach**: Generates a single script containing 50 skills.

## Running the Tests

### Run all reliability tests

```bash
cargo test --test skills_reliability
```

### Run a specific reliability test

```bash
cargo test --test skills_reliability test_sequential_multiple_skills_installation
cargo test --test skills_reliability test_same_skill_reinstall_multiple_times
cargo test --test skills_reliability test_concurrent_installation_simulation
cargo test --test skills_reliability test_resource_leak_detection_file_handles
cargo test --test skills_reliability test_large_skill_count_reliability
```

### Run with output

```bash
cargo test --test skills_reliability -- --nocapture
```

## Test Design Philosophy

These tests follow the same pattern as the existing performance tests (`tests/skills_install_performance.rs`):

1. **Focus on script generation**: Since actual skill installation requires Docker, npx, and network access (which may not be available in CI environments), the tests focus on the script generation phase.

2. **Mock-ready structure**: The tests are designed to be easily adaptable for mock-based testing if needed in the future.

3. **Clear documentation**: Each test includes detailed documentation explaining what it verifies and why.

## Limitations

- **No actual Docker testing**: These tests don't verify actual Docker container creation or skill installation inside containers.
- **No network testing**: Network-dependent operations (downloading skills from npm/GitHub) are not tested.
- **No file system testing**: Actual skill installation would write to `.kilocode/skills/`, which is not tested here.

## Related Tests

- `tests/skills_install_performance.rs` - Performance benchmarks for script generation
- `tests/skills_list_performance.rs` - Performance tests for skill listing
- `tests/skill_install_error_handling.rs` - Error handling tests
- `tests/skills_network_failure.rs` - Network failure simulation tests

## Performance Targets

The reliability tests verify that:

- Script generation for 10 skills completes successfully
- Script generation for 50 skills completes in under 1 second
- No significant time degradation occurs over 20+ iterations
- Scripts are idempotent and reproducible

These targets align with the 15-second performance target for full skill installation (which includes Docker startup + npx execution + skill download).
