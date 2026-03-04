//! Connection management for tracking active project connections
//!
//! This module provides thread-safe tracking of project connections including
//! heartbeat monitoring and stale connection detection.

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::instrument;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::gateway::reconnection::{
    ReconnectionConfig, ReconnectionManager,
};

/// Type alias for project ID (consistent with registry)
pub type ProjectId = String;

/// Errors that can occur during connection operations
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection not found for project: {0}")]
    ConnectionNotFound(ProjectId),

    #[error("Connection already exists for project: {0}")]
    ConnectionAlreadyExists(ProjectId),

    #[error("Invalid connection state: {0}")]
    InvalidState(String),

    #[error("Connection manager error: {0}")]
    ManagerError(String),

    #[error("Reconnection failed for project {project_id}: {message}")]
    ReconnectionFailed {
        project_id: ProjectId,
        message: String,
    },
}

/// Result type for connection operations
pub type ConnectionResult<T> = Result<T, ConnectionError>;

/// State of a connection
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    /// Connection has been disconnected
    Disconnected,
    /// Connection is active and healthy
    Connected,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Disconnected => write!(f, "Disconnected"),
        }
    }
}

/// Represents an active connection from a project
///
/// Tracks all necessary information for managing the connection lifecycle
/// including subscriptions and heartbeat for stale detection.
#[derive(Debug, Clone)]
pub struct Connection {
    /// Unique identifier for the project
    pub project_id: ProjectId,
    /// Unique session ID for this connection
    pub session_id: Uuid,
    /// List of channel IDs this project is subscribed to
    pub subscriptions: Vec<String>,
    /// Last heartbeat received from this connection
    pub last_heartbeat: DateTime<Utc>,
    /// Current state of the connection
    pub state: ConnectionState,
}

impl Connection {
    /// Create a new connection in the Connected state
    ///
    /// # Arguments
    /// * `project_id` - Unique identifier for the project
    /// * `session_id` - Unique session identifier
    /// * `subscriptions` - List of channel IDs to subscribe to
    ///
    /// # Example
    /// ```
    /// use uuid::Uuid;
    /// use gateway_connections::Connection;
    ///
    /// let conn = Connection::new(
    ///     "project-1".to_string(),
    ///     Uuid::new_v4(),
    ///     vec!["channel-1".to_string()],
    /// );
    /// ```
    pub fn new(project_id: ProjectId, session_id: Uuid, subscriptions: Vec<String>) -> Self {
        Self {
            project_id,
            session_id,
            subscriptions,
            last_heartbeat: Utc::now(),
            state: ConnectionState::Connected,
        }
    }

    /// Create a new connection with a fresh session ID
    pub fn new_with_fresh_session(project_id: ProjectId, subscriptions: Vec<String>) -> Self {
        Self::new(project_id, Uuid::new_v4(), subscriptions)
    }

    /// timestamp to Update the heartbeat current time
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
    }

    /// Check if the connection is considered stale based on timeout duration
    pub fn is_stale(&self, timeout: Duration) -> bool {
        let now = Utc::now();
        now - self.last_heartbeat > timeout
    }

    /// Mark the connection as disconnected
    pub fn disconnect(&mut self) {
        self.state = ConnectionState::Disconnected;
    }

    /// Mark the connection as connected
    pub fn connect(&mut self) {
        self.state = ConnectionState::Connected;
        self.update_heartbeat();
    }

    /// Add a subscription to this connection
    pub fn add_subscription(&mut self, channel_id: String) {
        if !self.subscriptions.contains(&channel_id) {
            self.subscriptions.push(channel_id);
        }
    }

    /// Remove a subscription from this connection
    pub fn remove_subscription(&mut self, channel_id: &str) {
        self.subscriptions.retain(|s| s != channel_id);
    }
}

/// Internal storage for connection manager
#[derive(Debug, Default)]
struct ConnectionManagerInner {
    /// Map from project_id to Connection
    connections: HashMap<ProjectId, Connection>,
}

/// Thread-safe manager for active connections
///
/// Uses `Arc<RwLock<ConnectionManagerInner>>` to allow concurrent
/// access from multiple async tasks.
#[derive(Debug, Clone, Default)]
pub struct ConnectionManager {
    inner: Arc<RwLock<ConnectionManagerInner>>,
}

impl ConnectionManager {
    /// Create a new empty ConnectionManager
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(ConnectionManagerInner::default())),
        }
    }

    /// Add a new connection to the manager
    ///
    /// Returns an error if a connection already exists for this project.
    #[instrument(name = "connection_add", skip(self, connection), fields(project_id))]
    pub async fn add_connection(&self, connection: Connection) -> ConnectionResult<()> {
        let project_id = connection.project_id.clone();

        let mut inner = self.inner.write().await;

        if inner.connections.contains_key(&project_id) {
            warn!(
                target: "gateway::connections",
                project_id = %project_id,
                "Attempted to add connection that already exists"
            );
            return Err(ConnectionError::ConnectionAlreadyExists(project_id));
        }

        inner.connections.insert(project_id.clone(), connection);

        // Record the project_id in the tracing span
        tracing::Span::current().record("project_id", &project_id);

        info!(
            target: "gateway::connections",
            project_id = %project_id,
            "Connection added successfully"
        );

        Ok(())
    }

    /// Remove a connection from the manager
    ///
    /// Returns the removed connection if it existed.
    #[instrument(name = "connection_remove", skip(self), fields(project_id))]
    pub async fn remove_connection(&self, project_id: &ProjectId) -> ConnectionResult<Connection> {
        let mut inner = self.inner.write().await;

        let removed = inner
            .connections
            .remove(project_id)
            .ok_or_else(|| ConnectionError::ConnectionNotFound(project_id.clone()))?;

        info!(
            target: "gateway::connections",
            project_id = %project_id,
            session_id = %removed.session_id,
            "Connection removed successfully"
        );

        Ok(removed)
    }

    /// Get a connection by project ID
    ///
    /// Returns None if no connection exists for this project.
    pub async fn get_connection(&self, project_id: &ProjectId) -> Option<Connection> {
        let inner = self.inner.read().await;
        inner.connections.get(project_id).cloned()
    }

    /// Get all active connections
    ///
    /// Returns a vector of all connections (both connected and disconnected).
    pub async fn all_connections(&self) -> Vec<Connection> {
        let inner = self.inner.read().await;
        inner.connections.values().cloned().collect()
    }

    /// Get all active (connected) connections
    ///
    /// Returns only connections in the Connected state.
    pub async fn active_connections(&self) -> Vec<Connection> {
        let inner = self.inner.read().await;
        inner
            .connections
            .values()
            .filter(|c| c.state == ConnectionState::Connected)
            .cloned()
            .collect()
    }

    /// Update the heartbeat for a connection
    ///
    /// Returns an error if the connection doesn't exist.
    #[instrument(name = "heartbeat_update", skip(self), fields(project_id))]
    pub async fn update_heartbeat(&self, project_id: &ProjectId) -> ConnectionResult<()> {
        let mut inner = self.inner.write().await;

        let Some(connection) = inner.connections.get_mut(project_id) else {
            return Err(ConnectionError::ConnectionNotFound(project_id.clone()));
        };

        connection.update_heartbeat();

        debug!(
            target: "gateway::connections",
            project_id = %project_id,
            "Heartbeat updated"
        );

        Ok(())
    }

    /// Disconnect a project connection
    ///
    /// Changes the connection state to Disconnected but keeps it in the manager.
    #[instrument(name = "connection_disconnect", skip(self), fields(project_id))]
    pub async fn disconnect(&self, project_id: &ProjectId) -> ConnectionResult<()> {
        let mut inner = self.inner.write().await;

        let Some(connection) = inner.connections.get_mut(project_id) else {
            return Err(ConnectionError::ConnectionNotFound(project_id.clone()));
        };

        connection.disconnect();

        info!(
            target: "gateway::connections",
            project_id = %project_id,
            "Connection disconnected"
        );

        Ok(())
    }

    /// Reconnect a project
    ///
    /// Changes the connection state back to Connected and updates heartbeat.
    #[instrument(name = "connection_reconnect", skip(self), fields(project_id))]
    pub async fn reconnect(&self, project_id: &ProjectId) -> ConnectionResult<()> {
        let mut inner = self.inner.write().await;

        let Some(connection) = inner.connections.get_mut(project_id) else {
            return Err(ConnectionError::ConnectionNotFound(project_id.clone()));
        };

        connection.connect();

        info!(
            target: "gateway::connections",
            project_id = %project_id,
            "Connection reconnected"
        );

        Ok(())
    }

    /// Attempt to reconnect with exponential backoff
    ///
    /// This method uses the ReconnectionManager to handle retry logic with
    /// exponential backoff (1s, 2s, 4s... max 60s). It preserves the existing
    /// session_id and subscriptions across reconnection attempts.
    ///
    /// # Arguments
    /// * `project_id` - The project ID to reconnect
    /// * `callback` - Async callback that attempts the actual reconnection.
    ///   Takes the current attempt number and returns true if successful.
    /// * `config` - Optional reconnection configuration. Uses default if None.
    ///
    /// # Returns
    /// * `Ok(true)` - Reconnection successful
    /// * `Ok(false)` - Reconnection failed but within retry limit (callback returned false)
    /// * `Err(ConnectionError::ConnectionNotFound)` - No connection exists for project
    /// * `Err(ConnectionError::ReconnectionFailed)` - All retries exhausted
    #[instrument(
        name = "connection_reconnect_with_backoff",
        skip(self, callback),
        fields(project_id)
    )]
    pub async fn reconnect_with_backoff<F, Fut>(&self, project_id: &ProjectId, callback: F, config: Option<ReconnectionConfig>) -> ConnectionResult<bool>
    where
        F: Fn(u32) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = bool> + Send,
    {
        let project_id = project_id.clone();

        // First, check if connection exists and get current session info
        let (_session_id, _subscriptions) = {
            let inner = self.inner.read().await;
            let Some(connection) = inner.connections.get(&project_id) else {
                return Err(ConnectionError::ConnectionNotFound(project_id.clone()));
            };
            // Preserve session_id and subscriptions across reconnects
            // (these values are preserved in the existing connection, not modified)
            (connection.session_id.clone(), connection.subscriptions.clone())
        };

        // Record project_id in tracing span
        tracing::Span::current().record("project_id", &project_id);

        // Create reconnection manager
        let reconnection_config = config.unwrap_or_default();
        let max_retries = reconnection_config.max_retries;
        let initial_delay = reconnection_config.initial_delay;
        let max_delay = reconnection_config.max_delay;
        let mut reconnection_manager = ReconnectionManager::new(project_id.clone(), reconnection_config);

        info!(
            target: "gateway::connections",
            project_id = %project_id,
            max_retries = max_retries,
            initial_delay_secs = initial_delay.as_secs(),
            max_delay_secs = max_delay.as_secs(),
            "Starting reconnection with exponential backoff"
        );

        // Attempt reconnection with the callback
        let result = reconnection_manager.attempt_reconnection(callback).await;

        match result {
            Ok(true) => {
                // Reconnection successful - update connection state
                let mut inner = self.inner.write().await;
                if let Some(connection) = inner.connections.get_mut(&project_id) {
                    connection.connect();
                    info!(
                        target: "gateway::connections",
                        project_id = %project_id,
                        session_id = %connection.session_id,
                        "Reconnection successful"
                    );
                }
                Ok(true)
            }
            Ok(false) => {
                // Callback returned false but within retry limit
                warn!(
                    target: "gateway::connections",
                    project_id = %project_id,
                    "Reconnection callback returned false"
                );
                Ok(false)
            }
            Err(reconnection_err) => {
                // All retries exhausted
                warn!(
                    target: "gateway::connections",
                    project_id = %project_id,
                    retries = max_retries,
                    error = %reconnection_err,
                    "Reconnection failed after max retries"
                );

                Err(ConnectionError::ReconnectionFailed {
                    project_id: project_id.clone(),
                    message: format!("Max retries ({}) exceeded: {}", max_retries, reconnection_err),
                })
            }
        }
    }

    /// Get the number of active connections
    pub async fn connection_count(&self) -> usize {
        let inner = self.inner.read().await;
        inner.connections.len()
    }

    /// Add a subscription to a connection
    pub async fn add_subscription(
        &self,
        project_id: &ProjectId,
        channel_id: String,
    ) -> ConnectionResult<()> {
        let mut inner = self.inner.write().await;

        let Some(connection) = inner.connections.get_mut(project_id) else {
            return Err(ConnectionError::ConnectionNotFound(project_id.clone()));
        };

        connection.add_subscription(channel_id);

        Ok(())
    }

    /// Remove a subscription from a connection
    pub async fn remove_subscription(
        &self,
        project_id: &ProjectId,
        channel_id: &str,
    ) -> ConnectionResult<()> {
        let mut inner = self.inner.write().await;

        let Some(connection) = inner.connections.get_mut(project_id) else {
            return Err(ConnectionError::ConnectionNotFound(project_id.clone()));
        };

        connection.remove_subscription(channel_id);

        Ok(())
    }
}

/// Callback type for stale connection cleanup
pub type CleanupCallback = Arc<dyn Fn(Vec<ProjectId>) + Send + Sync>;

/// Background task for detecting and cleaning up stale connections
///
/// Periodically checks all connections and calls the cleanup callback
/// for any connections that have exceeded the heartbeat timeout.
pub struct StaleConnectionDetector {
    /// The connection manager to check
    manager: ConnectionManager,
    /// Timeout duration after which a connection is considered stale
    timeout: Duration,
    /// Callback function to invoke with stale project IDs
    cleanup_callback: CleanupCallback,
    /// Handle for the spawned background task
    task_handle: Option<JoinHandle<()>>,
}

impl StaleConnectionDetector {
    /// Create a new StaleConnectionDetector
    ///
    /// # Arguments
    /// * `manager` - The ConnectionManager to monitor
    /// * `timeout` - Duration after which a connection is considered stale
    /// * `cleanup_callback` - Callback invoked with list of stale project IDs
    pub fn new(
        manager: ConnectionManager,
        timeout: Duration,
        cleanup_callback: CleanupCallback,
    ) -> Self {
        Self {
            manager,
            timeout,
            cleanup_callback,
            task_handle: None,
        }
    }

    /// Start the stale connection detection background task
    ///
    /// Spawns a tokio task that periodically checks for stale connections.
    /// The check interval is 1/4 of the timeout duration.
    pub fn start(&mut self, timeout: Duration) {
        // Don't start if already running
        if self.task_handle.is_some() {
            warn!(
                target: "gateway::connections",
                "StaleConnectionDetector already running"
            );
            return;
        }

        self.timeout = timeout;
        let manager = self.manager.clone();
        let callback = self.cleanup_callback.clone();
        let check_interval = timeout / 4;

        // Convert to milliseconds for more accurate timing
        let check_interval_ms = check_interval.num_milliseconds().max(10) as u64;

        let handle = tokio::spawn(async move {
            info!(
                target: "gateway::connections",
                check_interval_ms = check_interval_ms,
                timeout_secs = timeout.num_seconds(),
                "Stale connection detector started"
            );

            loop {
                // Wait for the check interval
                tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;

                // Find stale connections
                let stale_project_ids = manager
                    .all_connections()
                    .await
                    .into_iter()
                    .filter(|c| c.is_stale(timeout))
                    .map(|c| c.project_id)
                    .collect::<Vec<_>>();

                if !stale_project_ids.is_empty() {
                    info!(
                        target: "gateway::connections",
                        stale_count = stale_project_ids.len(),
                        "Detected stale connections"
                    );

                    // Call the cleanup callback
                    callback(stale_project_ids);
                }
            }
        });

        self.task_handle = Some(handle);
    }

    /// Stop the stale connection detection task
    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
            info!(
                target: "gateway::connections",
                "Stale connection detector stopped"
            );
        }
    }

    /// Check for stale connections synchronously (for testing)
    ///
    /// Returns a list of project IDs that have stale connections.
    pub async fn detect_stale(&self) -> Vec<ProjectId> {
        self.manager
            .all_connections()
            .await
            .into_iter()
            .filter(|c| c.is_stale(self.timeout))
            .map(|c| c.project_id)
            .collect()
    }
}

impl Drop for StaleConnectionDetector {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc as StdArc;
    use tokio::time::sleep;

    /// Helper to create a test connection
    fn create_test_connection(project_id: &str) -> Connection {
        Connection::new_with_fresh_session(
            project_id.to_string(),
            vec![format!("channel-{}", project_id)],
        )
    }

    #[tokio::test]
    async fn test_add_and_get_connection() {
        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");

        manager.add_connection(connection.clone()).await.unwrap();

        let retrieved = manager.get_connection(&"project-1".to_string()).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().project_id, "project-1");
    }

    #[tokio::test]
    async fn test_add_duplicate_connection_fails() {
        let manager = ConnectionManager::new();
        let connection1 = create_test_connection("project-1");
        let connection2 = create_test_connection("project-1");

        manager.add_connection(connection1).await.unwrap();

        let result = manager.add_connection(connection2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_connection() {
        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");

        manager.add_connection(connection).await.unwrap();

        let removed = manager
            .remove_connection(&"project-1".to_string())
            .await
            .unwrap();
        assert_eq!(removed.project_id, "project-1");

        let retrieved = manager.get_connection(&"project-1".to_string()).await;
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_connection_fails() {
        let manager = ConnectionManager::new();

        let result = manager.remove_connection(&"nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_all_connections() {
        let manager = ConnectionManager::new();

        manager
            .add_connection(create_test_connection("project-1"))
            .await
            .unwrap();
        manager
            .add_connection(create_test_connection("project-2"))
            .await
            .unwrap();
        manager
            .add_connection(create_test_connection("project-3"))
            .await
            .unwrap();

        let all = manager.all_connections().await;
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_connection_count() {
        let manager = ConnectionManager::new();

        assert_eq!(manager.connection_count().await, 0);

        manager
            .add_connection(create_test_connection("project-1"))
            .await
            .unwrap();
        assert_eq!(manager.connection_count().await, 1);

        manager
            .add_connection(create_test_connection("project-2"))
            .await
            .unwrap();
        assert_eq!(manager.connection_count().await, 2);
    }

    #[tokio::test]
    async fn test_update_heartbeat() {
        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");

        manager.add_connection(connection).await.unwrap();

        // Get original heartbeat
        let original = manager
            .get_connection(&"project-1".to_string())
            .await
            .unwrap();
        let original_heartbeat = original.last_heartbeat;

        // Wait a tiny bit and update heartbeat
        sleep(std::time::Duration::from_millis(10)).await;
        manager
            .update_heartbeat(&"project-1".to_string())
            .await
            .unwrap();

        let updated = manager
            .get_connection(&"project-1".to_string())
            .await
            .unwrap();
        assert!(updated.last_heartbeat >= original_heartbeat);
    }

    #[tokio::test]
    async fn test_update_heartbeat_nonexistent_fails() {
        let manager = ConnectionManager::new();

        let result = manager.update_heartbeat(&"nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_disconnect_and_reconnect() {
        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");

        manager.add_connection(connection).await.unwrap();

        // Disconnect
        manager.disconnect(&"project-1".to_string()).await.unwrap();

        let conn = manager
            .get_connection(&"project-1".to_string())
            .await
            .unwrap();
        assert_eq!(conn.state, ConnectionState::Disconnected);

        // Reconnect
        manager.reconnect(&"project-1".to_string()).await.unwrap();

        let conn = manager
            .get_connection(&"project-1".to_string())
            .await
            .unwrap();
        assert_eq!(conn.state, ConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_active_connections() {
        let manager = ConnectionManager::new();

        manager
            .add_connection(create_test_connection("project-1"))
            .await
            .unwrap();
        manager
            .add_connection(create_test_connection("project-2"))
            .await
            .unwrap();
        manager
            .add_connection(create_test_connection("project-3"))
            .await
            .unwrap();

        // Disconnect one
        manager.disconnect(&"project-2".to_string()).await.unwrap();

        let active = manager.active_connections().await;
        assert_eq!(active.len(), 2);
        assert!(active.iter().any(|c| c.project_id == "project-1"));
        assert!(active.iter().any(|c| c.project_id == "project-3"));
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");

        manager.add_connection(connection).await.unwrap();

        // Add subscription
        manager
            .add_subscription(&"project-1".to_string(), "channel-new".to_string())
            .await
            .unwrap();

        let conn = manager
            .get_connection(&"project-1".to_string())
            .await
            .unwrap();
        assert!(conn.subscriptions.contains(&"channel-new".to_string()));

        // Remove subscription
        manager
            .remove_subscription(&"project-1".to_string(), "channel-new")
            .await
            .unwrap();

        let conn = manager
            .get_connection(&"project-1".to_string())
            .await
            .unwrap();
        assert!(!conn.subscriptions.contains(&"channel-new".to_string()));
    }

    #[tokio::test]
    async fn test_stale_connection_detection() {
        let manager = ConnectionManager::new();

        // Add a connection
        manager
            .add_connection(create_test_connection("project-1"))
            .await
            .unwrap();

        // Create detector with very short timeout
        let callback = Arc::new(|_: Vec<ProjectId>| {});
        let detector =
            StaleConnectionDetector::new(manager.clone(), Duration::milliseconds(1), callback);

        // Wait longer than timeout
        sleep(std::time::Duration::from_millis(50)).await;

        // Should detect stale connection
        let stale = detector.detect_stale().await;
        assert_eq!(stale.len(), 1);
        assert_eq!(stale[0], "project-1");
    }

    #[tokio::test]
    async fn test_stale_connection_cleanup_callback() {
        let manager = ConnectionManager::new();

        // Add multiple connections
        manager
            .add_connection(create_test_connection("project-1"))
            .await
            .unwrap();
        manager
            .add_connection(create_test_connection("project-2"))
            .await
            .unwrap();

        // Track callback invocations
        let cleaned_count = StdArc::new(AtomicUsize::new(0));
        let cleaned_count_clone = cleaned_count.clone();

        let callback: CleanupCallback = StdArc::new(move |projects: Vec<ProjectId>| {
            cleaned_count_clone.fetch_add(projects.len(), Ordering::SeqCst);
        });

        // Create detector with short timeout and start it
        let mut detector =
            StaleConnectionDetector::new(manager, Duration::milliseconds(10), callback);
        detector.start(Duration::milliseconds(10));

        // Wait longer than timeout
        sleep(std::time::Duration::from_millis(100)).await;

        // Should have called callback
        assert!(cleaned_count.load(Ordering::SeqCst) >= 1);

        // Stop detector
        detector.stop().await;
    }

    #[tokio::test]
    async fn test_connection_is_stale() {
        let mut connection = create_test_connection("project-1");

        // Fresh connection should not be stale
        assert!(!connection.is_stale(Duration::seconds(60)));

        // Manually set heartbeat to the past
        connection.last_heartbeat = Utc::now() - Duration::seconds(120);

        // Should be stale with 60 second timeout
        assert!(connection.is_stale(Duration::seconds(60)));

        // Should not be stale with 180 second timeout
        assert!(!connection.is_stale(Duration::seconds(180)));
    }

    /// Test: Connection list accurate — verify all registered projects appear
    #[tokio::test]
    async fn test_connection_list_accurate() {
        let manager = ConnectionManager::new();

        // Register multiple projects
        let projects = vec!["project-a", "project-b", "project-c", "project-d"];
        for project_id in &projects {
            manager
                .add_connection(create_test_connection(project_id))
                .await
                .unwrap();
        }

        // Verify all appear in connection list
        let all = manager.all_connections().await;
        assert_eq!(all.len(), projects.len());

        for project_id in &projects {
            let conn = manager.get_connection(&project_id.to_string()).await;
            assert!(
                conn.is_some(),
                "Project {} should be in connection list",
                project_id
            );
        }
    }

    /// Test: Can connect 3+ projects — spawn concurrent connections
    #[tokio::test]
    async fn test_multiple_concurrent_connections() {
        use tokio::task;

        let manager = ConnectionManager::new();
        let manager_clone = manager.clone();

        // Spawn concurrent connection tasks
        let mut handles = vec![];
        for i in 0..5 {
            let mgr = manager_clone.clone();
            let handle = task::spawn(async move {
                mgr.add_connection(create_test_connection(&format!("project-{}", i)))
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }

        // Verify all connections exist
        assert_eq!(manager.connection_count().await, 5);

        let active = manager.active_connections().await;
        assert_eq!(active.len(), 5);
    }

    /// Test: Dead connections removed after timeout
    #[tokio::test]
    async fn test_dead_connections_removed_after_timeout() {
        let manager = ConnectionManager::new();

        // Add connections
        manager
            .add_connection(create_test_connection("project-alive"))
            .await
            .unwrap();
        manager
            .add_connection(create_test_connection("project-dead"))
            .await
            .unwrap();

        // Manually make one connection stale
        {
            let mut inner = manager.inner.write().await;
            if let Some(conn) = inner.connections.get_mut("project-dead") {
                conn.last_heartbeat = Utc::now() - Duration::seconds(300);
            }
        }

        // Create detector with 60 second timeout
        let cleanup_count = StdArc::new(AtomicUsize::new(0));
        let cleanup_count_clone = cleanup_count.clone();

        let callback: CleanupCallback = StdArc::new(move |projects: Vec<ProjectId>| {
            // Simulate cleanup by removing connections
            cleanup_count_clone.fetch_add(projects.len(), Ordering::SeqCst);
        });

        let mut detector =
            StaleConnectionDetector::new(manager.clone(), Duration::milliseconds(10), callback);

        // Start the background detector
        detector.start(Duration::milliseconds(10));

        // Wait for detection
        sleep(std::time::Duration::from_millis(200)).await;

        // Callback should have been invoked for the stale connection
        assert!(cleanup_count.load(Ordering::SeqCst) >= 1);

        detector.stop().await;
    }

    #[tokio::test]
    async fn test_connection_state_display() {
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(ConnectionState::Disconnected.to_string(), "Disconnected");
    }

    // Tests for reconnect_with_backoff method

    /// Test: Successful reconnection with exponential backoff
    #[tokio::test]
    async fn test_reconnect_with_backoff_success() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");
        let original_session_id = connection.session_id;
        
        manager.add_connection(connection).await.unwrap();

        // Track attempt count
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        // Callback that succeeds on first try
        let result = manager
            .reconnect_with_backoff(
                &"project-1".to_string(),
                move |_attempt| {
                    let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    async move { count == 0 } // Succeed on first attempt
                },
                None,
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Verify connection state is now Connected
        let conn = manager.get_connection(&"project-1".to_string()).await.unwrap();
        assert_eq!(conn.state, ConnectionState::Connected);
        
        // Verify session_id was preserved
        assert_eq!(conn.session_id, original_session_id);
    }

    /// Test: Failed reconnection after max retries exceeded
    #[tokio::test]
    async fn test_reconnect_with_backoff_max_retries_exceeded() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use crate::gateway::reconnection::ReconnectionConfig;
        use std::time::Duration;

        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");
        
        manager.add_connection(connection).await.unwrap();

        // Track attempt count
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        // Callback that always fails
        let result = manager
            .reconnect_with_backoff(
                &"project-1".to_string(),
                move |_attempt| {
                    attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    async move { false } // Always fail
                },
                Some(ReconnectionConfig {
                    initial_delay: Duration::from_millis(1),
                    max_delay: Duration::from_millis(10),
                    max_retries: 3,
                    multiplier: 2.0,
                }),
            )
            .await;

        // Should fail with ReconnectionFailed error
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            ConnectionError::ReconnectionFailed { project_id, message } => {
                assert_eq!(project_id, "project-1");
                assert!(message.contains("Max retries"));
            }
            _ => panic!("Expected ReconnectionFailed error"),
        }
    }

    /// Test: Preserves session_id and subscriptions across reconnects
    #[tokio::test]
    async fn test_reconnect_preserves_session_and_subscriptions() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let manager = ConnectionManager::new();
        
        // Create connection with specific session and subscriptions
        let connection = Connection::new(
            "project-1".to_string(),
            Uuid::new_v4(),
            vec!["channel-1".to_string(), "channel-2".to_string()],
        );
        let original_session_id = connection.session_id;
        let original_subscriptions = connection.subscriptions.clone();
        
        manager.add_connection(connection).await.unwrap();

        // Track attempts
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        // Callback that succeeds on second attempt
        let result = manager
            .reconnect_with_backoff(
                &"project-1".to_string(),
                move |_attempt| {
                    let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    async move { count >= 1 } // Succeed on second attempt
                },
                None,
            )
            .await;

        assert!(result.is_ok());
        
        // Verify session_id was preserved
        let conn = manager.get_connection(&"project-1".to_string()).await.unwrap();
        assert_eq!(conn.session_id, original_session_id);
        assert_eq!(conn.subscriptions, original_subscriptions);
    }

    /// Test: Connection not found returns error
    #[tokio::test]
    async fn test_reconnect_with_backoff_connection_not_found() {
        let manager = ConnectionManager::new();

        let result = manager
            .reconnect_with_backoff(
                &"nonexistent".to_string(),
                |_attempt| async move { true },
                None,
            )
            .await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ConnectionError::ConnectionNotFound(_) => {}
            _ => panic!("Expected ConnectionNotFound error"),
        }
    }

    /// Test: Callback returns false but within retry limit
    #[tokio::test]
    async fn test_reconnect_with_backoff_callback_returns_false() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use crate::gateway::reconnection::ReconnectionConfig;
        use std::time::Duration;

        let manager = ConnectionManager::new();
        let connection = create_test_connection("project-1");
        
        manager.add_connection(connection).await.unwrap();

        // Track attempts
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = Arc::clone(&attempt_count);

        // Custom config with 5 retries
        let config = ReconnectionConfig {
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            max_retries: 5,
            multiplier: 2.0,
        };

        // Callback that returns false (simulating connection failure but wants to continue)
        let result = manager
            .reconnect_with_backoff(
                &"project-1".to_string(),
                move |attempt| {
                    let count = attempt_count_clone.fetch_add(1, Ordering::SeqCst);
                    async move { 
                        // Return false for first 3 attempts, then true
                        count >= 3 
                    }
                },
                Some(config),
            )
            .await;

        // Should succeed after callback returns true
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // Verify connection state is Connected
        let conn = manager.get_connection(&"project-1".to_string()).await.unwrap();
        assert_eq!(conn.state, ConnectionState::Connected);
    }
}
