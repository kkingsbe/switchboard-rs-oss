# Performance Expectations: Skills Installation

This document defines the performance expectations for the skills installation functionality in switchboard and provides guidance on testing, troubleshooting, and performance optimization.

## Overview

The skills installation process is a critical operation that sets up Kilo Code agents with the necessary skills to perform their tasks. The process involves generating entrypoint scripts for Docker containers and then executing those containers with npx to install the skills.

**Important Distinction:** The performance tests in this document measure the **entrypoint script generation** phase (preparation), which occurs before Docker container creation and npx execution. This is the portion that can be reliably tested in unit tests without requiring Docker or network dependencies.

## Performance Targets

### Primary Performance Thresholds

| Phase | Target Threshold | Measured By Tests | Notes |
|-------|------------------|-------------------|-------|
| **Entrypoint Script Generation** | **< 1 second** for 1-10 skills | ✅ Yes | Unit testable - preparation phase |
| **Full Installation (Docker + npx)** | **15 seconds** | ❌ No | Requires Docker and network - integration test only |

### Specific Scenario Expectations

| Scenario | Expected Duration | Notes |
|----------|------------------|-------|
| **Single skill script generation** | < 500ms | Fastest scenario - minimal processing |
| **5 skills script generation** | < 1 second | Linear scaling expected |
| **10 skills script generation** | < 1 second | Upper bound for script generation |
| **Full installation (Docker + npx)** | 15 seconds | Includes container startup and npx execution |

### What is Being Measured

The performance tests measure **only the entrypoint script generation**, which includes:

- **Configuration parsing**: Reading and validating skill configurations from TOML
- **Entrypoint script template processing**: Generating the Docker entrypoint script
- **Skill manifest aggregation**: Collecting skill metadata for script generation

The tests **do not** measure:
- Docker container creation and startup
- npx package installation
- Network downloads for skill packages
- Actual skill package execution

These later phases require Docker and network availability and cannot be reliably tested in unit tests. The 15-second target for full installation is a **goal** that should be validated through integration testing or manual testing.

## Test Coverage

### Test File

[`tests/skills_install_performance.rs`](../tests/skills_install_performance.rs)

### Test Functions

#### 1. `test_skill_install_script_generation_performance`

**Purpose:** Measures entrypoint script generation time for a single skill installation.

**Expected Result:** Script generation should complete in < 500ms for a single skill.

**Test Scenario:**
- Creates a temporary directory
- Generates an entrypoint script for one skill
- Measures generation time
- Asserts time is under threshold

#### 2. `test_skill_install_script_generation_multiple_skills`

**Purpose:** Measures entrypoint script generation time for multiple skills (5 skills).

**Expected Result:** Script generation should complete in < 1 second for 5 skills.

**Test Scenario:**
- Creates a temporary directory
- Generates entrypoint scripts for 5 skills
- Measures generation time
- Asserts time is under threshold
- Verifies linear scaling behavior

#### 3. `test_skill_install_script_generation_benchmark`

**Purpose:** Benchmarks entrypoint script generation performance across different skill counts (1, 3, 5, 10 skills).

**Expected Result:** All configurations should complete script generation in < 1 second, demonstrating O(n) scaling.

**Test Scenario:**
- Tests script generation for 1, 3, 5, and 10 skills
- Reports timing for each configuration
- Verifies all are under the 1-second threshold
- Provides performance data for analysis

## Running the Tests

### Prerequisites

Ensure you have:
- Rust toolchain installed
- The switchboard project cloned
- Write permissions to create temporary test directories

### Running All Performance Tests

```bash
cargo test --test skills_install_performance -- --nocapture
```

### Running Specific Test Scenarios

```bash
# Test single skill script generation
cargo test skills_install_performance::test_skill_install_script_generation_performance -- --nocapture

# Test multiple skills script generation (5 skills)
cargo test skills_install_performance::test_skill_install_script_generation_multiple_skills -- --nocapture

# Test benchmark across different skill counts
cargo test skills_install_performance::test_skill_install_script_generation_benchmark -- --nocapture
```

### Understanding Test Output

The performance tests will output timing information in milliseconds:

```
running 3 tests
test skills_install_performance::test_skill_install_script_generation_performance ... ok

  Entrypoint script generation (1 skill) completed in 345ms
  Threshold: 1000ms, Status: PASSED
```

Key values to note:
- **Duration**: Actual script generation time in milliseconds
- **Threshold**: Maximum allowed time (1000ms for up to 10 skills)
- **Status**: PASSED or FAILED based on comparison

## Performance Characteristics

### Expected Scalability Behavior

The entrypoint script generation should exhibit **O(n)** complexity where:
- `n` = number of skills to install
- Each skill incurs a roughly constant overhead for:
  - Configuration file parsing
  - Skill metadata extraction
  - Entrypoint script template expansion

**The generation must NOT exhibit O(n²) or worse behavior**, which would indicate implementation issues such as:
- Nested loops over skill collections
- Repeated configuration file reads
- Inefficient string concatenation or template processing
- Unnecessary cloning of data structures

## Factors That May Affect Performance

### 1. Script Generation (Measured by Tests)

**Impact:** Low to Medium

The entrypoint script generation performance is affected by:
- **Number of skills**: More skills = more template processing
- **Configuration file size**: Larger TOML files take longer to parse
- **Skill manifest complexity**: More metadata = more processing
- **Template complexity**: More complex entrypoint templates take longer to expand

**Mitigation:**
- Use efficient string builders and templates
- Cache parsed configurations
- Minimize redundant file I/O
- Use efficient data structures for lookups

### 2. Docker Container Creation (Not Measured by Tests)

**Impact:** High

Docker container creation and startup time depends on:
- **Docker daemon performance**: Speed of container creation API
- **Image availability**: Whether base images are cached locally
- **System resources**: CPU, memory, and disk I/O performance
- **Docker daemon load**: Number of concurrent containers being created

**Mitigation:**
- Ensure Docker daemon is running and responsive
- Use cached base images where possible
- Implement container pooling (future enhancement)
- Set reasonable timeouts for container operations

### 3. npx Package Installation (Not Measured by Tests)

**Impact:** High

npx installation time depends on:
- **Network conditions**: Download speed for skill packages
- **npm registry performance**: Speed of package registry servers
- **Package size**: Larger packages take longer to download
- **Package dependencies**: Number and size of transitive dependencies
- **npm cache status**: Whether packages are already cached locally

**Mitigation:**
- Use npm cache to avoid redundant downloads
- Set reasonable timeouts for network operations
- Implement retry logic for transient network failures
- Consider package deduplication

### 4. Number of Skills

**Impact:** Medium

More skills means:
- More entrypoint script generation (measured, scales linearly)
- More Docker containers (not measured, scales linearly)
- More npx installations (not measured, depends on network)

**Mitigation:**
- Parallelize Docker container creation where possible
- Batch npx installations to reduce overhead
- Implement skill dependency resolution to avoid duplicates

### 5. System Load

**Impact:** Medium

- CPU contention slows script generation and Docker operations
- Memory pressure may affect Docker daemon performance
- I/O contention affects file reading and Docker image operations

**Note:** Performance tests should be run on systems with minimal load for consistency.

## Performance Monitoring in Production

### Recommended Metrics

Track these metrics in production to catch performance regressions early:

1. **Script Generation Time**: Distribution (p50, p95, p99) - measured during preparation phase
2. **Full Installation Time**: Distribution (p50, p95, p99) - measured end-to-end
3. **Skill Count**: Correlation with generation and installation times
4. **Error Rate**: How often installation-related errors occur
5. **Docker Operation Time**: Time spent in Docker container creation and startup
6. **npx Execution Time**: Time spent in npx package installation

### Metrics Field

The 15-second target for full installation is tracked in the [`skills_install_time_seconds`](../src/metrics.rs) metric field, which captures the end-to-end installation time from preparation to completion.

### Benchmarking Script

Create a benchmark script to periodically test performance:

```bash
#!/bin/bash
# benchmark_skills_install.sh

echo "Running skills installation performance benchmark..."

# Measure script generation (unit test)
echo "Script generation (unit test):"
cargo test --test skills_install_performance -- --nocapture

# Measure full installation (requires Docker and network)
echo "Full installation (Docker + npx):"
time switchboard skills install test-skill

echo "Benchmark completed"
```

Run this script regularly and log results for trend analysis.

## Integration Testing Note

Full integration testing of skills installation requires:

- **Docker daemon**: Running and accessible
- **npx**: Available in the system PATH
- **Network access**: For downloading skill packages
- **npm registry**: Accessible for package resolution

Unit tests in [`tests/skills_install_performance.rs`](../tests/skills_install_performance.rs) intentionally avoid these dependencies by testing only the entrypoint script generation phase. This allows for fast, reliable, and repeatable tests.

For end-to-end validation, run manual integration tests or create an integration test suite that includes Docker and network dependencies.

## Troubleshooting Failed Performance Tests

If performance tests fail, follow this systematic approach:

### Step 1: Identify the Failed Scenario

Determine which specific scenario is failing:
- Single skill script generation (< 500ms expected)
- Multiple skills script generation - 5 skills (< 1000ms expected)
- Benchmark test - various skill counts (< 1000ms each expected)

### Step 2: Reproduce the Failure

Run the failing test in isolation with verbose output:
```bash
cargo test <test_name> -- --nocapture --show-output
```

### Step 3: Analyze System Conditions

Check for environmental factors:
```bash
# Check system load
top

# Check disk I/O (Linux)
iostat -x 1

# Check available memory
free -h
```

### Step 4: Profile the Code

Use Rust's profiling tools to identify bottlenecks:

#### Using `cargo-flamegraph`:
```bash
cargo install flamegraph
cargo flamegraph --test skills_install_performance <test_name>
```

#### Using built-in timing:
Add timing logs around key operations:
```rust
let start = std::time::Instant::now();
// ... operation ...
println!("Operation took: {:?}", start.elapsed());
```

### Step 5: Common Performance Bottlenecks

Look for these common issues:

#### 1. Excessive File I/O
- **Symptom**: Many `read_to_string` or similar calls
- **Fix**: Cache file contents, use buffered readers
- **Check**: Use `strace` (Linux) to see file access patterns

#### 2. Inefficient Data Structures
- **Symptom**: Nested loops over skill collections
- **Fix**: Use `HashMap` or `HashSet` for O(1) lookups
- **Check**: Review algorithm complexity

#### 3. Unnecessary Cloning
- **Symptom**: Frequent `.clone()` calls on large structs
- **Fix**: Use references (`&T`) or `Cow` types
- **Check**: Audit `.clone()` usage with `cargo clippy`

#### 4. Inefficient String Processing
- **Symptom**: String concatenation in loops
- **Fix**: Use `String::with_capacity` or `format!` macro
- **Check**: Review string manipulation code

#### 5. Repeated Configuration Parsing
- **Symptom**: Configuration files parsed multiple times
- **Fix**: Cache parsed configurations for the duration of the operation
- **Check**: Count parsing invocations

### Step 6: Verify Fixes

After making changes:
1. Run the failing test again
2. Verify it now passes
3. Run all performance tests to ensure no regressions
4. Run the full test suite to verify functionality isn't broken

## Performance Optimization Guidelines

### DO

- Use efficient data structures (`HashMap`, `HashSet`)
- Minimize file I/O operations
- Cache frequently-accessed data (configurations, parsed manifests)
- Use references instead of clones where possible
- Implement proper error handling that doesn't impact the happy path
- Profile before optimizing
- Consider parallelization for independent operations

### DO NOT

- Prematurely optimize without profiling data
- Sacrifice code correctness for micro-optimizations
- Use complex algorithms without understanding the trade-offs
- Ignore error handling in performance-critical paths
- Optimize code that isn't in the hot path

## References

- Test file: [`tests/skills_install_performance.rs`](../tests/skills_install_performance.rs)
- Implementation: [`src/container/mod.rs`](../src/container/mod.rs) (path may vary)
- Skills documentation: [`docs/skills-feature.md`](../docs/skills-feature.md)
- Related performance documentation: [`docs/PERFORMANCE_SKILLS_LIST.md`](./PERFORMANCE_SKILLS_LIST.md)

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-20 | Initial documentation of skills installation performance expectations |

---

**Last Updated:** 2026-02-20  
**Maintainer:** Development Team  
**Related Sprint:** Sprint 4, Task 2
