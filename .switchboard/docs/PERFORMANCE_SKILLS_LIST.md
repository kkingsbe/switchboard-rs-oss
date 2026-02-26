# Performance Expectations: `switchboard skills list` Command

This document defines the performance expectations for the `switchboard skills list` command and provides guidance on testing, troubleshooting, and performance optimization.

## Overview

The `switchboard skills list` command is a core CLI functionality that users frequently interact with to discover and manage their installed skills. As such, it must be responsive and predictable across various usage scenarios.

## Performance Targets

### Primary Performance Threshold: **3 Seconds**

All `switchboard skills list` command invocations should complete within **3 seconds**, regardless of:
- Number of installed skills
- Presence or absence of search queries
- Whether global skills are being scanned
- npx availability status

### Specific Scenario Expectations

| Scenario | Expected Duration | Notes |
|----------|------------------|-------|
| **Empty skills directory** | < 100ms | Fastest scenario - minimal disk I/O |
| **Multiple installed skills** | < 3 seconds | Should scale linearly, not exponentially |
| **With search query (`--search`)** | < 3 seconds | Filtering should not significantly impact performance |
| **Global skills scan** | < 3 seconds | Includes checking global skills directory |
| **npx availability check** | < 1 second | Single system command invocation |

## Rationale for 3-Second Threshold

The 3-second performance threshold is based on several key considerations:

1. **User Experience (UX) Principles**
   - Research shows that users perceive operations taking longer than 2-3 seconds as "slow" or "laggy"
   - The 3-second threshold provides a buffer above the 1-second "instant" perception
   - Commands exceeding this threshold may cause users to suspect something is wrong

2. **Frequency of Use**
   - `skills list` is a discovery command that users run frequently
   - Poor performance on frequently-used commands compounds user frustration
   - Users may need to run this command multiple times during a single session

3. **Comparison with Similar Tools**
   - Most CLI package managers (npm, cargo, pip) list subcommands complete in under 2 seconds
   - Maintaining parity with industry standards prevents competitive disadvantage
   - Rust's performance capabilities should enable us to meet or exceed these standards

4. **Operational Context**
   - The command may be run as part of scripts or automation workflows
   - Longer delays can cascade and affect overall system performance
   - Fast execution enables real-time feedback in interactive shells

## Performance Characteristics

### Expected Scalability Behavior

The `switchboard skills list` command should exhibit **O(n)** complexity where:
- `n` = number of installed skills
- Each skill incurs a roughly constant overhead for:
  - Frontmatter parsing
  - Metadata extraction
  - Display formatting
  - Search filtering (when applicable)

**The command must NOT exhibit O(n²) or worse behavior**, which would indicate implementation issues such as:
- Nested loops over skill collections
- Repeated filesystem reads
- Inefficient string matching algorithms
- Unnecessary cloning of data structures

## Factors That May Affect Performance

### 1. Disk I/O Performance

**Impact:** High
- Reading skill manifest files from disk
- Directory traversal and enumeration
- System cache effects (warm vs. cold cache)

**Mitigation:**
- Ensure skills directory structure is optimized
- Minimize redundant file reads
- Use buffered I/O operations
- Consider caching parsed manifests (future enhancement)

### 2. Network Conditions

**Impact:** Low (for local skills) / High (for global/remote skills)
- Checking npx availability may involve network calls
- Global skills directory might be on network-mounted storage
- Some skills may fetch remote metadata (if implemented)

**Mitigation:**
- Set reasonable timeouts for network operations
- Implement fallback behavior when network is unavailable
- Cache results of availability checks

### 3. Number of Installed Skills

**Impact:** Medium to High (depends on implementation quality)
- More skills = more files to read and parse
- More skills = more metadata to process
- Search queries may filter more items

**Mitigation:**
- Parallel file reading where possible
- Efficient data structures for lookups
- Lazy evaluation of skill details

### 4. System Load

**Impact:** Medium
- CPU contention slows parsing and processing
- Memory pressure may affect caching
- I/O contention affects file reading speed

**Note:** Performance tests should be run on systems with minimal load for consistency.

### 5. Skill Manifest Complexity

**Impact:** Low to Medium
- Large frontmatter blocks take longer to parse
- Complex YAML/TOML structures require more processing
- Malformed manifests may trigger error handling paths

**Mitigation:**
- Validate manifest structure during installation
- Use efficient parsing libraries
- Implement graceful error handling for malformed files

## Running Performance Tests

### Prerequisites

Ensure you have:
- Rust toolchain installed
- The switchboard project cloned
- Write permissions to create temporary test directories

### Running All Performance Tests

```bash
cargo test --test skills_list_performance -- --nocapture
```

### Running Specific Test Scenarios

```bash
# Test empty skills directory
cargo test skills_list_performance::test_empty_skills_list_performance -- --nocapture

# Test multiple installed skills
cargo test skills_list_performance::test_multiple_skills_list_performance -- --nocapture

# Test with search query
cargo test skills_list_performance::test_skills_search_performance -- --nocapture
```

### Understanding Test Output

The performance tests will output timing information in milliseconds:

```
running 3 tests
test skills_list_performance::test_empty_skills_list_performance ... ok

  Empty skills list completed in 45ms
  Threshold: 3000ms, Status: PASSED
```

Key values to note:
- **Duration**: Actual execution time in milliseconds
- **Threshold**: Maximum allowed time (3000ms)
- **Status**: PASSED or FAILED based on comparison

## Troubleshooting Failed Performance Tests

If performance tests fail, follow this systematic approach:

### Step 1: Identify the Failed Scenario

Determine which specific scenario is failing:
- Empty skills directory (< 100ms expected)
- Multiple installed skills (< 3000ms expected)
- Search with query (< 3000ms expected)

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
cargo flamegraph --test skills_list_performance <test_name>
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
- **Symptom**: Nested loops over collections
- **Fix**: Use `HashMap` or `HashSet` for O(1) lookups
- **Check**: Review algorithm complexity

#### 3. Unnecessary Cloning
- **Symptom**: Frequent `.clone()` calls on large structs
- **Fix**: Use references (`&T`) or `Cow` types
- **Check**: Audit `.clone()` usage with `cargo clippy`

#### 4. Synchronous Blocking Operations
- **Symptom**: Operations waiting on external resources
- **Fix**: Implement timeouts, consider async/await
- **Check**: Identify blocking I/O calls

#### 5. Repeated npx Checks
- **Symptom**: npx availability checked multiple times per invocation
- **Fix**: Cache the result for the duration of the command
- **Check**: Count npx check invocations

### Step 6: Verify Fixes

After making changes:
1. Run the failing test again
2. Verify it now passes
3. Run all performance tests to ensure no regressions
4. Run the full test suite to verify functionality isn't broken

## Performance Monitoring in Production

### Recommended Metrics

Track these metrics in production to catch performance regressions early:

1. **Command Duration**: Distribution (p50, p95, p99)
2. **Skill Count**: Correlation with duration
3. **Error Rate**: How often performance-related errors occur
4. **Cache Hit Rate**: If caching is implemented

### Benchmarking Script

Create a benchmark script to periodically test performance:

```bash
#!/bin/bash
# benchmark_skills_list.sh

echo "Running skills list performance benchmark..."
time switchboard skills list > /dev/null
echo "Run completed"
```

Run this script regularly and log results for trend analysis.

## Performance Optimization Guidelines

### DO

- Use efficient data structures (`HashMap`, `HashSet`)
- Minimize file I/O operations
- Cache frequently-accessed data
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

- Test file: [`tests/skills_list_performance.rs`](../tests/skills_list_performance.rs)
- Implementation: [`src/cli/skills.rs`](../src/cli/skills.rs) (path may vary)
- Skills documentation: [`docs/skills-feature.md`](../docs/skills-feature.md)

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2026-02-20 | Initial documentation of performance expectations |

---

**Last Updated:** 2026-02-20  
**Maintainer:** Development Team  
**Related Sprint:** Sprint 4, Task 1
