# Performance Testing Methodology

This document describes the performance testing methodology for switchboard, including how to run tests, timing thresholds, factors affecting performance, and CI/CD integration for regression detection.

## Overview

The switchboard project includes a comprehensive performance testing infrastructure designed to ensure consistent and acceptable performance for core CLI operations. The tests are located in the `tests/` directory and focus on measurable, repeatable operations.

## Test Infrastructure

### Core Modules

| Module | Purpose |
|--------|---------|
| [`tests/performance_common.rs`](../tests/performance_common.rs) | Shared utilities for performance testing |
| [`tests/skills_list_performance.rs`](../tests/skills_list_performance.rs) | Skills list command performance tests |
| [`tests/skills_install_performance.rs`](../tests/skills_install_performance.rs) | Skills installation performance tests |
| [`tests/skills_install_time_metrics.rs`](../tests/skills_install_time_metrics.rs) | Metrics validation tests |

### Performance Common Infrastructure

The [`performance_common.rs`](../tests/performance_common.rs) module provides:

- **`BenchmarkResult`** - Struct to capture timing and throughput metrics
- **`PerformanceThreshold`** - Struct to define acceptable performance bounds
- **`measure()`** - Async function benchmark with warmup iteration
- **`measure_with_throughput()`** - Benchmark with operations-per-second calculation
- **`assert_performance_threshold()`** - Validates results against thresholds
- **`print_benchmark_summary()`** - Formatted output for test results
- **`create_temp_dir()`** - Test isolation with automatic cleanup
- **`set_env_var()` / `restore_env_var()`** - Environment variable management

## Running Performance Tests

### Prerequisites

- Rust toolchain installed (latest stable recommended)
- switchboard project cloned
- Write permissions for temporary test directories

### Run All Performance Tests

```bash
# Run all performance tests with output
cargo test --test skills_list_performance -- --nocapture
cargo test --test skills_install_performance -- --nocapture

# Or run all tests including performance
cargo test performance -- --nocapture
```

### Run Specific Test Files

```bash
# Skills list performance tests
cargo test --test skills_list_performance -- --nocapture

# Skills install performance tests
cargo test --test skills_install_performance -- --nocapture

# Metrics validation tests
cargo test --test skills_install_time_metrics -- --nocapture
```

### Run Individual Tests

```bash
# Skills list tests
cargo test skills_list_performance::test_skills_list_empty_directory_performance -- --nocapture
cargo test skills_list_performance::test_skills_list_multiple_skills_mock_performance -- --nocapture
cargo test skills_list_performance::test_skills_list_search_query_mock_performance -- --nocapture

# Skills install tests
cargo test skills_install_performance::test_skill_install_script_generation_performance -- --nocapture
cargo test skills_install_performance::test_skill_install_script_generation_multiple_skills -- --nocapture
```

## Timing Thresholds

### Current Thresholds

| Operation | Threshold | Test File |
|-----------|-----------|-----------|
| Skills list (empty directory) | < 3 seconds | [`skills_list_performance.rs`](../tests/skills_list_performance.rs) |
| Skills list (multiple skills) | < 3 seconds | [`skills_list_performance.rs`](../tests/skills_list_performance.rs) |
| Skills list (with search) | < 3 seconds | [`skills_list_performance.rs`](../tests/skills_list_performance.rs) |
| Global skills scan | < 3 seconds | [`skills_list_performance.rs`](../tests/skills_list_performance.rs) |
| npx availability check | < 3 seconds | [`skills_list_performance.rs`](../tests/skills_list_performance.rs) |
| Script generation (1-10 skills) | < 1 second | [`skills_install_performance.rs`](../tests/skills_install_performance.rs) |
| Full skill installation | < 15 seconds (target) | Manual testing only |

### Rationale for Thresholds

#### 3-Second Threshold (Skills List)

The 3-second threshold is based on:
1. **User Experience Research** - Operations exceeding 2-3 seconds are perceived as "slow"
2. **Industry Comparison** - npm, cargo, pip list commands complete in under 2 seconds
3. **Frequency of Use** - `skills list` is a frequently-used discovery command
4. **Operational Context** - May be run in scripts or automation workflows

#### 1-Second Threshold (Script Generation)

Script generation is purely CPU-bound string processing:
1. **Fast by Design** - Only template expansion, no I/O or network
2. **Linear Scaling** - O(n) complexity with skill count
3. **Preparation Phase** - Only a small fraction of total installation time

#### 15-Second Target (Full Installation)

The 15-second target includes:
1. Docker container creation and startup
2. npx package download and installation
3. Network-dependent operations

This target requires integration testing and cannot be reliably measured in unit tests.

## Factors Affecting Performance

### 1. Disk I/O Performance

**Impact:** High (skills list), Low (script generation)

- Reading skill manifest files from disk
- Directory traversal and enumeration
- System cache effects (warm vs. cold cache)

**Mitigation:**
- Ensure skills directory structure is optimized
- Minimize redundant file reads
- Use buffered I/O operations

### 2. Network Conditions

**Impact:** High (full installation), Low (script generation)

- npx availability checking
- npm package downloads
- Global skills directory access (if network-mounted)

**Mitigation:**
- Set reasonable timeouts for network operations
- Implement fallback behavior when network is unavailable
- Cache availability check results

### 3. Number of Installed Skills

**Impact:** Medium to High

- More skills = more files to read and parse
- More skills = more metadata to process
- Search filtering scales with skill count

**Mitigation:**
- Parallel file reading where possible
- Efficient data structures for lookups
- Lazy evaluation of skill details

### 4. System Load

**Impact:** Medium

- CPU contention slows parsing and processing
- Memory pressure affects caching
- I/O contention affects file reading

**Note:** Performance tests should be run on systems with minimal load for consistency.

### 5. Skill Manifest Complexity

**Impact:** Low to Medium

- Large frontmatter blocks take longer to parse
- Complex YAML/TOML structures require more processing
- Malformed manifests trigger error handling paths

### 6. Docker Environment

**Impact:** High (full installation only)

- Docker daemon performance
- Image availability (cached vs. pulled)
- Container startup time

## CI/CD Performance Regression Detection

### Running Performance Tests in CI

Add performance tests to your CI pipeline:

```yaml
# Example GitHub Actions workflow
name: Performance Tests

on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run skills list performance tests
        run: cargo test --test skills_list_performance -- --nocapture
      
      - name: Run skills install performance tests
        run: cargo test --test skills_install_performance -- --nocapture
      
      - name: Run metrics tests
        run: cargo test --test skills_install_time_metrics -- --nocapture
```

### Tracking Performance Over Time

#### Option 1: Store Test Results

```bash
# Capture test output with timestamps
cargo test --test skills_list_performance -- --nocapture 2>&1 | \
  tee performance-results-$(date +%Y%m%d-%H%M%S).log
```

#### Option 2: Use Custom Benchmark Runner

Create a script to track historical performance:

```bash
#!/bin/bash
# benchmark_runner.sh

TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="performance-results-${TIMESTAMP}.json"

echo "Running performance tests..."

# Run tests and capture JSON output
cargo test --test skills_list_performance -- --nocapture --report-time json | \
  jq -r '.[] | {name: .name, time: .exec_time}' > "$RESULTS_FILE"

echo "Results saved to $RESULTS_FILE"
```

#### Option 3: Integrate with Monitoring

For production environments, integrate with metrics collection:

```bash
# After running switchboard commands
cat switchboard.toml | grep -q "metrics_file" && \
  jq '.skills_install_time_seconds' "$(grep metrics_file switchboard.toml | cut -d'"' -f2)"
```

### Alert Thresholds

Configure alerts based on percentage degradation:

| Metric | Warning Threshold | Critical Threshold |
|--------|------------------|-------------------|
| Skills list time | > 2 seconds | > 3 seconds |
| Script generation time | > 500ms | > 1 second |
| Full installation time | > 12 seconds | > 15 seconds |

### Regression Detection Strategy

1. **Baseline Establishment**
   - Run tests on a clean system
   - Record average times over multiple runs
   - Establish standard deviation

2. **Continuous Monitoring**
   - Run performance tests on every commit
   - Compare against baseline
   - Track trends over time

3. **Alert Configuration**
   - Set thresholds at 2x baseline for warnings
   - Set thresholds at 3x baseline for failures
   - Account for system variability

4. **Investigation Process**
   ```
   1. Identify failing test
   2. Check system conditions (load, disk, network)
   3. Review recent code changes
   4. Profile the code
   5. Verify fix with before/after comparison
   ```

### Example CI Configuration

```yaml
# .github/workflows/performance.yml
name: Performance Regression

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: ~/.cargo
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run performance tests
        run: |
          echo "Running skills list performance tests..."
          cargo test --test skills_list_performance -- --nocapture
          
          echo "Running skills install performance tests..."
          cargo test --test skills_install_performance -- --nocapture
      
      - name: Upload results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: performance-results
          path: performance-results*.log
```

## Test Design Patterns

### Using Performance Common Helpers

```rust
use performance_common::{measure, assert_performance_threshold, PerformanceThreshold};

#[tokio::test]
async fn my_performance_test() {
    // Use the measure helper with warmup
    let result = measure(
        "my_operation".to_string(),
        10,  // iterations
        || async {
            // Your async operation here
            do_work().await;
        }
    ).await;
    
    // Assert against threshold
    let threshold = PerformanceThreshold::new(1.0);  // 1 second
    assert_performance_threshold(&result, &threshold)
        .expect("Performance threshold not met");
}
```

### Direct Timing (Current Implementation)

```rust
use std::time::Instant;

#[tokio::test]
async fn my_performance_test() {
    let start = Instant::now();
    
    // Perform operation
    do_work().await;
    
    let duration = start.elapsed();
    
    assert!(
        duration.as_secs() < 3,
        "Operation took {:?}, expected < 3 seconds",
        duration
    );
}
```

## Maintenance Notes

### Updating Thresholds

When updating performance thresholds:

1. Document the rationale for the change
2. Run tests multiple times to establish new baseline
3. Consider system variability
4. Update this document and related documentation

### Adding New Performance Tests

1. Follow the naming convention: `test_<operation>_<scenario>_performance`
2. Use meaningful threshold values based on user expectations
3. Document the test in the appropriate documentation file
4. Add to CI pipeline

## References

- Test files: [`tests/skills_list_performance.rs`](../tests/skills_list_performance.rs), [`tests/skills_install_performance.rs`](../tests/skills_install_performance.rs)
- Infrastructure: [`tests/performance_common.rs`](../tests/performance_common.rs)
- Existing documentation: [`docs/PERFORMANCE_SKILLS_LIST.md`](./PERFORMANCE_SKILLS_LIST.md), [`docs/PERFORMANCE_SKILLS_INSTALL.md`](./PERFORMANCE_SKILLS_INSTALL.md)

---

**Last Updated:** 2026-02-21  
**Maintainer:** Development Team  
**Related Sprint:** Sprint 4, Task 1
