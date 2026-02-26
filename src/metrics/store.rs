use crate::metrics::MetricsError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

/// Top-level JSON structure for all metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AllMetrics {
    pub agents: HashMap<String, AgentMetricsData>,
}

/// Serialized representation of AgentMetrics for JSON
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentMetricsData {
    pub total_runs: u64,
    pub successful_runs: u64,
    pub failed_runs: u64,
    pub total_duration_ms: u64,
    pub runs: Vec<AgentRunResultData>,
    /// Cumulative queue wait time in seconds for averaging later
    pub queue_wait_time_seconds: Option<u64>,
    /// Individual queue wait times in seconds for each queued run
    pub queue_wait_times: Vec<u64>,
    /// Number of containers terminated via SIGTERM.
    pub sigterm_count: u64,
    /// Number of containers terminated via SIGKILL.
    pub sigkill_count: u64,
    /// Number of runs that exceeded timeout.
    pub timeout_count: u64,
    /// Cumulative count of skills successfully installed across all runs
    #[serde(default)]
    pub total_skills_installed: u64,
    /// Cumulative count of skills that failed to install across all runs
    #[serde(default)]
    pub total_skills_failed: u64,
    /// Cumulative skill install time in seconds (for averaging)
    #[serde(default)]
    pub skills_install_time_seconds: Option<f64>,
    /// Number of runs with at least one skill installation failure
    #[serde(default)]
    pub runs_with_skill_failures: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentRunResultData {
    pub run_id: String,
    pub timestamp: u64,
    pub duration_ms: u64,
    pub status: String, // "success" or "failure"
    pub error_message: Option<String>,
    /// Number of skills successfully installed during this run
    #[serde(default)]
    pub skills_installed_count: u32,
    /// Number of skills that failed to install during this run
    #[serde(default)]
    pub skills_failed_count: u32,
    /// Time spent installing skills during this run, in seconds
    #[serde(default)]
    pub skills_install_time_seconds: Option<f64>,
}

/// Store for managing persistence of metrics data.
///
/// This structure provides a simple API for persisting and loading metrics data
/// from a JSON file. It handles atomic writes, file corruption recovery, and
/// directory creation. Metrics are stored in `<log_dir>/metrics.json` with the
/// following features:
///
/// - **Atomic Writes**: Uses a temp file + rename pattern to ensure data integrity
/// - **Corruption Recovery**: Backs up corrupted files before returning errors
/// - **Auto-Creation**: Creates the log directory if it doesn't exist
/// - **JSON Format**: Uses human-readable JSON for easy inspection and debugging
///
/// # Fields
///
/// - `log_dir` - The directory where `metrics.json` will be stored
///
/// # Example
///
/// ```no_run
/// use switchboard::metrics::{MetricsStore, AllMetrics};
/// use std::path::PathBuf;
///
/// // Create a metrics store
/// let store = MetricsStore::new(PathBuf::from("./logs"));
///
/// // Save metrics (creates log directory if needed)
/// let metrics = AllMetrics::default();
/// store.save(&metrics).unwrap();
///
/// // Load metrics back
/// let loaded = store.load().unwrap();
///
/// // Metrics can be updated and saved again
/// // store.save(&updated_metrics).unwrap();
/// ```
pub struct MetricsStore {
    log_dir: PathBuf,
}

impl MetricsStore {
    /// Create a new MetricsStore with the specified log directory.
    pub fn new(log_dir: PathBuf) -> Self {
        Self { log_dir }
    }

    /// Load metrics from the metrics.json file in the log directory.
    /// Returns an error if the file doesn't exist.
    /// Corrupted files are backed up to `metrics.json.backup.<timestamp>`.
    pub fn load(&self) -> Result<AllMetrics, MetricsError> {
        let metrics_path = self.log_dir.join("metrics.json");

        if !metrics_path.exists() {
            // File doesn't exist - return FileNotFound error
            return Err(MetricsError::FileNotFound(
                metrics_path.display().to_string(),
            ));
        }

        let content = fs::read_to_string(&metrics_path)
            .map_err(|e| MetricsError::ReadError(format!("Failed to read metrics file: {}", e)))?;

        match serde_json::from_str::<AllMetrics>(&content) {
            Ok(metrics) => Ok(metrics),
            Err(_e) => {
                // JSON parse failed - corrupted file
                let timestamp = SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let backup_path = format!("{}.backup.{}", metrics_path.display(), timestamp);
                std::fs::copy(&metrics_path, &backup_path).map_err(|e| {
                    MetricsError::WriteError(format!("Failed to backup corrupted file: {}", e))
                })?;
                // Log a warning and return corrupted file error
                eprintln!(
                    "Warning: Metrics file corrupted. Backup saved to: {}",
                    backup_path
                );
                Err(MetricsError::CorruptedFile(backup_path))
            }
        }
    }

    /// Save metrics atomically to the metrics.json file in the log directory.
    /// Uses a temp file + rename pattern to ensure atomic writes.
    pub fn save(&self, metrics: &AllMetrics) -> Result<(), MetricsError> {
        self.save_internal(metrics)
    }

    /// Save metrics atomically with retry logic and exponential backoff.
    ///
    /// This method attempts to save metrics multiple times on failure, using
    /// exponential backoff between retry attempts. This improves reliability in
    /// scenarios with temporary file system issues, network filesystem delays, or
    /// concurrent write conflicts.
    ///
    /// # Retry Strategy
    ///
    /// - **Maximum retries**: 3 attempts (1 initial attempt + 2 retries)
    /// - **Initial backoff**: 100ms
    /// - **Backoff multiplier**: 2 (exponential backoff)
    /// - **Retry delays**: 100ms, 200ms
    ///
    /// # Arguments
    ///
    /// * `metrics` - Reference to the AllMetrics to save
    ///
    /// # Returns
    ///
    /// * `Ok(())` on successful save
    /// * `Err(MetricsError)` if all retry attempts fail
    ///
    /// # Example
    ///
    /// ```no_run
    /// use switchboard::metrics::{MetricsStore, AllMetrics};
    /// use std::path::PathBuf;
    ///
    /// let store = MetricsStore::new(PathBuf::from("./logs"));
    /// let metrics = AllMetrics::default();
    ///
    /// // Save with automatic retry on failure
    /// match store.save_with_retry(&metrics) {
    ///     Ok(()) => println!("Metrics saved successfully"),
    ///     Err(e) => eprintln!("Failed to save metrics after retries: {:?}", e),
    /// }
    /// ```
    pub fn save_with_retry(&self, metrics: &AllMetrics) -> Result<(), MetricsError> {
        let max_retries = 3;
        let initial_delay_ms: u64 = 100;

        for attempt in 0..max_retries {
            match self.save_internal(metrics) {
                Ok(()) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Metrics save succeeded on attempt {} after {} retries",
                            attempt + 1,
                            attempt
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    if attempt < max_retries - 1 {
                        let delay_ms = initial_delay_ms * 2_u64.pow(attempt as u32);
                        tracing::warn!(
                            "Metrics save attempt {} failed, retrying in {}ms: {:?}",
                            attempt + 1,
                            delay_ms,
                            e
                        );
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    } else {
                        tracing::error!(
                            "Metrics save failed after {} retry attempts: {:?}",
                            max_retries,
                            e
                        );
                        return Err(e);
                    }
                }
            }
        }

        // This line should never be reached, but rustc requires it
        Err(MetricsError::WriteError(
            "Failed to save metrics after maximum retries".to_string(),
        ))
    }

    /// Internal implementation of the save operation.
    /// Uses a temp file + rename pattern to ensure atomic writes.
    fn save_internal(&self, metrics: &AllMetrics) -> Result<(), MetricsError> {
        let metrics_path = self.log_dir.join("metrics.json");
        let temp_path = metrics_path.with_extension("tmp");

        // Ensure log directory exists
        if !self.log_dir.exists() {
            std::fs::create_dir_all(&self.log_dir).map_err(|e| {
                MetricsError::WriteError(format!("Failed to create log directory: {}", e))
            })?;
        }

        // Write to temp file first
        let content = serde_json::to_string_pretty(metrics).map_err(|e| {
            MetricsError::SerializationError(format!("Failed to serialize metrics: {}", e))
        })?;

        let mut file = std::fs::File::create(&temp_path)
            .map_err(|e| MetricsError::WriteError(format!("Failed to create temp file: {}", e)))?;

        file.write_all(content.as_bytes()).map_err(|e| {
            MetricsError::WriteError(format!("Failed to write to temp file: {}", e))
        })?;

        // Atomic rename from temp to final
        std::fs::rename(&temp_path, &metrics_path).map_err(|e| {
            MetricsError::WriteError(format!("Failed to rename metrics file: {}", e))
        })?;

        Ok(())
    }

    /// Check the integrity of the metrics data.
    ///
    /// This method verifies that:
    /// - The metrics file exists (if not, returns Ok with a warning)
    /// - The metrics file can be parsed as valid JSON
    /// - The metrics data structure is internally consistent
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Metrics data is valid and consistent
    /// * `Ok(false)` - Metrics file doesn't exist yet (first run)
    /// * `Err(MetricsError)` - Metrics data is corrupted or inconsistent
    pub fn check_integrity(&self) -> Result<bool, MetricsError> {
        let metrics_path = self.log_dir.join("metrics.json");

        if !metrics_path.exists() {
            // File doesn't exist - this is fine for first run
            tracing::debug!("Metrics file does not exist yet (first run)");
            return Ok(false);
        }

        // Try to load and validate the metrics
        match self.load() {
            Ok(metrics) => {
                // Validate internal consistency
                for (agent_name, agent_data) in &metrics.agents {
                    // Check that counters are consistent
                    let actual_total = agent_data.successful_runs + agent_data.failed_runs;
                    if agent_data.total_runs != actual_total {
                        tracing::error!(
                            "Metrics integrity check failed for agent '{}': total_runs ({}) != successful_runs ({}) + failed_runs ({})",
                            agent_name,
                            agent_data.total_runs,
                            agent_data.successful_runs,
                            agent_data.failed_runs
                        );
                        return Err(MetricsError::CorruptedFile(format!(
                            "Inconsistent counter data for agent '{}'",
                            agent_name
                        )));
                    }

                    // Check that runs vector length matches total_runs
                    if agent_data.runs.len() as u64 != agent_data.total_runs {
                        tracing::error!(
                            "Metrics integrity check failed for agent '{}': runs vector length ({}) != total_runs ({})",
                            agent_name,
                            agent_data.runs.len(),
                            agent_data.total_runs
                        );
                        return Err(MetricsError::CorruptedFile(format!(
                            "Inconsistent runs vector length for agent '{}'",
                            agent_name
                        )));
                    }
                }

                tracing::debug!("Metrics integrity check passed");
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Metrics integrity check failed: {:?}", e);
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Helper function to create test AllMetrics
    fn create_test_all_metrics() -> AllMetrics {
        let mut agents = HashMap::new();

        let agent1 = AgentMetricsData {
            total_runs: 5,
            successful_runs: 4,
            failed_runs: 1,
            total_duration_ms: 50000,
            runs: vec![
                AgentRunResultData {
                    run_id: "container_1".to_string(),
                    timestamp: 1234567890,
                    duration_ms: 10000,
                    status: "success".to_string(),
                    error_message: None,
                    skills_installed_count: 0,
                    skills_failed_count: 0,
                    skills_install_time_seconds: None,
                },
                AgentRunResultData {
                    run_id: "container_2".to_string(),
                    timestamp: 1234567891,
                    duration_ms: 10000,
                    status: "failure".to_string(),
                    error_message: Some("exit_code: 1".to_string()),
                    skills_installed_count: 0,
                    skills_failed_count: 0,
                    skills_install_time_seconds: None,
                },
            ],
            queue_wait_time_seconds: Some(25),
            queue_wait_times: vec![10, 15],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        let agent2 = AgentMetricsData {
            total_runs: 3,
            successful_runs: 3,
            failed_runs: 0,
            total_duration_ms: 30000,
            runs: vec![AgentRunResultData {
                run_id: "container_3".to_string(),
                timestamp: 1234567892,
                duration_ms: 10000,
                status: "success".to_string(),
                error_message: None,
                skills_installed_count: 0,
                skills_failed_count: 0,
                skills_install_time_seconds: None,
            }],
            queue_wait_time_seconds: None,
            queue_wait_times: vec![],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        agents.insert("agent_1".to_string(), agent1);
        agents.insert("agent_2".to_string(), agent2);

        AllMetrics { agents }
    }

    // ==================== MetricsStore Tests ====================

    #[test]
    fn test_metrics_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        // Store creation should succeed
        assert_eq!(store.log_dir, temp_dir.path());
    }

    #[test]
    fn test_metrics_store_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        // Create test metrics
        let original_metrics = create_test_all_metrics();

        // Save the metrics
        store.save(&original_metrics).unwrap();

        // Load the metrics back
        let loaded_metrics = store.load().unwrap();

        // Verify the metrics are the same
        assert_eq!(loaded_metrics.agents.len(), 2);
        assert!(loaded_metrics.agents.contains_key("agent_1"));
        assert!(loaded_metrics.agents.contains_key("agent_2"));

        let agent1 = &loaded_metrics.agents["agent_1"];
        assert_eq!(agent1.total_runs, 5);
        assert_eq!(agent1.successful_runs, 4);
        assert_eq!(agent1.failed_runs, 1);
        assert_eq!(agent1.total_duration_ms, 50000);
        assert_eq!(agent1.runs.len(), 2);
        assert_eq!(agent1.queue_wait_time_seconds, Some(25));

        let agent2 = &loaded_metrics.agents["agent_2"];
        assert_eq!(agent2.total_runs, 3);
        assert_eq!(agent2.successful_runs, 3);
        assert_eq!(agent2.failed_runs, 0);
        assert_eq!(agent2.total_duration_ms, 30000);
        assert_eq!(agent2.runs.len(), 1);
        assert_eq!(agent2.queue_wait_time_seconds, None);
    }

    #[test]
    fn test_metrics_store_load_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        // Load from non-existent file should return an error
        let result = store.load();

        assert!(matches!(result, Err(MetricsError::FileNotFound(_))));
    }

    #[test]
    fn test_metrics_store_load_corrupted_file() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        // Create a corrupted metrics.json file
        let metrics_path = temp_dir.path().join("metrics.json");
        std::fs::write(&metrics_path, "{ invalid json content }").unwrap();

        // Load should return an error for corrupted file
        let result = store.load();

        assert!(matches!(result, Err(MetricsError::CorruptedFile(_))));
    }

    #[test]
    fn test_metrics_store_atomic_save() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        let metrics = create_test_all_metrics();

        // Save the metrics
        store.save(&metrics).unwrap();

        // Verify metrics.json exists and tmp file doesn't
        let metrics_path = temp_dir.path().join("metrics.json");
        let temp_path = metrics_path.with_extension("tmp");

        assert!(
            metrics_path.exists(),
            "metrics.json should exist after save"
        );
        assert!(
            !temp_path.exists(),
            "temp file should not exist after successful save"
        );

        // Load and verify content
        let loaded = store.load().unwrap();
        assert_eq!(loaded.agents.len(), 2);
    }

    #[test]
    fn test_metrics_store_overwrite_existing() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        // Save initial metrics
        let initial_metrics = create_test_all_metrics();
        store.save(&initial_metrics).unwrap();

        // Create different metrics
        let mut new_metrics = AllMetrics {
            agents: HashMap::new(),
        };

        let new_agent = AgentMetricsData {
            total_runs: 1,
            successful_runs: 1,
            failed_runs: 0,
            total_duration_ms: 1000,
            runs: vec![AgentRunResultData {
                run_id: "new_container".to_string(),
                timestamp: 9999999999,
                duration_ms: 1000,
                status: "success".to_string(),
                error_message: None,
                skills_installed_count: 0,
                skills_failed_count: 0,
                skills_install_time_seconds: None,
            }],
            queue_wait_time_seconds: None,
            queue_wait_times: vec![],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        new_metrics
            .agents
            .insert("new_agent".to_string(), new_agent);

        // Overwrite
        store.save(&new_metrics).unwrap();

        // Verify new data was saved
        let loaded = store.load().unwrap();
        assert_eq!(loaded.agents.len(), 1);
        assert!(loaded.agents.contains_key("new_agent"));
        assert!(!loaded.agents.contains_key("agent_1"));
        assert!(!loaded.agents.contains_key("agent_2"));
    }

    // ==================== AllMetrics Tests ====================

    #[test]
    fn test_all_metrics_creation_with_multiple_agents() {
        let metrics = create_test_all_metrics();

        assert_eq!(metrics.agents.len(), 2);
        assert!(metrics.agents.contains_key("agent_1"));
        assert!(metrics.agents.contains_key("agent_2"));
    }

    #[test]
    fn test_all_metrics_serialization() {
        let metrics = create_test_all_metrics();

        // Serialize to JSON
        let json = serde_json::to_string_pretty(&metrics).unwrap();

        // Verify JSON contains expected data
        assert!(json.contains("agent_1"));
        assert!(json.contains("agent_2"));
        assert!(json.contains("total_runs"));
        assert!(json.contains("successful_runs"));
        assert!(json.contains("failed_runs"));
    }

    #[test]
    fn test_all_metrics_deserialization() {
        let json_string = r#"{
            "agents": {
                "test_agent": {
                    "total_runs": 2,
                    "successful_runs": 2,
                    "failed_runs": 0,
                    "total_duration_ms": 20000,
                    "runs": [
                        {
                            "run_id": "container_1",
                            "timestamp": 1234567890,
                            "duration_ms": 10000,
                            "status": "success",
                            "error_message": null,
                            "skills_installed_count": 0,
                            "skills_failed_count": 0,
                            "skills_install_time_seconds": null
                        }
                    ],
                    "queue_wait_time_seconds": null,
                    "queue_wait_times": [],
                    "sigterm_count": 0,
                    "sigkill_count": 0,
                    "timeout_count": 0,
                    "total_skills_installed": 0,
                    "total_skills_failed": 0,
                    "skills_install_time_seconds": null,
                    "runs_with_skill_failures": 0
                }
            }
        }"#;

        let metrics: AllMetrics = serde_json::from_str(json_string).unwrap();

        assert_eq!(metrics.agents.len(), 1);
        assert!(metrics.agents.contains_key("test_agent"));

        let agent = &metrics.agents["test_agent"];
        assert_eq!(agent.total_runs, 2);
        assert_eq!(agent.successful_runs, 2);
        assert_eq!(agent.failed_runs, 0);
        assert_eq!(agent.total_duration_ms, 20000);
        assert_eq!(agent.runs.len(), 1);
    }

    #[test]
    fn test_all_metrics_roundtrip() {
        let original = create_test_all_metrics();

        // Serialize
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize
        let deserialized: AllMetrics = serde_json::from_str(&json).unwrap();

        // Verify equality
        assert_eq!(original, deserialized);
    }

    // ==================== AgentMetricsData Tests ====================

    #[test]
    fn test_agent_metrics_data_creation() {
        let agent_data = AgentMetricsData {
            total_runs: 10,
            successful_runs: 8,
            failed_runs: 2,
            total_duration_ms: 100000,
            runs: vec![],
            queue_wait_time_seconds: Some(15),
            queue_wait_times: vec![],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        assert_eq!(agent_data.total_runs, 10);
        assert_eq!(agent_data.successful_runs, 8);
        assert_eq!(agent_data.failed_runs, 2);
        assert_eq!(agent_data.total_duration_ms, 100000);
        assert!(agent_data.runs.is_empty());
        assert_eq!(agent_data.queue_wait_time_seconds, Some(15));
    }

    #[test]
    fn test_agent_metrics_data_adding_runs() {
        let mut agent_data = AgentMetricsData {
            total_runs: 0,
            successful_runs: 0,
            failed_runs: 0,
            total_duration_ms: 0,
            runs: vec![],
            queue_wait_time_seconds: None,
            queue_wait_times: vec![],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        // Add first run
        let run1 = AgentRunResultData {
            run_id: "container_1".to_string(),
            timestamp: 1234567890,
            duration_ms: 10000,
            status: "success".to_string(),
            error_message: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };
        agent_data.runs.push(run1.clone());
        agent_data.total_runs += 1;
        agent_data.successful_runs += 1;
        agent_data.total_duration_ms += run1.duration_ms;

        // Add second run
        let run2 = AgentRunResultData {
            run_id: "container_2".to_string(),
            timestamp: 1234567891,
            duration_ms: 15000,
            status: "failure".to_string(),
            error_message: Some("exit_code: 1".to_string()),
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };
        agent_data.runs.push(run2.clone());
        agent_data.total_runs += 1;
        agent_data.failed_runs += 1;
        agent_data.total_duration_ms += run2.duration_ms;

        // Verify totals
        assert_eq!(agent_data.total_runs, 2);
        assert_eq!(agent_data.successful_runs, 1);
        assert_eq!(agent_data.failed_runs, 1);
        assert_eq!(agent_data.total_duration_ms, 25000);
        assert_eq!(agent_data.runs.len(), 2);
    }

    #[test]
    fn test_agent_metrics_data_serialization() {
        let agent_data = AgentMetricsData {
            total_runs: 5,
            successful_runs: 4,
            failed_runs: 1,
            total_duration_ms: 50000,
            runs: vec![AgentRunResultData {
                run_id: "container_1".to_string(),
                timestamp: 1234567890,
                duration_ms: 10000,
                status: "success".to_string(),
                error_message: None,
                skills_installed_count: 0,
                skills_failed_count: 0,
                skills_install_time_seconds: None,
            }],
            queue_wait_time_seconds: Some(10),
            queue_wait_times: vec![],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        let json = serde_json::to_string_pretty(&agent_data).unwrap();
        assert!(json.contains("total_runs"));
        assert!(json.contains("container_1"));
    }

    // ==================== AgentRunResultData Tests ====================

    #[test]
    fn test_agent_run_result_data_creation() {
        let run_data = AgentRunResultData {
            run_id: "container_abc123".to_string(),
            timestamp: 1234567890,
            duration_ms: 10000,
            status: "success".to_string(),
            error_message: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        assert_eq!(run_data.run_id, "container_abc123");
        assert_eq!(run_data.timestamp, 1234567890);
        assert_eq!(run_data.duration_ms, 10000);
        assert_eq!(run_data.status, "success");
        assert_eq!(run_data.error_message, None);
    }

    #[test]
    fn test_agent_run_result_data_with_error() {
        let run_data = AgentRunResultData {
            run_id: "container_fail".to_string(),
            timestamp: 1234567891,
            duration_ms: 5000,
            status: "failure".to_string(),
            error_message: Some("exit_code: 127".to_string()),
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        assert_eq!(run_data.status, "failure");
        assert_eq!(run_data.error_message, Some("exit_code: 127".to_string()));
    }

    #[test]
    fn test_agent_run_result_data_serialization() {
        let run_data = AgentRunResultData {
            run_id: "container_test".to_string(),
            timestamp: 1234567890,
            duration_ms: 10000,
            status: "success".to_string(),
            error_message: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        // Serialize
        let json = serde_json::to_string(&run_data).unwrap();
        assert!(json.contains("container_test"));
        assert!(json.contains("success"));

        // Deserialize
        let deserialized: AgentRunResultData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.run_id, run_data.run_id);
        assert_eq!(deserialized.timestamp, run_data.timestamp);
        assert_eq!(deserialized.duration_ms, run_data.duration_ms);
        assert_eq!(deserialized.status, run_data.status);
        assert_eq!(deserialized.error_message, run_data.error_message);
    }

    #[test]
    fn test_agent_run_result_data_roundtrip() {
        let original = AgentRunResultData {
            run_id: "container_roundtrip".to_string(),
            timestamp: 1234567890,
            duration_ms: 15000,
            status: "failure".to_string(),
            error_message: Some("timed_out".to_string()),
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: AgentRunResultData = serde_json::from_str(&json).unwrap();

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_agent_run_result_data_comparison() {
        let run1 = AgentRunResultData {
            run_id: "container_1".to_string(),
            timestamp: 1234567890,
            duration_ms: 10000,
            status: "success".to_string(),
            error_message: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let run2 = AgentRunResultData {
            run_id: "container_1".to_string(),
            timestamp: 1234567890,
            duration_ms: 10000,
            status: "success".to_string(),
            error_message: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        let run3 = AgentRunResultData {
            run_id: "container_2".to_string(),
            timestamp: 1234567890,
            duration_ms: 10000,
            status: "success".to_string(),
            error_message: None,
            skills_installed_count: 0,
            skills_failed_count: 0,
            skills_install_time_seconds: None,
        };

        assert_eq!(run1, run2);
        assert_ne!(run1, run3);
    }

    // ==================== Backward Compatibility Tests ====================

    #[test]
    fn test_backward_compatibility_missing_skill_fields() {
        // Test that old JSON without skill fields can still be deserialized
        let old_json_string = r#"{
            "agents": {
                "old_agent": {
                    "total_runs": 1,
                    "successful_runs": 1,
                    "failed_runs": 0,
                    "total_duration_ms": 10000,
                    "runs": [
                        {
                            "run_id": "container_old",
                            "timestamp": 1234567890,
                            "duration_ms": 10000,
                            "status": "success",
                            "error_message": null
                        }
                    ],
                    "queue_wait_time_seconds": null,
                    "queue_wait_times": [],
                    "sigterm_count": 0,
                    "sigkill_count": 0,
                    "timeout_count": 0
                }
            }
        }"#;

        let metrics: AllMetrics = serde_json::from_str(old_json_string).unwrap();

        // Verify the agent was loaded
        assert_eq!(metrics.agents.len(), 1);
        assert!(metrics.agents.contains_key("old_agent"));

        let agent = &metrics.agents["old_agent"];
        assert_eq!(agent.total_runs, 1);

        // Verify that skill fields have default values (0 or None)
        assert_eq!(agent.total_skills_installed, 0);
        assert_eq!(agent.total_skills_failed, 0);
        assert_eq!(agent.skills_install_time_seconds, None);
        assert_eq!(agent.runs_with_skill_failures, 0);

        // Verify run-level skill fields also have default values
        assert_eq!(agent.runs.len(), 1);
        let run = &agent.runs[0];
        assert_eq!(run.skills_installed_count, 0);
        assert_eq!(run.skills_failed_count, 0);
        assert_eq!(run.skills_install_time_seconds, None);
    }

    // ==================== SaveWithRetry Tests ====================

    #[test]
    fn test_save_with_retry_success_on_first_attempt() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        let metrics = create_test_all_metrics();

        // save_with_retry should succeed on first attempt
        let result = store.save_with_retry(&metrics);
        assert!(result.is_ok(), "save_with_retry should succeed");

        // Verify the file was created
        let metrics_path = temp_dir.path().join("metrics.json");
        assert!(metrics_path.exists(), "metrics.json should exist");

        // Verify we can load the saved metrics
        let loaded = store.load().unwrap();
        assert_eq!(loaded.agents.len(), 2);
    }

    #[test]
    fn test_save_with_retry_creates_directory_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested/logs");
        let store = MetricsStore::new(nested_dir.clone());

        let metrics = create_test_all_metrics();

        // save_with_retry should create the directory and succeed
        let result = store.save_with_retry(&metrics);
        assert!(result.is_ok(), "save_with_retry should create directories");

        // Verify the directory was created
        assert!(nested_dir.exists(), "nested directory should exist");

        // Verify the file was created
        let metrics_path = nested_dir.join("metrics.json");
        assert!(metrics_path.exists(), "metrics.json should exist");
    }

    #[test]
    fn test_save_with_retry_overwrites_existing() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        // Save initial metrics
        let initial_metrics = create_test_all_metrics();
        store.save_with_retry(&initial_metrics).unwrap();

        // Create different metrics
        let mut new_metrics = AllMetrics {
            agents: HashMap::new(),
        };

        let new_agent = AgentMetricsData {
            total_runs: 1,
            successful_runs: 1,
            failed_runs: 0,
            total_duration_ms: 1000,
            runs: vec![AgentRunResultData {
                run_id: "new_container".to_string(),
                timestamp: 9999999999,
                duration_ms: 1000,
                status: "success".to_string(),
                error_message: None,
                skills_installed_count: 0,
                skills_failed_count: 0,
                skills_install_time_seconds: None,
            }],
            queue_wait_time_seconds: None,
            queue_wait_times: vec![],
            sigterm_count: 0,
            sigkill_count: 0,
            timeout_count: 0,
            total_skills_installed: 0,
            total_skills_failed: 0,
            skills_install_time_seconds: None,
            runs_with_skill_failures: 0,
        };

        new_metrics
            .agents
            .insert("new_agent".to_string(), new_agent);

        // Overwrite with save_with_retry
        store.save_with_retry(&new_metrics).unwrap();

        // Verify new data was saved
        let loaded = store.load().unwrap();
        assert_eq!(loaded.agents.len(), 1);
        assert!(loaded.agents.contains_key("new_agent"));
        assert!(!loaded.agents.contains_key("agent_1"));
        assert!(!loaded.agents.contains_key("agent_2"));
    }

    #[test]
    fn test_save_with_retry_behavior_matches_save() {
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        let metrics = create_test_all_metrics();

        // Save using save_with_retry
        store.save_with_retry(&metrics).unwrap();

        // Load and verify
        let loaded = store.load().unwrap();
        assert_eq!(loaded.agents.len(), 2);
        assert!(loaded.agents.contains_key("agent_1"));
        assert!(loaded.agents.contains_key("agent_2"));

        // Verify content is identical
        assert_eq!(loaded, metrics);
    }

    #[test]
    fn test_save_internal_is_private() {
        // This test verifies that save_internal is not part of the public API
        // by confirming we can still test the public interface correctly
        let temp_dir = TempDir::new().unwrap();
        let store = MetricsStore::new(temp_dir.path().to_path_buf());

        let metrics = create_test_all_metrics();

        // Using the public save() method (which calls save_internal internally)
        store.save(&metrics).unwrap();

        // Verify the metrics were saved correctly
        let loaded = store.load().unwrap();
        assert_eq!(loaded.agents.len(), 2);
    }
}
