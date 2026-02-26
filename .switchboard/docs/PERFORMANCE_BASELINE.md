# Performance Baseline Metrics

This document defines the baseline performance metrics for switchboard operations, including expected timing ranges and definitions of normal vs. slow performance.

## Baseline Overview

The following baseline metrics were established through testing on typical development hardware. Actual performance may vary based on system configuration, load, and network conditions.

## Skills List Operations

### Empty Skills Directory

| Metric | Value |
|--------|-------|
| **Expected Time** | < 100ms |
| **Threshold (FAIL)** | > 3 seconds |
| **Typical Range** | 10ms - 100ms |

**What is measured:**
- Directory existence check
- Empty directory scan
- Result formatting

**Factors that affect baseline:**
- Filesystem performance
- System cache state
- Storage type (SSD vs HDD)

**Normal vs. Slow:**
- ✅ **Normal:** 10-100ms
- ⚠️ **Warning:** 100ms - 500ms
- ❌ **Slow:** > 3 seconds

### Multiple Skills Listing (3 skills)

| Metric | Value |
|--------|-------|
| **Expected Time** | < 500ms |
| **Threshold (FAIL)** | > 3 seconds |
| **Typical Range** | 50ms - 500ms |

**What is measured:**
- Directory traversal
- SKILL.md file reading (3 files)
- Frontmatter parsing (3 files)
- Metadata extraction
- Result formatting

**Factors that affect baseline:**
- Number of skills
- SKILL.md file size
- Frontmatter complexity

**Normal vs. Slow:**
- ✅ **Normal:** 50-500ms
- ⚠️ **Warning:** 500ms - 1 second
- ❌ **Slow:** > 3 seconds

### Multiple Skills Listing (5 skills)

| Metric | Value |
|--------|-------|
| **Expected Time** | < 1 second |
| **Threshold (FAIL)** | > 3 seconds |
| **Typical Range** | 100ms - 800ms |

**Normal vs. Slow:**
- ✅ **Normal:** 100-800ms
- ⚠️ **Warning:** 800ms - 2 seconds
- ❌ **Slow:** > 3 seconds

### Search Query Filtering

| Metric | Value |
|--------|-------|
| **Expected Time** | < 500ms |
| **Threshold (FAIL)** | > 3 seconds |
| **Typical Range** | 100ms - 500ms |

**What is measured:**
- All skills scan
- Name/description filtering
- Result formatting

**Normal vs. Slow:**
- ✅ **Normal:** 100-500ms
- ⚠️ **Warning:** 500ms - 1 second
- ❌ **Slow:** > 3 seconds

### Global Skills Scan

| Metric | Value |
|--------|-------|
| **Expected Time** | < 500ms |
| **Threshold (FAIL)** | > 3 seconds |
| **Typical Range** | 50ms - 500ms |

**Normal vs. Slow:**
- ✅ **Normal:** 50-500ms
- ⚠️ **Warning:** 500ms - 1 second
- ❌ **Slow:** > 3 seconds

### npx Availability Check

| Metric | Value |
|--------|-------|
| **Expected Time** | < 100ms |
| **Threshold (FAIL)** | > 3 seconds |
| **Typical Range** | 10ms - 100ms |

**Normal vs. Slow:**
- ✅ **Normal:** 10-100ms
- ⚠️ **Warning:** 100ms - 500ms
- ❌ **Slow:** > 3 seconds

## Skills Installation Operations

### Entrypoint Script Generation (1 skill)

| Metric | Value |
|--------|-------|
| **Expected Time** | < 100ms |
| **Threshold (FAIL)** | > 1 second |
| **Typical Range** | 10ms - 100ms |

**What is measured:**
- Configuration parsing
- Template expansion
- Script string construction

**Normal vs. Slow:**
- ✅ **Normal:** 10-100ms
- ⚠️ **Warning:** 100ms - 500ms
- ❌ **Slow:** > 1 second

### Entrypoint Script Generation (5 skills)

| Metric | Value |
|--------|-------|
| **Expected Time** | < 300ms |
| **Threshold (FAIL)** | > 1 second |
| **Typical Range** | 50ms - 300ms |

**Normal vs. Slow:**
- ✅ **Normal:** 50-300ms
- ⚠️ **Warning:** 300ms - 500ms
- ❌ **Slow:** > 1 second

### Entrypoint Script Generation (10 skills)

| Metric | Value |
|--------|-------|
| **Expected Time** | < 500ms |
| **Threshold (FAIL)** | > 1 second |
| **Typical Range** | 100ms - 500ms |

**Normal vs. Slow:**
- ✅ **Normal:** 100-500ms
- ⚠️ **Warning:** 500ms - 800ms
- ❌ **Slow:** > 1 second

### Full Skill Installation (Docker + npx)

| Metric | Value |
|--------|-------|
| **Target Time** | < 15 seconds |
| **Threshold (FAIL)** | > 30 seconds |
| **Typical Range** | 10-15 seconds |

**What is measured (integration test only):**
- Docker container creation
- Container startup
- npx package download
- npx package installation
- Skill initialization

**Note:** Full installation requires Docker and network, and is not measured in unit tests.

**Normal vs. Slow:**
- ✅ **Normal:** 10-15 seconds
- ⚠️ **Warning:** 15-20 seconds
- ❌ **Slow:** > 30 seconds

## Metrics Validation

### skills_install_time_seconds Field

| Metric | Expected Range |
|--------|---------------|
| **Valid Range** | 0.1 - 300 seconds |
| **None when** | No skills configured |
| **Some when** | Skills installation attempted |

**Validation Tests:**
- Field is `Some(value)` when skills installed
- Field is `None` when no skills configured
- Value persists to metrics.json correctly
- Fractional seconds are preserved

## Performance Classification Guide

### How to Classify Performance

Use this guide to classify the performance of an operation:

| Classification | Criteria | Action Required |
|---------------|----------|-----------------|
| **Excellent** | < 50% of threshold | None - exceeds expectations |
| **Good** | 50-75% of threshold | None - within expected range |
| **Acceptable** | 75-100% of threshold | None - meets requirements |
| **Warning** | 100-150% of threshold | Monitor and investigate |
| **Slow** | 150-200% of threshold | Log issue, investigate cause |
| **Failed** | > 200% of threshold | Fix required before merge |

### Example Classification

For a 3-second threshold operation:

| Measured Time | Classification | % of Threshold |
|---------------|---------------|----------------|
| 0.5 seconds | Excellent | 17% |
| 1.5 seconds | Good | 50% |
| 2.5 seconds | Acceptable | 83% |
| 3.5 seconds | Warning | 117% |
| 5.0 seconds | Slow | 167% |
| 7.0 seconds | Failed | 233% |

## Common Performance Patterns

### Linear Scaling (Expected)

Performance should scale linearly with skill count:

```
Skills: 1    → Time: ~50ms
Skills: 5    → Time: ~250ms (5x)
Skills: 10   → Time: ~500ms (10x)
```

### Non-Linear Scaling (Issue)

If performance degrades faster than linear, investigate:

- Nested loops over skill collections
- Repeated file reads
- Inefficient string operations
- Unnecessary cloning

### Cold vs. Warm Cache

| Condition | Expected Impact |
|-----------|-----------------|
| **Cold cache** | 2-5x slower for disk I/O |
| **Warm cache** | Baseline performance |

Run tests multiple times to establish warm cache baseline.

## Troubleshooting Guide

### Performance is Slow - Quick Check

1. **Is it consistent?** - Run test 3 times
2. **Is system under load?** - Check `top` / `htop`
3. **Is disk I/O high?** - Check `iostat`
4. **Is network involved?** - Check connectivity

### Common Causes of Degradation

| Symptom | Likely Cause | Investigation |
|---------|--------------|----------------|
| High CPU | Algorithmic issue | Profile code |
| High I/O wait | Disk bottleneck | Check disk usage |
| High memory | Memory pressure | Check `free -h` |
| Network delays | DNS/connectivity | Check network |

### When to Investigate

Investigate if performance deviates by more than 50% from baseline without obvious external factors.

## Baseline Maintenance

### Updating Baselines

When updating baseline metrics:

1. Run tests on reference hardware
2. Collect data from multiple runs (minimum 5)
3. Calculate mean and standard deviation
4. Document the reference system
5. Update this document

### Reference System Specs

| Component | Specification |
|-----------|---------------|
| CPU | 4+ cores @ 2.5GHz |
| RAM | 8GB+ |
| Storage | SSD |
| OS | Linux (Ubuntu 22.04+) |

### Recording New Baselines

```bash
# Run tests multiple times and collect output
for i in {1..5}; do
  echo "Run $i:"
  cargo test --test skills_list_performance -- --nocapture 2>&1 | \
    grep -E "(completed in|took)"
done
```

## Historical Performance Data

| Date | Skills List | Script Gen (5) | Notes |
|------|-------------|----------------|-------|
| 2026-02-20 | < 3s (baseline) | < 1s (baseline) | Initial baseline |

## References

- Test files: [`tests/skills_list_performance.rs`](../tests/skills_list_performance.rs), [`tests/skills_install_performance.rs`](../tests/skills_install_performance.rs)
- Methodology: [`docs/PERFORMANCE_TESTING_METHODOLOGY.md`](./PERFORMANCE_TESTING_METHODOLOGY.md)
- Skills list docs: [`docs/PERFORMANCE_SKILLS_LIST.md`](./PERFORMANCE_SKILLS_LIST.md)
- Skills install docs: [`docs/PERFORMANCE_SKILLS_INSTALL.md`](./PERFORMANCE_SKILLS_INSTALL.md)

---

**Last Updated:** 2026-02-21  
**Maintainer:** Development Team  
**Related Sprint:** Sprint 4, Task 1
