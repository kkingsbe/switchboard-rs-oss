<<<<<<< HEAD
//! Shared performance test infrastructure module
//!
//! This module provides reusable utilities for performance testing across
//! all performance test files. It includes:
//!
//! - Benchmark result tracking with metrics
//! - Performance threshold assertion
//! - Benchmark measurement with warmup
//! - Summary printing for visibility
//! - Test setup helpers (temp dirs, env vars)

use std::future::Future;
use std::sync::Mutex;
use tempfile::TempDir;

/// Benchmark result containing timing and throughput metrics
///
/// This struct captures the results of a single benchmark run, including
/// the duration, number of iterations, and optional throughput metrics.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Duration in seconds (average across all iterations)
    pub duration_secs: f64,
    /// Number of iterations performed
    pub iterations: u32,
    /// Optional throughput metric (operations per second)
    pub throughput_ops_per_sec: Option<f64>,
}

impl BenchmarkResult {
    /// Creates a new benchmark result
    pub fn new(
        name: String,
        duration_secs: f64,
        iterations: u32,
        throughput_ops_per_sec: Option<f64>,
    ) -> Self {
        Self {
            name,
            duration_secs,
            iterations,
            throughput_ops_per_sec,
        }
    }
}

/// Performance threshold for benchmark validation
///
/// This struct defines the acceptable performance bounds for a benchmark.
/// A benchmark passes if its duration is within the maximum allowed time
/// and (optionally) its throughput meets the minimum required value.
#[derive(Debug, Clone)]
pub struct PerformanceThreshold {
    /// Maximum allowed duration in seconds
    pub max_duration_secs: f64,
    /// Optional minimum required throughput (operations per second)
    pub min_throughput_ops_per_sec: Option<f64>,
}

impl PerformanceThreshold {
    /// Creates a new performance threshold with duration only
    pub fn new(max_duration_secs: f64) -> Self {
        Self {
            max_duration_secs,
            min_throughput_ops_per_sec: None,
        }
    }

    /// Creates a new performance threshold with both duration and throughput
    pub fn with_throughput(max_duration_secs: f64, min_throughput_ops_per_sec: f64) -> Self {
        Self {
            max_duration_secs,
            min_throughput_ops_per_sec: Some(min_throughput_ops_per_sec),
        }
    }
}

/// Measures the performance of an async operation across multiple iterations
///
/// This function runs the provided async function multiple times, measuring
/// the average duration. A warmup iteration is performed before timing starts
/// to account for JIT compilation and initialization effects.
///
/// # Arguments
///
/// * `name` - Name for the benchmark
/// * `iterations` - Number of timed iterations to perform
/// * `f` - Async function to benchmark
///
/// # Returns
///
/// A `BenchmarkResult` containing the average duration and iteration count
///
/// # Example
///
/// ```no_run
/// # use switchboard::some_async_function;
/// # async fn example() {
/// let result = performance_common::measure(
///     "my_benchmark".to_string(),
///     10,
///     || async { some_async_function().await }
/// ).await;
///
/// println!("{} completed in {:.4}s ({} iterations)",
///     result.name, result.duration_secs, result.iterations);
/// # }
/// ```
pub async fn measure<F, Fut>(name: String, iterations: u32, f: F) -> BenchmarkResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    // Warmup iteration (not timed)
    let _ = f().await;

    // Timed iterations
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f().await;
    }
    let total_duration = start.elapsed();

    let duration_secs = total_duration.as_secs_f64() / iterations as f64;

    BenchmarkResult::new(name, duration_secs, iterations, None)
}

/// Measures the performance of an async operation across multiple iterations with throughput
///
/// This function is similar to `measure` but also calculates throughput metrics.
/// The operation count parameter allows calculating operations per second.
///
/// # Arguments
///
/// * `name` - Name for the benchmark
/// * `iterations` - Number of timed iterations to perform
/// * `operations_per_iteration` - Number of operations performed in each iteration
/// * `f` - Async function to benchmark
///
/// # Returns
///
/// A `BenchmarkResult` containing the average duration, iteration count, and throughput
pub async fn measure_with_throughput<F, Fut>(
    name: String,
    iterations: u32,
    operations_per_iteration: u32,
    f: F,
) -> BenchmarkResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    // Warmup iteration (not timed)
    let _ = f().await;

    // Timed iterations
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f().await;
    }
    let total_duration = start.elapsed();

    let duration_secs = total_duration.as_secs_f64() / iterations as f64;

    // Calculate throughput: operations per second
    let total_operations = operations_per_iteration as f64 * iterations as f64;
    let throughput_ops_per_sec = total_operations / total_duration.as_secs_f64();

    BenchmarkResult::new(
        name,
        duration_secs,
        iterations,
        Some(throughput_ops_per_sec),
    )
}

/// Asserts that a benchmark result meets the performance threshold
///
/// This function validates that the benchmark result is within acceptable
/// performance bounds. It checks both duration and (optionally) throughput.
///
/// # Arguments
///
/// * `result` - The benchmark result to validate
/// * `threshold` - The performance threshold to check against
///
/// # Returns
///
/// * `Ok(())` if all assertions pass
/// * `Err(String)` with a descriptive error message if any assertion fails
///
/// # Example
///
/// ```no_run
/// # use performance_common::{BenchmarkResult, PerformanceThreshold};
/// # fn example() {
/// let result = BenchmarkResult::new("test".to_string(), 1.5, 10, Some(100.0));
/// let threshold = PerformanceThreshold::new(2.0);
///
/// assert_performance_threshold(&result, &threshold)
///     .expect("Performance threshold not met");
/// # }
/// ```
pub fn assert_performance_threshold(
    result: &BenchmarkResult,
    threshold: &PerformanceThreshold,
) -> Result<(), String> {
    // Check duration threshold
    if result.duration_secs > threshold.max_duration_secs {
        return Err(format!(
            "Duration threshold failed: '{}' took {:.4}s, expected <= {:.4}s",
            result.name, result.duration_secs, threshold.max_duration_secs
        ));
    }

    // Check throughput threshold if specified
    if let Some(min_throughput) = threshold.min_throughput_ops_per_sec {
        match result.throughput_ops_per_sec {
            Some(actual_throughput) => {
                if actual_throughput < min_throughput {
                    return Err(format!(
                        "Throughput threshold failed: '{}' achieved {:.2} ops/sec, expected >= {:.2} ops/sec",
                        result.name, actual_throughput, min_throughput
                    ));
                }
            }
            None => {
                return Err(format!(
                    "Throughput threshold specified but result '{}' has no throughput data",
                    result.name
                ));
            }
        }
    }

    Ok(())
}

/// Prints a formatted summary table of benchmark results
///
/// This function displays all benchmark results in a readable table format,
/// showing the name, duration, iterations, and optional throughput metrics.
/// Useful for test output visibility and quick performance assessment.
///
/// # Arguments
///
/// * `results` - Slice of benchmark results to display
///
/// # Example
///
/// ```no_run
/// # use performance_common::{BenchmarkResult, print_benchmark_summary};
/// # fn example() {
/// let results = vec![
///     BenchmarkResult::new("benchmark1".to_string(), 0.5, 10, Some(100.0)),
///     BenchmarkResult::new("benchmark2".to_string(), 1.2, 10, None),
/// ];
/// print_benchmark_summary(&results);
/// # }
/// ```
pub fn print_benchmark_summary(results: &[BenchmarkResult]) {
    println!("\n=== Benchmark Summary ===\n");

    if results.is_empty() {
        println!("No benchmark results to display.\n");
        return;
    }

    // Calculate column widths
    let name_width = results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or("Name".len());
    let name_width = std::cmp::max(name_width, 4);

    // Print header
    println!(
        "{:<name_width$} | {:>10} | {:>10} | {:>15}",
        "Name", "Time (s)", "Iterations", "Throughput (ops/s)"
    );
    println!(
        "{:-<name_width$}-+-{:-<10}-+-{:-<10}-+-{:-<15}",
        "", "", "", ""
    );

    // Print each result
    for result in results {
        let throughput_str = match result.throughput_ops_per_sec {
            Some(tp) => format!("{:>15.2}", tp),
            None => String::from("           N/A"),
        };
        println!(
            "{:<name_width$} | {:>10.4} | {:>10} | {}",
            result.name, result.duration_secs, result.iterations, throughput_str
        );
    }

    println!();
}

/// Creates a temporary directory for test isolation
///
/// This function creates a new temporary directory that will be automatically
/// deleted when the returned `TempDir` value is dropped.
///
/// # Returns
///
/// A `TempDir` handle that automatically cleans up the directory when dropped
///
/// # Example
///
/// ```no_run
/// # use tempfile::TempDir;
/// # fn example() {
/// let temp_dir = performance_common::create_temp_dir();
/// let test_path = temp_dir.path().join("test.txt");
/// // Write test data...
/// // temp_dir is automatically cleaned up when it goes out of scope
/// # }
/// ```
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Sets an environment variable and returns information to restore it
///
/// This function sets an environment variable and returns a tuple containing
/// the previous value and a mutex-protected cleanup guard. This ensures
/// the environment variable can be restored after the test completes.
///
/// # Arguments
///
/// * `key` - Environment variable name
/// * `value` - New value for the environment variable
///
/// # Returns
///
/// A tuple of (old_value, restore_guard) where restore_guard is a Mutex
/// containing the old value that can be used to restore the original value
///
/// # Example
///
/// ```no_run
/// # fn example() {
/// let (old_home, guard) = performance_common::set_env_var("HOME", "/tmp/test");
/// // Run test with modified environment...
/// // Restore original value when done
/// if let Some(old_val) = guard.lock().unwrap().take() {
///     std::env::set_var("HOME", old_val);
/// }
/// # }
/// ```
pub fn set_env_var(key: &str, value: &str) -> (String, Option<Mutex<Option<String>>>) {
    let old_value = std::env::var(key).ok();
    std::env::set_var(key, value);
    let restore_guard = Mutex::new(old_value);
    (value.to_string(), Some(restore_guard))
}

/// Restores an environment variable to its previous value
///
/// This function restores an environment variable to its original value
/// using the restore guard returned by `set_env_var`.
///
/// # Arguments
///
/// * `key` - Environment variable name
/// * `restore_guard` - Mutex containing the optional old value to restore
///
/// # Example
///
/// ```no_run
/// # fn example() {
/// let (old_home, guard) = performance_common::set_env_var("HOME", "/tmp/test");
/// // ... test code ...
/// performance_common::restore_env_var("HOME", guard);
/// # }
/// ```
pub fn restore_env_var(key: &str, restore_guard: Option<Mutex<Option<String>>>) {
    if let Some(guard) = restore_guard {
        let old_value = guard.lock().unwrap().take();
        match old_value {
            Some(val) => std::env::set_var(key, val),
            None => std::env::remove_var(key),
        }
    }
}

// ============================================================================
// Enhanced Timing Utilities with Microsecond Precision
// ============================================================================

/// High-precision timer for performance measurements
///
/// This struct provides microsecond-level precision for timing operations,
/// with additional features like lap timing and automatic duration formatting.
#[derive(Clone)]
pub struct Timer {
    name: String,
    start: std::time::Instant,
    laps: Vec<(String, std::time::Duration)>,
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("name", &self.name)
            .field("laps", &self.laps)
            .finish()
    }
}

impl Timer {
    /// Creates a new timer with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: std::time::Instant::now(),
            laps: Vec::new(),
        }
    }

    /// Records a lap time with an optional label
    pub fn lap(&mut self, label: impl Into<String>) {
        let elapsed = self.start.elapsed();
        self.laps.push((label.into(), elapsed));
    }

    /// Returns the total elapsed time since timer creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    /// Returns elapsed time in microseconds for precise comparisons
    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Returns elapsed time in seconds (f64) for benchmarking
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Returns the timer name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns recorded laps
    pub fn laps(&self) -> &[(String, std::time::Duration)] {
        &self.laps
    }

    /// Formats elapsed time in a human-readable format
    pub fn format_elapsed(&self) -> String {
        let elapsed = self.elapsed();
        format_duration(elapsed)
    }

    /// Creates a BenchmarkResult from this timer
    pub fn to_benchmark_result(&self, iterations: u32) -> BenchmarkResult {
        BenchmarkResult::new(
            self.name.clone(),
            self.elapsed_secs() / iterations.max(1) as f64,
            iterations,
            None,
        )
    }
}

/// Formats a duration in a human-readable format
///
/// Displays the duration with appropriate precision (microseconds for short
/// durations, milliseconds for medium, seconds for long).
pub fn format_duration(duration: std::time::Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    } else {
        format!("{:.4}s", duration.as_secs_f64())
    }
}

/// Convenience function to quickly time an async operation
///
/// This is a simpler alternative to `measure` that returns the duration
/// directly without wrapping it in a BenchmarkResult.
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// let duration = performance_common::time_async("operation", || async {
///     // async operation
/// }).await;
/// println!("Operation took {:?}", duration);
/// # }
/// ```
pub async fn time_async<F, Fut>(name: &str, f: F) -> std::time::Duration
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut timer = Timer::new(name);
    f().await;
    timer.elapsed()
}

// ============================================================================
// Setup/Teardown Infrastructure for Performance Tests
// ============================================================================

/// Setup and teardown context for performance tests
///
/// This struct manages the test environment, ensuring proper isolation
/// and cleanup between performance test runs.
pub struct PerformanceTestSetup {
    /// Temporary directory for test isolation (if created)
    temp_dir: Option<TempDir>,
    /// Environment variables to restore (key -> original value)
    env_vars: Vec<(String, Option<String>)>,
    /// Custom cleanup functions (not included in Debug)
    #[allow(dead_code)]
    cleanup_fns: Vec<Box<dyn Fn() + Send>>,
}

impl std::fmt::Debug for PerformanceTestSetup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceTestSetup")
            .field("temp_dir", &self.temp_dir)
            .field("env_vars", &self.env_vars)
            .finish()
    }
}

impl PerformanceTestSetup {
    /// Creates a new empty test setup
    pub fn new() -> Self {
        Self {
            temp_dir: None,
            env_vars: Vec::new(),
            cleanup_fns: Vec::new(),
        }
    }

    /// Creates a new test setup with a temporary directory
    pub fn with_temp_dir(temp_dir: TempDir) -> Self {
        Self {
            temp_dir: Some(temp_dir),
            env_vars: Vec::new(),
            cleanup_fns: Vec::new(),
        }
    }

    /// Sets an environment variable, storing the original value for restoration
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        let old_value = std::env::var(&key).ok();
        std::env::set_var(&key, value);
        self.env_vars.push((key, old_value));
    }

    /// Adds a custom cleanup function to be called during teardown
    pub fn add_cleanup<F>(&mut self, cleanup_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        self.cleanup_fns.push(Box::new(cleanup_fn));
    }

    /// Returns the temporary directory path if one was created
    pub fn temp_dir(&self) -> Option<&std::path::Path> {
        self.temp_dir.as_ref().map(|td| td.path())
    }

    /// Performs teardown, restoring all environment variables and running cleanup
    pub fn teardown(self) {
        // Run custom cleanup functions first
        for cleanup_fn in self.cleanup_fns {
            cleanup_fn();
        }

        // Restore environment variables
        for (key, old_value) in self.env_vars {
            match old_value {
                Some(val) => std::env::set_var(&key, val),
                None => std::env::remove_var(&key),
            }
        }

        // TempDir is automatically cleaned up when dropped
    }
}

impl Default for PerformanceTestSetup {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a standard performance test setup with a fresh temporary directory
///
/// This is a convenience function for creating a standardized test environment.
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// let setup = performance_common::setup_performance_test();
/// let temp_dir = setup.temp_dir().unwrap();
///
/// // ... run performance test ...
///
/// setup.teardown();  // Automatically cleans up
/// # }
/// ```
pub fn setup_performance_test() -> PerformanceTestSetup {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    PerformanceTestSetup::with_temp_dir(temp_dir)
}

// ============================================================================
// Baseline Metrics Tracker
// ============================================================================

/// Baseline metric for a single test
///
/// Stores the baseline performance data for a specific test, including
/// the expected duration and optional throughput targets.
#[derive(Debug, Clone)]
pub struct BaselineMetric {
    /// Test name
    pub name: String,
    /// Expected maximum duration in seconds
    pub max_duration_secs: f64,
    /// Expected average duration in seconds
    pub avg_duration_secs: f64,
    /// Optional minimum throughput (ops/sec)
    pub min_throughput: Option<f64>,
    /// Number of samples in baseline
    pub sample_count: u32,
    /// Standard deviation of baseline samples
    pub std_dev_secs: f64,
}

impl BaselineMetric {
    /// Creates a new baseline metric
    pub fn new(name: impl Into<String>, avg_duration_secs: f64) -> Self {
        Self {
            name: name.into(),
            max_duration_secs: avg_duration_secs * 1.5, // 50% tolerance by default
            avg_duration_secs,
            min_throughput: None,
            sample_count: 1,
            std_dev_secs: 0.0,
        }
    }

    /// Creates a baseline metric with custom max duration
    pub fn with_max_duration(mut self, max_secs: f64) -> Self {
        self.max_duration_secs = max_secs;
        self
    }

    /// Creates a baseline metric with throughput target
    pub fn with_throughput(mut self, min_throughput: f64) -> Self {
        self.min_throughput = Some(min_throughput);
        self
    }

    /// Updates the baseline with a new sample
    pub fn update_with_sample(&mut self, duration_secs: f64) {
        // Calculate running mean and std dev using Welford's algorithm
        self.sample_count += 1;
        let delta = duration_secs - self.avg_duration_secs;
        self.avg_duration_secs += delta / self.sample_count as f64;
        let delta2 = duration_secs - self.avg_duration_secs;
        // Approximate standard deviation
        self.std_dev_secs =
            ((self.std_dev_secs.powi(2) * (self.sample_count - 1) as f64) + delta * delta2).sqrt()
                / (self.sample_count - 1).max(1) as f64;

        // Update max to be 50% above average or keep existing if lower
        let new_max = self.avg_duration_secs * 1.5;
        if new_max > self.max_duration_secs {
            self.max_duration_secs = new_max;
        }
    }

    /// Checks if a duration is within the baseline bounds
    pub fn is_within_bounds(&self, duration_secs: f64) -> bool {
        duration_secs <= self.max_duration_secs
    }

    /// Returns regression status
    pub fn check_regression(&self, duration_secs: f64) -> RegressionStatus {
        let ratio = duration_secs / self.avg_duration_secs;
        if ratio > 2.0 {
            RegressionStatus::SignificantRegression
        } else if ratio > 1.3 {
            RegressionStatus::MinorRegression
        } else if ratio < 0.7 {
            RegressionStatus::Improvement
        } else {
            RegressionStatus::Stable
        }
    }
}

/// Regression status indicating whether performance has degraded
#[derive(Debug, Clone, PartialEq)]
pub enum RegressionStatus {
    /// Performance is stable (within 30% of baseline)
    Stable,
    /// Minor regression (30-100% slower than baseline)
    MinorRegression,
    /// Significant regression (more than 2x slower than baseline)
    SignificantRegression,
    /// Performance has improved (more than 30% faster)
    Improvement,
}

impl RegressionStatus {
    /// Returns true if this status indicates a regression
    pub fn is_regression(&self) -> bool {
        matches!(
            self,
            RegressionStatus::MinorRegression | RegressionStatus::SignificantRegression
        )
    }

    /// Returns a human-readable description
    pub fn description(&self) -> &str {
        match self {
            RegressionStatus::Stable => "stable",
            RegressionStatus::MinorRegression => "minor regression",
            RegressionStatus::SignificantRegression => "significant regression",
            RegressionStatus::Improvement => "improvement",
        }
    }
}

/// Baseline tracker that records and compares performance metrics
///
/// This struct manages baseline metrics for multiple tests and provides
/// functionality to check for regressions.
#[derive(Debug, Default)]
pub struct BaselineTracker {
    metrics: std::collections::HashMap<String, BaselineMetric>,
    results: Vec<BenchmarkResult>,
}

impl BaselineTracker {
    /// Creates a new baseline tracker
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
            results: Vec::new(),
        }
    }

    /// Records a baseline metric for a test
    pub fn record_baseline(&mut self, metric: BaselineMetric) {
        let name = metric.name.clone();
        self.metrics.insert(name, metric);
    }

    /// Records a benchmark result
    pub fn record_result(&mut self, result: BenchmarkResult) {
        // Update baseline if it exists
        if let Some(baseline) = self.metrics.get_mut(&result.name) {
            baseline.update_with_sample(result.duration_secs);
        }
        self.results.push(result);
    }

    /// Checks if a result shows regression compared to baseline
    pub fn check_regression(&self, name: &str, duration_secs: f64) -> Option<RegressionStatus> {
        self.metrics
            .get(name)
            .map(|b| b.check_regression(duration_secs))
    }

    /// Gets the baseline metric for a test
    pub fn get_baseline(&self, name: &str) -> Option<&BaselineMetric> {
        self.metrics.get(name)
    }

    /// Returns all recorded results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Saves baseline metrics to a file (simple line-based format)
    ///
    /// Format: name|avg_duration|max_duration|sample_count|std_dev
    pub fn save_baseline_to_file(
        metrics: &std::collections::HashMap<String, BaselineMetric>,
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        let mut content = String::new();
        for (name, metric) in metrics {
            content.push_str(&format!(
                "{}|{}|{}|{}|{}\n",
                name,
                metric.avg_duration_secs,
                metric.max_duration_secs,
                metric.sample_count,
                metric.std_dev_secs
            ));
        }
        std::fs::write(path, content)
    }

    /// Loads baseline metrics from a file
    pub fn load_baseline_from_file(
        path: &std::path::Path,
    ) -> std::io::Result<std::collections::HashMap<String, BaselineMetric>> {
        let content = std::fs::read_to_string(path)?;
        let mut metrics = std::collections::HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let name = parts[0].to_string();
                let avg_duration_secs: f64 = parts[1].parse().unwrap_or(0.0);
                let max_duration_secs: f64 = parts[2].parse().unwrap_or(avg_duration_secs * 1.5);
                let sample_count: u32 = parts[3].parse().unwrap_or(1);
                let std_dev_secs: f64 = parts[4].parse().unwrap_or(0.0);

                metrics.insert(
                    name,
                    BaselineMetric {
                        name: parts[0].to_string(),
                        avg_duration_secs,
                        max_duration_secs,
                        min_throughput: None,
                        sample_count,
                        std_dev_secs,
                    },
                );
            }
        }

        Ok(metrics)
    }

    /// Prints a summary of baseline metrics
    pub fn print_summary(&self) {
        println!("\n=== Baseline Metrics Summary ===\n");
        if self.metrics.is_empty() {
            println!("No baseline metrics recorded.\n");
            return;
        }

        for (name, metric) in &self.metrics {
            println!("Test: {}", name);
            println!("  Average: {:.4}s", metric.avg_duration_secs);
            println!("  Max (threshold): {:.4}s", metric.max_duration_secs);
            println!("  Std Dev: {:.4}s", metric.std_dev_secs);
            println!("  Samples: {}", metric.sample_count);
            if let Some(tp) = metric.min_throughput {
                println!("  Min Throughput: {:.2} ops/s", tp);
            }
            println!();
        }
    }
}

/// Detects if performance has regressed compared to baseline
///
/// This function compares a new measurement against the baseline and returns
/// a regression status. If no baseline exists, it creates one.
///
/// # Arguments
///
/// * `tracker` - The baseline tracker to use
/// * `name` - The test name
/// * `duration_secs` - The measured duration in seconds
/// * `create_baseline_if_missing` - Whether to create a baseline if none exists
///
/// # Returns
///
/// A tuple of (regression_status, created_baseline)
pub fn detect_regression(
    tracker: &mut BaselineTracker,
    name: &str,
    duration_secs: f64,
    create_baseline_if_missing: bool,
) -> (Option<RegressionStatus>, bool) {
    let status = tracker.check_regression(name, duration_secs);

    if let Some(s) = status {
        (Some(s), false)
    } else if create_baseline_if_missing {
        let baseline = BaselineMetric::new(name, duration_secs);
        tracker.record_baseline(baseline);
        (None, true)
    } else {
        (None, false)
    }
}

/// Prints a regression warning if performance has degraded
///
/// This function logs a warning message if the regression status indicates
/// performance degradation.
pub fn log_regression_warning(name: &str, status: &RegressionStatus, duration_secs: f64) {
    if status.is_regression() {
        println!(
            "⚠️  PERFORMANCE WARNING: {} shows {} ({:.4}s)",
            name,
            status.description(),
            duration_secs
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Test the measure function with a simple async operation
    #[tokio::test]
    async fn test_measure_function() {
        // Create a simple async function that simulates work
        async fn simple_operation() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let result = measure("test_benchmark".to_string(), 5, || async {
            simple_operation().await
        })
        .await;

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.iterations, 5);
        assert!(result.duration_secs > 0.0, "Duration should be positive");
        assert!(result.duration_secs < 1.0, "Duration should be reasonable");
        assert!(result.throughput_ops_per_sec.is_none());
    }

    /// Test the measure_with_throughput function
    #[tokio::test]
    async fn test_measure_with_throughput_function() {
        // Create a simple async operation that processes 10 items
        async fn process_items() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        let result = measure_with_throughput("throughput_benchmark".to_string(), 3, 10, || async {
            process_items().await
        })
        .await;

        assert_eq!(result.name, "throughput_benchmark");
        assert_eq!(result.iterations, 3);
        assert!(result.duration_secs > 0.0, "Duration should be positive");
        assert!(
            result.throughput_ops_per_sec.is_some(),
            "Throughput should be calculated"
        );
        assert!(
            result.throughput_ops_per_sec.unwrap() > 0.0,
            "Throughput should be positive"
        );
    }

    /// Test assert_performance_threshold with passing case (duration only)
    #[test]
    fn test_assert_performance_threshold_passing_duration() {
        let result = BenchmarkResult::new("fast_operation".to_string(), 0.5, 10, None);
        let threshold = PerformanceThreshold::new(1.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(
            validation.is_ok(),
            "Should pass threshold: {:?}",
            validation.err()
        );
    }

    /// Test assert_performance_threshold with passing case (duration + throughput)
    #[test]
    fn test_assert_performance_threshold_passing_with_throughput() {
        let result = BenchmarkResult::new("efficient_operation".to_string(), 0.8, 10, Some(150.0));
        let threshold = PerformanceThreshold::with_throughput(1.0, 100.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(
            validation.is_ok(),
            "Should pass threshold: {:?}",
            validation.err()
        );
    }

    /// Test assert_performance_threshold failing on duration
    #[test]
    fn test_assert_performance_threshold_failing_duration() {
        let result = BenchmarkResult::new("slow_operation".to_string(), 2.5, 10, None);
        let threshold = PerformanceThreshold::new(1.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(validation.is_err(), "Should fail threshold on duration");

        let error_msg = validation.unwrap_err();
        assert!(
            error_msg.contains("Duration threshold failed"),
            "Error should mention duration failure"
        );
        assert!(
            error_msg.contains("slow_operation"),
            "Error should include benchmark name"
        );
    }

    /// Test assert_performance_threshold failing on throughput
    #[test]
    fn test_assert_performance_threshold_failing_throughput() {
        let result = BenchmarkResult::new("inefficient_operation".to_string(), 0.5, 10, Some(50.0));
        let threshold = PerformanceThreshold::with_throughput(1.0, 100.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(validation.is_err(), "Should fail threshold on throughput");

        let error_msg = validation.unwrap_err();
        assert!(
            error_msg.contains("Throughput threshold failed"),
            "Error should mention throughput failure"
        );
        assert!(
            error_msg.contains("inefficient_operation"),
            "Error should include benchmark name"
        );
    }

    /// Test assert_performance_threshold when result has no throughput data
    #[test]
    fn test_assert_performance_threshold_no_throughput_data() {
        let result = BenchmarkResult::new("no_throughput".to_string(), 0.5, 10, None);
        let threshold = PerformanceThreshold::with_throughput(1.0, 100.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(
            validation.is_err(),
            "Should fail when throughput expected but not provided"
        );

        let error_msg = validation.unwrap_err();
        assert!(
            error_msg.contains("Throughput threshold specified"),
            "Error should mention missing throughput data"
        );
    }

    /// Test print_benchmark_summary output format
    #[test]
    fn test_print_benchmark_summary() {
        let results = vec![
            BenchmarkResult::new("benchmark1".to_string(), 0.5, 10, Some(100.0)),
            BenchmarkResult::new("benchmark2".to_string(), 1.2, 5, None),
            BenchmarkResult::new(
                "very_long_benchmark_name".to_string(),
                2.3456,
                100,
                Some(1234.5678),
            ),
        ];

        // Note: In a real test, we'd capture stdout properly.
        // For now, we just verify the function doesn't panic.

        // Just verify it doesn't panic
        print_benchmark_summary(&results);

        // Verify results are properly structured
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "benchmark1");
        assert_eq!(results[0].throughput_ops_per_sec, Some(100.0));
        assert_eq!(results[1].throughput_ops_per_sec, None);
    }

    /// Test print_benchmark_summary with empty results
    #[test]
    fn test_print_benchmark_summary_empty() {
        let results: Vec<BenchmarkResult> = vec![];

        // Verify it doesn't panic with empty results
        print_benchmark_summary(&results);
    }

    /// Test BenchmarkResult::new constructor
    #[test]
    fn test_benchmark_result_new() {
        let result = BenchmarkResult::new("test".to_string(), 1.5, 20, Some(200.0));

        assert_eq!(result.name, "test");
        assert_eq!(result.duration_secs, 1.5);
        assert_eq!(result.iterations, 20);
        assert_eq!(result.throughput_ops_per_sec, Some(200.0));
    }

    /// Test PerformanceThreshold::new constructor
    #[test]
    fn test_performance_threshold_new() {
        let threshold = PerformanceThreshold::new(2.0);

        assert_eq!(threshold.max_duration_secs, 2.0);
        assert!(threshold.min_throughput_ops_per_sec.is_none());
    }

    /// Test PerformanceThreshold::with_throughput constructor
    #[test]
    fn test_performance_threshold_with_throughput() {
        let threshold = PerformanceThreshold::with_throughput(2.0, 50.0);

        assert_eq!(threshold.max_duration_secs, 2.0);
        assert_eq!(threshold.min_throughput_ops_per_sec, Some(50.0));
    }

    /// Test create_temp_dir helper
    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir();
        let path = temp_dir.path();

        assert!(path.exists(), "Temp dir path should exist");
        assert!(path.is_dir(), "Temp dir should be a directory");

        // Verify we can write to it
        let test_file = path.join("test.txt");
        std::fs::write(&test_file, "test content").expect("Should write to temp dir");
        assert!(test_file.exists(), "Test file should exist");

        // temp_dir will be cleaned up when dropped
    }

    /// Test set_env_var and restore_env_var helpers
    #[test]
    fn test_set_and_restore_env_var() {
        let test_key = "PERFORMANCE_TEST_VAR";
        let test_value = "test_value";

        // Clear the variable first if it exists
        std::env::remove_var(test_key);

        // Set the variable
        let (returned_value, guard) = set_env_var(test_key, test_value);

        assert_eq!(returned_value, test_value);
        assert_eq!(
            std::env::var(test_key).unwrap(),
            test_value,
            "Env var should be set"
        );

        // Restore the variable (which will remove it since it didn't exist before)
        restore_env_var(test_key, guard);

        assert!(
            std::env::var(test_key).is_err(),
            "Env var should be removed after restore"
        );
    }

    /// Test set_env_var and restore_env_var with existing value
    #[test]
    fn test_set_and_restore_env_var_with_existing() {
        let test_key = "PERFORMANCE_TEST_VAR_EXISTING";
        let original_value = "original_value";
        let test_value = "test_value";

        // Set an initial value
        std::env::set_var(test_key, original_value);

        // Set a new value
        let (returned_value, guard) = set_env_var(test_key, test_value);

        assert_eq!(returned_value, test_value);
        assert_eq!(
            std::env::var(test_key).unwrap(),
            test_value,
            "Env var should be set to new value"
        );

        // Restore the variable (should restore original value)
        restore_env_var(test_key, guard);

        assert_eq!(
            std::env::var(test_key).unwrap(),
            original_value,
            "Env var should be restored to original value"
        );

        // Clean up
        std::env::remove_var(test_key);
    }

    /// Test that multiple measurements work correctly
    #[tokio::test]
    async fn test_multiple_measurements() {
        let mut results = Vec::new();

        for i in 1..=3 {
            let result = measure(format!("benchmark_{}", i), 3, || async {
                tokio::time::sleep(Duration::from_millis(5)).await;
            })
            .await;
            results.push(result);
        }

        assert_eq!(results.len(), 3);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.name, format!("benchmark_{}", i + 1));
            assert_eq!(result.iterations, 3);
            assert!(result.duration_secs > 0.0);
        }
    }
}
=======
// Shared performance test infrastructure module
//
// This module provides reusable utilities for performance testing across
// all performance test files. It includes:
//
// - Benchmark result tracking with metrics
// - Performance threshold assertion
// - Benchmark measurement with warmup
// - Summary printing for visibility
// - Test setup helpers (temp dirs, env vars)

use std::future::Future;
use std::sync::Mutex;
use tempfile::TempDir;

/// Benchmark result containing timing and throughput metrics
///
/// This struct captures the results of a single benchmark run, including
/// the duration, number of iterations, and optional throughput metrics.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    /// Name of the benchmark
    pub name: String,
    /// Duration in seconds (average across all iterations)
    pub duration_secs: f64,
    /// Number of iterations performed
    pub iterations: u32,
    /// Optional throughput metric (operations per second)
    pub throughput_ops_per_sec: Option<f64>,
}

impl BenchmarkResult {
    /// Creates a new benchmark result
    pub fn new(
        name: String,
        duration_secs: f64,
        iterations: u32,
        throughput_ops_per_sec: Option<f64>,
    ) -> Self {
        Self {
            name,
            duration_secs,
            iterations,
            throughput_ops_per_sec,
        }
    }
}

/// Performance threshold for benchmark validation
///
/// This struct defines the acceptable performance bounds for a benchmark.
/// A benchmark passes if its duration is within the maximum allowed time
/// and (optionally) its throughput meets the minimum required value.
#[derive(Debug, Clone)]
pub struct PerformanceThreshold {
    /// Maximum allowed duration in seconds
    pub max_duration_secs: f64,
    /// Optional minimum required throughput (operations per second)
    pub min_throughput_ops_per_sec: Option<f64>,
}

impl PerformanceThreshold {
    /// Creates a new performance threshold with duration only
    pub fn new(max_duration_secs: f64) -> Self {
        Self {
            max_duration_secs,
            min_throughput_ops_per_sec: None,
        }
    }

    /// Creates a new performance threshold with both duration and throughput
    pub fn with_throughput(max_duration_secs: f64, min_throughput_ops_per_sec: f64) -> Self {
        Self {
            max_duration_secs,
            min_throughput_ops_per_sec: Some(min_throughput_ops_per_sec),
        }
    }
}

/// Measures the performance of an async operation across multiple iterations
///
/// This function runs the provided async function multiple times, measuring
/// the average duration. A warmup iteration is performed before timing starts
/// to account for JIT compilation and initialization effects.
///
/// # Arguments
///
/// * `name` - Name for the benchmark
/// * `iterations` - Number of timed iterations to perform
/// * `f` - Async function to benchmark
///
/// # Returns
///
/// A `BenchmarkResult` containing the average duration and iteration count
///
/// # Example
///
/// ```no_run
/// # use switchboard::some_async_function;
/// # async fn example() {
/// let result = performance_common::measure(
///     "my_benchmark".to_string(),
///     10,
///     || async { some_async_function().await }
/// ).await;
///
/// println!("{} completed in {:.4}s ({} iterations)",
///     result.name, result.duration_secs, result.iterations);
/// # }
/// ```
pub async fn measure<F, Fut>(name: String, iterations: u32, f: F) -> BenchmarkResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    // Warmup iteration (not timed)
    let _ = f().await;

    // Timed iterations
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f().await;
    }
    let total_duration = start.elapsed();

    let duration_secs = total_duration.as_secs_f64() / iterations as f64;

    BenchmarkResult::new(name, duration_secs, iterations, None)
}

/// Measures the performance of an async operation across multiple iterations with throughput
///
/// This function is similar to `measure` but also calculates throughput metrics.
/// The operation count parameter allows calculating operations per second.
///
/// # Arguments
///
/// * `name` - Name for the benchmark
/// * `iterations` - Number of timed iterations to perform
/// * `operations_per_iteration` - Number of operations performed in each iteration
/// * `f` - Async function to benchmark
///
/// # Returns
///
/// A `BenchmarkResult` containing the average duration, iteration count, and throughput
pub async fn measure_with_throughput<F, Fut>(
    name: String,
    iterations: u32,
    operations_per_iteration: u32,
    f: F,
) -> BenchmarkResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    // Warmup iteration (not timed)
    let _ = f().await;

    // Timed iterations
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        f().await;
    }
    let total_duration = start.elapsed();

    let duration_secs = total_duration.as_secs_f64() / iterations as f64;

    // Calculate throughput: operations per second
    let total_operations = operations_per_iteration as f64 * iterations as f64;
    let throughput_ops_per_sec = total_operations / total_duration.as_secs_f64();

    BenchmarkResult::new(
        name,
        duration_secs,
        iterations,
        Some(throughput_ops_per_sec),
    )
}

/// Asserts that a benchmark result meets the performance threshold
///
/// This function validates that the benchmark result is within acceptable
/// performance bounds. It checks both duration and (optionally) throughput.
///
/// # Arguments
///
/// * `result` - The benchmark result to validate
/// * `threshold` - The performance threshold to check against
///
/// # Returns
///
/// * `Ok(())` if all assertions pass
/// * `Err(String)` with a descriptive error message if any assertion fails
///
/// # Example
///
/// ```no_run
/// # use performance_common::{BenchmarkResult, PerformanceThreshold};
/// # fn example() {
/// let result = BenchmarkResult::new("test".to_string(), 1.5, 10, Some(100.0));
/// let threshold = PerformanceThreshold::new(2.0);
///
/// assert_performance_threshold(&result, &threshold)
///     .expect("Performance threshold not met");
/// # }
/// ```
pub fn assert_performance_threshold(
    result: &BenchmarkResult,
    threshold: &PerformanceThreshold,
) -> Result<(), String> {
    // Check duration threshold
    if result.duration_secs > threshold.max_duration_secs {
        return Err(format!(
            "Duration threshold failed: '{}' took {:.4}s, expected <= {:.4}s",
            result.name, result.duration_secs, threshold.max_duration_secs
        ));
    }

    // Check throughput threshold if specified
    if let Some(min_throughput) = threshold.min_throughput_ops_per_sec {
        match result.throughput_ops_per_sec {
            Some(actual_throughput) => {
                if actual_throughput < min_throughput {
                    return Err(format!(
                        "Throughput threshold failed: '{}' achieved {:.2} ops/sec, expected >= {:.2} ops/sec",
                        result.name, actual_throughput, min_throughput
                    ));
                }
            }
            None => {
                return Err(format!(
                    "Throughput threshold specified but result '{}' has no throughput data",
                    result.name
                ));
            }
        }
    }

    Ok(())
}

/// Prints a formatted summary table of benchmark results
///
/// This function displays all benchmark results in a readable table format,
/// showing the name, duration, iterations, and optional throughput metrics.
/// Useful for test output visibility and quick performance assessment.
///
/// # Arguments
///
/// * `results` - Slice of benchmark results to display
///
/// # Example
///
/// ```no_run
/// # use performance_common::{BenchmarkResult, print_benchmark_summary};
/// # fn example() {
/// let results = vec![
///     BenchmarkResult::new("benchmark1".to_string(), 0.5, 10, Some(100.0)),
///     BenchmarkResult::new("benchmark2".to_string(), 1.2, 10, None),
/// ];
/// print_benchmark_summary(&results);
/// # }
/// ```
pub fn print_benchmark_summary(results: &[BenchmarkResult]) {
    println!("\n=== Benchmark Summary ===\n");

    if results.is_empty() {
        println!("No benchmark results to display.\n");
        return;
    }

    // Calculate column widths
    let name_width = results
        .iter()
        .map(|r| r.name.len())
        .max()
        .unwrap_or("Name".len());
    let name_width = std::cmp::max(name_width, 4);

    // Print header
    println!(
        "{:<name_width$} | {:>10} | {:>10} | {:>15}",
        "Name", "Time (s)", "Iterations", "Throughput (ops/s)"
    );
    println!(
        "{:-<name_width$}-+-{:-<10}-+-{:-<10}-+-{:-<15}",
        "", "", "", ""
    );

    // Print each result
    for result in results {
        let throughput_str = match result.throughput_ops_per_sec {
            Some(tp) => format!("{:>15.2}", tp),
            None => String::from("           N/A"),
        };
        println!(
            "{:<name_width$} | {:>10.4} | {:>10} | {}",
            result.name, result.duration_secs, result.iterations, throughput_str
        );
    }

    println!();
}

/// Creates a temporary directory for test isolation
///
/// This function creates a new temporary directory that will be automatically
/// deleted when the returned `TempDir` value is dropped.
///
/// # Returns
///
/// A `TempDir` handle that automatically cleans up the directory when dropped
///
/// # Example
///
/// ```no_run
/// # use tempfile::TempDir;
/// # fn example() {
/// let temp_dir = performance_common::create_temp_dir();
/// let test_path = temp_dir.path().join("test.txt");
/// // Write test data...
/// // temp_dir is automatically cleaned up when it goes out of scope
/// # }
/// ```
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// Sets an environment variable and returns information to restore it
///
/// This function sets an environment variable and returns a tuple containing
/// the previous value and a mutex-protected cleanup guard. This ensures
/// the environment variable can be restored after the test completes.
///
/// # Arguments
///
/// * `key` - Environment variable name
/// * `value` - New value for the environment variable
///
/// # Returns
///
/// A tuple of (old_value, restore_guard) where restore_guard is a Mutex
/// containing the old value that can be used to restore the original value
///
/// # Example
///
/// ```no_run
/// # fn example() {
/// let (old_home, guard) = performance_common::set_env_var("HOME", "/tmp/test");
/// // Run test with modified environment...
/// // Restore original value when done
/// if let Some(old_val) = guard.lock().unwrap().take() {
///     std::env::set_var("HOME", old_val);
/// }
/// # }
/// ```
pub fn set_env_var(key: &str, value: &str) -> (String, Option<Mutex<Option<String>>>) {
    let old_value = std::env::var(key).ok();
    std::env::set_var(key, value);
    let restore_guard = Mutex::new(old_value);
    (value.to_string(), Some(restore_guard))
}

/// Restores an environment variable to its previous value
///
/// This function restores an environment variable to its original value
/// using the restore guard returned by `set_env_var`.
///
/// # Arguments
///
/// * `key` - Environment variable name
/// * `restore_guard` - Mutex containing the optional old value to restore
///
/// # Example
///
/// ```no_run
/// # fn example() {
/// let (old_home, guard) = performance_common::set_env_var("HOME", "/tmp/test");
/// // ... test code ...
/// performance_common::restore_env_var("HOME", guard);
/// # }
/// ```
pub fn restore_env_var(key: &str, restore_guard: Option<Mutex<Option<String>>>) {
    if let Some(guard) = restore_guard {
        let old_value = guard.lock().unwrap().take();
        match old_value {
            Some(val) => std::env::set_var(key, val),
            None => std::env::remove_var(key),
        }
    }
}

// ============================================================================
// Enhanced Timing Utilities with Microsecond Precision
// ============================================================================

/// High-precision timer for performance measurements
///
/// This struct provides microsecond-level precision for timing operations,
/// with additional features like lap timing and automatic duration formatting.
#[derive(Clone)]
pub struct Timer {
    name: String,
    start: std::time::Instant,
    laps: Vec<(String, std::time::Duration)>,
}

impl std::fmt::Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("name", &self.name)
            .field("laps", &self.laps)
            .finish()
    }
}

impl Timer {
    /// Creates a new timer with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: std::time::Instant::now(),
            laps: Vec::new(),
        }
    }

    /// Records a lap time with an optional label
    pub fn lap(&mut self, label: impl Into<String>) {
        let elapsed = self.start.elapsed();
        self.laps.push((label.into(), elapsed));
    }

    /// Returns the total elapsed time since timer creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }

    /// Returns elapsed time in microseconds for precise comparisons
    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    /// Returns elapsed time in seconds (f64) for benchmarking
    pub fn elapsed_secs(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    /// Returns the timer name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns recorded laps
    pub fn laps(&self) -> &[(String, std::time::Duration)] {
        &self.laps
    }

    /// Formats elapsed time in a human-readable format
    pub fn format_elapsed(&self) -> String {
        let elapsed = self.elapsed();
        format_duration(elapsed)
    }

    /// Creates a BenchmarkResult from this timer
    pub fn to_benchmark_result(&self, iterations: u32) -> BenchmarkResult {
        BenchmarkResult::new(
            self.name.clone(),
            self.elapsed_secs() / iterations.max(1) as f64,
            iterations,
            None,
        )
    }
}

/// Formats a duration in a human-readable format
///
/// Displays the duration with appropriate precision (microseconds for short
/// durations, milliseconds for medium, seconds for long).
pub fn format_duration(duration: std::time::Duration) -> String {
    let micros = duration.as_micros();
    if micros < 1_000 {
        format!("{}μs", micros)
    } else if micros < 1_000_000 {
        format!("{:.2}ms", micros as f64 / 1_000.0)
    } else {
        format!("{:.4}s", duration.as_secs_f64())
    }
}

/// Convenience function to quickly time an async operation
///
/// This is a simpler alternative to `measure` that returns the duration
/// directly without wrapping it in a BenchmarkResult.
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// let duration = performance_common::time_async("operation", || async {
///     // async operation
/// }).await;
/// println!("Operation took {:?}", duration);
/// # }
/// ```
pub async fn time_async<F, Fut>(name: &str, f: F) -> std::time::Duration
where
    F: Fn() -> Fut,
    Fut: Future<Output = ()>,
{
    let mut timer = Timer::new(name);
    f().await;
    timer.elapsed()
}

// ============================================================================
// Setup/Teardown Infrastructure for Performance Tests
// ============================================================================

/// Setup and teardown context for performance tests
///
/// This struct manages the test environment, ensuring proper isolation
/// and cleanup between performance test runs.
pub struct PerformanceTestSetup {
    /// Temporary directory for test isolation (if created)
    temp_dir: Option<TempDir>,
    /// Environment variables to restore (key -> original value)
    env_vars: Vec<(String, Option<String>)>,
    /// Custom cleanup functions (not included in Debug)
    #[allow(dead_code)]
    cleanup_fns: Vec<Box<dyn Fn() + Send>>,
}

impl std::fmt::Debug for PerformanceTestSetup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceTestSetup")
            .field("temp_dir", &self.temp_dir)
            .field("env_vars", &self.env_vars)
            .finish()
    }
}

impl PerformanceTestSetup {
    /// Creates a new empty test setup
    pub fn new() -> Self {
        Self {
            temp_dir: None,
            env_vars: Vec::new(),
            cleanup_fns: Vec::new(),
        }
    }

    /// Creates a new test setup with a temporary directory
    pub fn with_temp_dir(temp_dir: TempDir) -> Self {
        Self {
            temp_dir: Some(temp_dir),
            env_vars: Vec::new(),
            cleanup_fns: Vec::new(),
        }
    }

    /// Sets an environment variable, storing the original value for restoration
    pub fn set_env(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let key = key.into();
        let value = value.into();
        let old_value = std::env::var(&key).ok();
        std::env::set_var(&key, value);
        self.env_vars.push((key, old_value));
    }

    /// Adds a custom cleanup function to be called during teardown
    pub fn add_cleanup<F>(&mut self, cleanup_fn: F)
    where
        F: Fn() + Send + 'static,
    {
        self.cleanup_fns.push(Box::new(cleanup_fn));
    }

    /// Returns the temporary directory path if one was created
    pub fn temp_dir(&self) -> Option<&std::path::Path> {
        self.temp_dir.as_ref().map(|td| td.path())
    }

    /// Performs teardown, restoring all environment variables and running cleanup
    pub fn teardown(self) {
        // Run custom cleanup functions first
        for cleanup_fn in self.cleanup_fns {
            cleanup_fn();
        }

        // Restore environment variables
        for (key, old_value) in self.env_vars {
            match old_value {
                Some(val) => std::env::set_var(&key, val),
                None => std::env::remove_var(&key),
            }
        }

        // TempDir is automatically cleaned up when dropped
    }
}

impl Default for PerformanceTestSetup {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a standard performance test setup with a fresh temporary directory
///
/// This is a convenience function for creating a standardized test environment.
///
/// # Example
///
/// ```no_run
/// # async fn example() {
/// let setup = performance_common::setup_performance_test();
/// let temp_dir = setup.temp_dir().unwrap();
///
/// // ... run performance test ...
///
/// setup.teardown();  // Automatically cleans up
/// # }
/// ```
pub fn setup_performance_test() -> PerformanceTestSetup {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    PerformanceTestSetup::with_temp_dir(temp_dir)
}

// ============================================================================
// Baseline Metrics Tracker
// ============================================================================

/// Baseline metric for a single test
///
/// Stores the baseline performance data for a specific test, including
/// the expected duration and optional throughput targets.
#[derive(Debug, Clone)]
pub struct BaselineMetric {
    /// Test name
    pub name: String,
    /// Expected maximum duration in seconds
    pub max_duration_secs: f64,
    /// Expected average duration in seconds
    pub avg_duration_secs: f64,
    /// Optional minimum throughput (ops/sec)
    pub min_throughput: Option<f64>,
    /// Number of samples in baseline
    pub sample_count: u32,
    /// Standard deviation of baseline samples
    pub std_dev_secs: f64,
}

impl BaselineMetric {
    /// Creates a new baseline metric
    pub fn new(name: impl Into<String>, avg_duration_secs: f64) -> Self {
        Self {
            name: name.into(),
            max_duration_secs: avg_duration_secs * 1.5, // 50% tolerance by default
            avg_duration_secs,
            min_throughput: None,
            sample_count: 1,
            std_dev_secs: 0.0,
        }
    }

    /// Creates a baseline metric with custom max duration
    pub fn with_max_duration(mut self, max_secs: f64) -> Self {
        self.max_duration_secs = max_secs;
        self
    }

    /// Creates a baseline metric with throughput target
    pub fn with_throughput(mut self, min_throughput: f64) -> Self {
        self.min_throughput = Some(min_throughput);
        self
    }

    /// Updates the baseline with a new sample
    pub fn update_with_sample(&mut self, duration_secs: f64) {
        // Calculate running mean and std dev using Welford's algorithm
        self.sample_count += 1;
        let delta = duration_secs - self.avg_duration_secs;
        self.avg_duration_secs += delta / self.sample_count as f64;
        let delta2 = duration_secs - self.avg_duration_secs;
        // Approximate standard deviation
        self.std_dev_secs =
            ((self.std_dev_secs.powi(2) * (self.sample_count - 1) as f64) + delta * delta2).sqrt()
                / (self.sample_count - 1).max(1) as f64;

        // Update max to be 50% above average or keep existing if lower
        let new_max = self.avg_duration_secs * 1.5;
        if new_max > self.max_duration_secs {
            self.max_duration_secs = new_max;
        }
    }

    /// Checks if a duration is within the baseline bounds
    pub fn is_within_bounds(&self, duration_secs: f64) -> bool {
        duration_secs <= self.max_duration_secs
    }

    /// Returns regression status
    pub fn check_regression(&self, duration_secs: f64) -> RegressionStatus {
        let ratio = duration_secs / self.avg_duration_secs;
        if ratio > 2.0 {
            RegressionStatus::SignificantRegression
        } else if ratio > 1.3 {
            RegressionStatus::MinorRegression
        } else if ratio < 0.7 {
            RegressionStatus::Improvement
        } else {
            RegressionStatus::Stable
        }
    }
}

/// Regression status indicating whether performance has degraded
#[derive(Debug, Clone, PartialEq)]
pub enum RegressionStatus {
    /// Performance is stable (within 30% of baseline)
    Stable,
    /// Minor regression (30-100% slower than baseline)
    MinorRegression,
    /// Significant regression (more than 2x slower than baseline)
    SignificantRegression,
    /// Performance has improved (more than 30% faster)
    Improvement,
}

impl RegressionStatus {
    /// Returns true if this status indicates a regression
    pub fn is_regression(&self) -> bool {
        matches!(
            self,
            RegressionStatus::MinorRegression | RegressionStatus::SignificantRegression
        )
    }

    /// Returns a human-readable description
    pub fn description(&self) -> &str {
        match self {
            RegressionStatus::Stable => "stable",
            RegressionStatus::MinorRegression => "minor regression",
            RegressionStatus::SignificantRegression => "significant regression",
            RegressionStatus::Improvement => "improvement",
        }
    }
}

/// Baseline tracker that records and compares performance metrics
///
/// This struct manages baseline metrics for multiple tests and provides
/// functionality to check for regressions.
#[derive(Debug, Default)]
pub struct BaselineTracker {
    metrics: std::collections::HashMap<String, BaselineMetric>,
    results: Vec<BenchmarkResult>,
}

impl BaselineTracker {
    /// Creates a new baseline tracker
    pub fn new() -> Self {
        Self {
            metrics: std::collections::HashMap::new(),
            results: Vec::new(),
        }
    }

    /// Records a baseline metric for a test
    pub fn record_baseline(&mut self, metric: BaselineMetric) {
        let name = metric.name.clone();
        self.metrics.insert(name, metric);
    }

    /// Records a benchmark result
    pub fn record_result(&mut self, result: BenchmarkResult) {
        // Update baseline if it exists
        if let Some(baseline) = self.metrics.get_mut(&result.name) {
            baseline.update_with_sample(result.duration_secs);
        }
        self.results.push(result);
    }

    /// Checks if a result shows regression compared to baseline
    pub fn check_regression(&self, name: &str, duration_secs: f64) -> Option<RegressionStatus> {
        self.metrics
            .get(name)
            .map(|b| b.check_regression(duration_secs))
    }

    /// Gets the baseline metric for a test
    pub fn get_baseline(&self, name: &str) -> Option<&BaselineMetric> {
        self.metrics.get(name)
    }

    /// Returns all recorded results
    pub fn results(&self) -> &[BenchmarkResult] {
        &self.results
    }

    /// Saves baseline metrics to a file (simple line-based format)
    ///
    /// Format: name|avg_duration|max_duration|sample_count|std_dev
    pub fn save_baseline_to_file(
        metrics: &std::collections::HashMap<String, BaselineMetric>,
        path: &std::path::Path,
    ) -> std::io::Result<()> {
        let mut content = String::new();
        for (name, metric) in metrics {
            content.push_str(&format!(
                "{}|{}|{}|{}|{}\n",
                name,
                metric.avg_duration_secs,
                metric.max_duration_secs,
                metric.sample_count,
                metric.std_dev_secs
            ));
        }
        std::fs::write(path, content)
    }

    /// Loads baseline metrics from a file
    pub fn load_baseline_from_file(
        path: &std::path::Path,
    ) -> std::io::Result<std::collections::HashMap<String, BaselineMetric>> {
        let content = std::fs::read_to_string(path)?;
        let mut metrics = std::collections::HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                let name = parts[0].to_string();
                let avg_duration_secs: f64 = parts[1].parse().unwrap_or(0.0);
                let max_duration_secs: f64 = parts[2].parse().unwrap_or(avg_duration_secs * 1.5);
                let sample_count: u32 = parts[3].parse().unwrap_or(1);
                let std_dev_secs: f64 = parts[4].parse().unwrap_or(0.0);

                metrics.insert(
                    name,
                    BaselineMetric {
                        name: parts[0].to_string(),
                        avg_duration_secs,
                        max_duration_secs,
                        min_throughput: None,
                        sample_count,
                        std_dev_secs,
                    },
                );
            }
        }

        Ok(metrics)
    }

    /// Prints a summary of baseline metrics
    pub fn print_summary(&self) {
        println!("\n=== Baseline Metrics Summary ===\n");
        if self.metrics.is_empty() {
            println!("No baseline metrics recorded.\n");
            return;
        }

        for (name, metric) in &self.metrics {
            println!("Test: {}", name);
            println!("  Average: {:.4}s", metric.avg_duration_secs);
            println!("  Max (threshold): {:.4}s", metric.max_duration_secs);
            println!("  Std Dev: {:.4}s", metric.std_dev_secs);
            println!("  Samples: {}", metric.sample_count);
            if let Some(tp) = metric.min_throughput {
                println!("  Min Throughput: {:.2} ops/s", tp);
            }
            println!();
        }
    }
}

/// Detects if performance has regressed compared to baseline
///
/// This function compares a new measurement against the baseline and returns
/// a regression status. If no baseline exists, it creates one.
///
/// # Arguments
///
/// * `tracker` - The baseline tracker to use
/// * `name` - The test name
/// * `duration_secs` - The measured duration in seconds
/// * `create_baseline_if_missing` - Whether to create a baseline if none exists
///
/// # Returns
///
/// A tuple of (regression_status, created_baseline)
pub fn detect_regression(
    tracker: &mut BaselineTracker,
    name: &str,
    duration_secs: f64,
    create_baseline_if_missing: bool,
) -> (Option<RegressionStatus>, bool) {
    let status = tracker.check_regression(name, duration_secs);

    if let Some(s) = status {
        (Some(s), false)
    } else if create_baseline_if_missing {
        let baseline = BaselineMetric::new(name, duration_secs);
        tracker.record_baseline(baseline);
        (None, true)
    } else {
        (None, false)
    }
}

/// Prints a regression warning if performance has degraded
///
/// This function logs a warning message if the regression status indicates
/// performance degradation.
pub fn log_regression_warning(name: &str, status: &RegressionStatus, duration_secs: f64) {
    if status.is_regression() {
        println!(
            "⚠️  PERFORMANCE WARNING: {} shows {} ({:.4}s)",
            name,
            status.description(),
            duration_secs
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Test the measure function with a simple async operation
    #[tokio::test]
    async fn test_measure_function() {
        // Create a simple async function that simulates work
        async fn simple_operation() {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let result = measure("test_benchmark".to_string(), 5, || async {
            simple_operation().await
        })
        .await;

        assert_eq!(result.name, "test_benchmark");
        assert_eq!(result.iterations, 5);
        assert!(result.duration_secs > 0.0, "Duration should be positive");
        assert!(result.duration_secs < 1.0, "Duration should be reasonable");
        assert!(result.throughput_ops_per_sec.is_none());
    }

    /// Test the measure_with_throughput function
    #[tokio::test]
    async fn test_measure_with_throughput_function() {
        // Create a simple async operation that processes 10 items
        async fn process_items() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        let result = measure_with_throughput("throughput_benchmark".to_string(), 3, 10, || async {
            process_items().await
        })
        .await;

        assert_eq!(result.name, "throughput_benchmark");
        assert_eq!(result.iterations, 3);
        assert!(result.duration_secs > 0.0, "Duration should be positive");
        assert!(
            result.throughput_ops_per_sec.is_some(),
            "Throughput should be calculated"
        );
        assert!(
            result.throughput_ops_per_sec.unwrap() > 0.0,
            "Throughput should be positive"
        );
    }

    /// Test assert_performance_threshold with passing case (duration only)
    #[test]
    fn test_assert_performance_threshold_passing_duration() {
        let result = BenchmarkResult::new("fast_operation".to_string(), 0.5, 10, None);
        let threshold = PerformanceThreshold::new(1.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(
            validation.is_ok(),
            "Should pass threshold: {:?}",
            validation.err()
        );
    }

    /// Test assert_performance_threshold with passing case (duration + throughput)
    #[test]
    fn test_assert_performance_threshold_passing_with_throughput() {
        let result = BenchmarkResult::new("efficient_operation".to_string(), 0.8, 10, Some(150.0));
        let threshold = PerformanceThreshold::with_throughput(1.0, 100.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(
            validation.is_ok(),
            "Should pass threshold: {:?}",
            validation.err()
        );
    }

    /// Test assert_performance_threshold failing on duration
    #[test]
    fn test_assert_performance_threshold_failing_duration() {
        let result = BenchmarkResult::new("slow_operation".to_string(), 2.5, 10, None);
        let threshold = PerformanceThreshold::new(1.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(validation.is_err(), "Should fail threshold on duration");

        let error_msg = validation.unwrap_err();
        assert!(
            error_msg.contains("Duration threshold failed"),
            "Error should mention duration failure"
        );
        assert!(
            error_msg.contains("slow_operation"),
            "Error should include benchmark name"
        );
    }

    /// Test assert_performance_threshold failing on throughput
    #[test]
    fn test_assert_performance_threshold_failing_throughput() {
        let result = BenchmarkResult::new("inefficient_operation".to_string(), 0.5, 10, Some(50.0));
        let threshold = PerformanceThreshold::with_throughput(1.0, 100.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(validation.is_err(), "Should fail threshold on throughput");

        let error_msg = validation.unwrap_err();
        assert!(
            error_msg.contains("Throughput threshold failed"),
            "Error should mention throughput failure"
        );
        assert!(
            error_msg.contains("inefficient_operation"),
            "Error should include benchmark name"
        );
    }

    /// Test assert_performance_threshold when result has no throughput data
    #[test]
    fn test_assert_performance_threshold_no_throughput_data() {
        let result = BenchmarkResult::new("no_throughput".to_string(), 0.5, 10, None);
        let threshold = PerformanceThreshold::with_throughput(1.0, 100.0);

        let validation = assert_performance_threshold(&result, &threshold);
        assert!(
            validation.is_err(),
            "Should fail when throughput expected but not provided"
        );

        let error_msg = validation.unwrap_err();
        assert!(
            error_msg.contains("Throughput threshold specified"),
            "Error should mention missing throughput data"
        );
    }

    /// Test print_benchmark_summary output format
    #[test]
    fn test_print_benchmark_summary() {
        let results = vec![
            BenchmarkResult::new("benchmark1".to_string(), 0.5, 10, Some(100.0)),
            BenchmarkResult::new("benchmark2".to_string(), 1.2, 5, None),
            BenchmarkResult::new(
                "very_long_benchmark_name".to_string(),
                2.3456,
                100,
                Some(1234.5678),
            ),
        ];

        // Note: In a real test, we'd capture stdout properly.
        // For now, we just verify the function doesn't panic.

        // Just verify it doesn't panic
        print_benchmark_summary(&results);

        // Verify results are properly structured
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].name, "benchmark1");
        assert_eq!(results[0].throughput_ops_per_sec, Some(100.0));
        assert_eq!(results[1].throughput_ops_per_sec, None);
    }

    /// Test print_benchmark_summary with empty results
    #[test]
    fn test_print_benchmark_summary_empty() {
        let results: Vec<BenchmarkResult> = vec![];

        // Verify it doesn't panic with empty results
        print_benchmark_summary(&results);
    }

    /// Test BenchmarkResult::new constructor
    #[test]
    fn test_benchmark_result_new() {
        let result = BenchmarkResult::new("test".to_string(), 1.5, 20, Some(200.0));

        assert_eq!(result.name, "test");
        assert_eq!(result.duration_secs, 1.5);
        assert_eq!(result.iterations, 20);
        assert_eq!(result.throughput_ops_per_sec, Some(200.0));
    }

    /// Test PerformanceThreshold::new constructor
    #[test]
    fn test_performance_threshold_new() {
        let threshold = PerformanceThreshold::new(2.0);

        assert_eq!(threshold.max_duration_secs, 2.0);
        assert!(threshold.min_throughput_ops_per_sec.is_none());
    }

    /// Test PerformanceThreshold::with_throughput constructor
    #[test]
    fn test_performance_threshold_with_throughput() {
        let threshold = PerformanceThreshold::with_throughput(2.0, 50.0);

        assert_eq!(threshold.max_duration_secs, 2.0);
        assert_eq!(threshold.min_throughput_ops_per_sec, Some(50.0));
    }

    /// Test create_temp_dir helper
    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir();
        let path = temp_dir.path();

        assert!(path.exists(), "Temp dir path should exist");
        assert!(path.is_dir(), "Temp dir should be a directory");

        // Verify we can write to it
        let test_file = path.join("test.txt");
        std::fs::write(&test_file, "test content").expect("Should write to temp dir");
        assert!(test_file.exists(), "Test file should exist");

        // temp_dir will be cleaned up when dropped
    }

    /// Test set_env_var and restore_env_var helpers
    #[test]
    fn test_set_and_restore_env_var() {
        let test_key = "PERFORMANCE_TEST_VAR";
        let test_value = "test_value";

        // Clear the variable first if it exists
        std::env::remove_var(test_key);

        // Set the variable
        let (returned_value, guard) = set_env_var(test_key, test_value);

        assert_eq!(returned_value, test_value);
        assert_eq!(
            std::env::var(test_key).unwrap(),
            test_value,
            "Env var should be set"
        );

        // Restore the variable (which will remove it since it didn't exist before)
        restore_env_var(test_key, guard);

        assert!(
            std::env::var(test_key).is_err(),
            "Env var should be removed after restore"
        );
    }

    /// Test set_env_var and restore_env_var with existing value
    #[test]
    fn test_set_and_restore_env_var_with_existing() {
        let test_key = "PERFORMANCE_TEST_VAR_EXISTING";
        let original_value = "original_value";
        let test_value = "test_value";

        // Set an initial value
        std::env::set_var(test_key, original_value);

        // Set a new value
        let (returned_value, guard) = set_env_var(test_key, test_value);

        assert_eq!(returned_value, test_value);
        assert_eq!(
            std::env::var(test_key).unwrap(),
            test_value,
            "Env var should be set to new value"
        );

        // Restore the variable (should restore original value)
        restore_env_var(test_key, guard);

        assert_eq!(
            std::env::var(test_key).unwrap(),
            original_value,
            "Env var should be restored to original value"
        );

        // Clean up
        std::env::remove_var(test_key);
    }

    /// Test that multiple measurements work correctly
    #[tokio::test]
    async fn test_multiple_measurements() {
        let mut results = Vec::new();

        for i in 1..=3 {
            let result = measure(format!("benchmark_{}", i), 3, || async {
                tokio::time::sleep(Duration::from_millis(5)).await;
            })
            .await;
            results.push(result);
        }

        assert_eq!(results.len(), 3);
        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.name, format!("benchmark_{}", i + 1));
            assert_eq!(result.iterations, 3);
            assert!(result.duration_secs > 0.0);
        }
    }
}
>>>>>>> skills-improvements
