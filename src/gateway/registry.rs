//! Channel Registry for tracking project<->channel subscriptions
//!
//! This module provides thread-safe tracking of which projects are subscribed
//! to which channels, enabling correct message routing in the gateway.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};
use tracing::instrument;
use uuid::Uuid;

/// Type alias for project ID
pub type ProjectId = String;

/// Errors that can occur during registry operations
#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Project not found: {0}")]
    ProjectNotFound(ProjectId),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Project already registered: {0}")]
    ProjectAlreadyRegistered(ProjectId),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type for registry operations
pub type RegistryResult<T> = Result<T, RegistryError>;

/// Internal state of the registry
///
/// Maps channels to projects and projects to their connections
#[derive(Debug, Default)]
struct RegistryInner {
    /// Map from channel_id to list of project IDs subscribed to that channel
    channel_to_projects: HashMap<String, Vec<ProjectId>>,
    /// Map from project_id to ProjectConnection
    projects: HashMap<ProjectId, ProjectConnection>,
}

/// Represents a connected project in the registry
#[derive(Debug, Clone)]
pub struct ProjectConnection {
    /// Unique identifier for the project
    pub project_id: ProjectId,
    /// Human-readable name of the project
    pub project_name: String,
    /// Sender for WebSocket messages to the project
    pub ws_sender: mpsc::Sender<String>,
    /// Unique session ID for this connection
    pub session_id: Uuid,
    /// List of channel IDs this project is subscribed to
    pub subscribed_channels: Vec<String>,
    /// When this project was registered
    pub registered_at: DateTime<Utc>,
    /// Last heartbeat received from this project
    pub last_heartbeat: DateTime<Utc>,
}

impl ProjectConnection {
    /// Create a new ProjectConnection
    pub fn new(
        project_id: ProjectId,
        project_name: String,
        ws_sender: mpsc::Sender<String>,
    ) -> Self {
        Self {
            project_id: project_id.clone(),
            project_name,
            ws_sender,
            session_id: Uuid::new_v4(),
            subscribed_channels: Vec::new(),
            registered_at: Utc::now(),
            last_heartbeat: Utc::now(),
        }
    }
}

/// Thread-safe registry for tracking channel-to-project mappings
///
/// This struct uses `Arc<RwLock<RegistryInner>>` to allow concurrent
/// access from multiple async tasks.
#[derive(Debug, Clone)]
pub struct ChannelRegistry {
    inner: Arc<RwLock<RegistryInner>>,
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelRegistry {
    /// Create a new empty ChannelRegistry
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(RegistryInner::default())),
        }
    }

    /// Register a project with the gateway and subscribe to channels
    ///
    /// If the project is already registered, updates its channel subscriptions.
    #[instrument(name = "registry_register", skip(self, project, channels), fields(project_id, project_name))]
    pub async fn register(
        &self,
        project: ProjectConnection,
        channels: Vec<String>,
    ) -> RegistryResult<()> {
        let project_id = project.project_id.clone();

        {
            let mut inner = self.inner.write().await;

            // Check if project already exists
            let is_update = inner.projects.contains_key(&project_id);

            if is_update {
                warn!(
                    target: "gateway::registry",
                    project_id = %project_id,
                    "Updating existing project registration"
                );
                // Collect old channels first to avoid borrow checker issues
                let old_channels: Vec<String> = inner
                    .projects
                    .get(&project_id)
                    .map(|p| p.subscribed_channels.clone())
                    .unwrap_or_default();

                // Remove from old channel mappings
                for channel in &old_channels {
                    if let Some(projects) = inner.channel_to_projects.get_mut(channel) {
                        projects.retain(|p| p != &project_id);
                    }
                }
            } else {
                info!(
                    target: "gateway::registry",
                    project_id = %project_id,
                    project_name = %project.project_name,
                    session_id = %project.session_id,
                    "Registering new project"
                );
            }

            // Add to projects map
            let mut updated_project = project;
            updated_project.subscribed_channels = channels.clone();
            
            // Record the project_id and project_name in the tracing span
            tracing::Span::current()
                .record("project_id", &project_id)
                .record("project_name", &updated_project.project_name);

            inner.projects.insert(project_id.clone(), updated_project);

            // Add to channel mappings (fan-out: multiple projects per channel)
            for channel in &channels {
                inner
                    .channel_to_projects
                    .entry(channel.clone())
                    .or_insert_with(Vec::new)
                    .push(project_id.clone());
            }
        }

        debug!(
            target: "gateway::registry",
            project_id = %project_id,
            channel_count = channels.len(),
            channels = ?channels,
            "Project registered successfully"
        );

        Ok(())
    }

    /// Unregister a project from the gateway
    ///
    /// Removes the project from all channel subscriptions.
    pub async fn unregister(&self, project_id: &ProjectId) -> RegistryResult<()> {
        let project_id_str = project_id.clone();

        let mut inner = self.inner.write().await;

        // Check if project exists
        let Some(project) = inner.projects.remove(project_id) else {
            warn!(
                target: "gateway::registry",
                project_id = %project_id,
                "Attempted to unregister non-existent project"
            );
            return Err(RegistryError::ProjectNotFound(project_id.clone()));
        };

        // Remove from all channel mappings
        for channel in &project.subscribed_channels {
            if let Some(projects) = inner.channel_to_projects.get_mut(channel) {
                projects.retain(|p| p != project_id);
            }
            // Clean up empty channel entries
            if let Some(projects) = inner.channel_to_projects.get(channel) {
                if projects.is_empty() {
                    inner.channel_to_projects.remove(channel);
                }
            }
        }

        info!(
            target: "gateway::registry",
            project_id = %project_id_str,
            project_name = %project.project_name,
            session_id = %project.session_id,
            channels = ?project.subscribed_channels,
            "Project unregistered successfully"
        );

        Ok(())
    }

    /// Get all projects subscribed to a specific channel
    ///
    /// Returns a vector of project IDs that are subscribed to the given channel.
    pub async fn projects_for_channel(&self, channel_id: &str) -> Vec<ProjectId> {
        let inner = self.inner.read().await;

        inner
            .channel_to_projects
            .get(channel_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Add a channel subscription for a project
    pub async fn add_channel_subscription(
        &self,
        project_id: &ProjectId,
        channel_id: &str,
    ) -> RegistryResult<()> {
        let mut inner = self.inner.write().await;

        // Check if project exists
        let Some(project) = inner.projects.get_mut(project_id) else {
            return Err(RegistryError::ProjectNotFound(project_id.clone()));
        };

        // Add channel if not already subscribed
        if !project
            .subscribed_channels
            .contains(&channel_id.to_string())
        {
            project.subscribed_channels.push(channel_id.to_string());

            // Add to channel-to-projects mapping
            inner
                .channel_to_projects
                .entry(channel_id.to_string())
                .or_insert_with(Vec::new)
                .push(project_id.clone());

            debug!(
                project_id = %project_id,
                channel_id = %channel_id,
                "Added channel subscription"
            );
        }

        Ok(())
    }

    /// Remove a channel subscription for a project
    pub async fn remove_channel_subscription(
        &self,
        project_id: &ProjectId,
        channel_id: &str,
    ) -> RegistryResult<()> {
        let mut inner = self.inner.write().await;

        // Check if project exists
        let Some(project) = inner.projects.get_mut(project_id) else {
            return Err(RegistryError::ProjectNotFound(project_id.clone()));
        };

        // Remove channel from project's subscriptions
        project.subscribed_channels.retain(|c| c != channel_id);

        // Remove from channel-to-projects mapping
        if let Some(projects) = inner.channel_to_projects.get_mut(channel_id) {
            projects.retain(|p| p != project_id);
        }

        // Clean up empty channel entries
        if let Some(projects) = inner.channel_to_projects.get(channel_id) {
            if projects.is_empty() {
                inner.channel_to_projects.remove(channel_id);
            }
        }

        debug!(
            project_id = %project_id,
            channel_id = %channel_id,
            "Removed channel subscription"
        );

        Ok(())
    }

    /// Get a project's connection info
    pub async fn get_project(&self, project_id: &ProjectId) -> RegistryResult<ProjectConnection> {
        let inner = self.inner.read().await;

        inner
            .projects
            .get(project_id)
            .cloned()
            .ok_or_else(|| RegistryError::ProjectNotFound(project_id.clone()))
    }

    /// Check if a project is registered
    pub async fn is_registered(&self, project_id: &ProjectId) -> bool {
        let inner = self.inner.read().await;
        inner.projects.contains_key(project_id)
    }

    /// Get all registered projects
    pub async fn all_projects(&self) -> Vec<ProjectConnection> {
        let inner = self.inner.read().await;
        inner.projects.values().cloned().collect()
    }

    /// Get all channels that have subscribers
    pub async fn all_channels(&self) -> Vec<String> {
        let inner = self.inner.read().await;
        inner.channel_to_projects.keys().cloned().collect()
    }

    /// Update the heartbeat time for a project
    pub async fn update_heartbeat(&self, project_id: &ProjectId) -> RegistryResult<()> {
        let mut inner = self.inner.write().await;

        let Some(project) = inner.projects.get_mut(project_id) else {
            return Err(RegistryError::ProjectNotFound(project_id.clone()));
        };

        project.last_heartbeat = Utc::now();

        debug!(
            target: "gateway::registry",
            project_id = %project_id,
            "Heartbeat updated"
        );

        Ok(())
    }

    /// Check if a project's connection is stale based on timeout duration
    pub async fn is_connection_stale(&self, project_id: &ProjectId, timeout: chrono::Duration) -> bool {
        let inner = self.inner.read().await;

        if let Some(project) = inner.projects.get(project_id) {
            let now = Utc::now();
            return now - project.last_heartbeat > timeout;
        }

        // If project not found, consider it stale
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    /// Helper to create a test project connection
    fn create_test_project(project_id: &str) -> (ProjectConnection, mpsc::Sender<String>) {
        let (sender, _receiver) = mpsc::channel(10);
        let sender_clone = sender.clone();
        let project = ProjectConnection::new(
            project_id.to_string(),
            format!("Test Project {}", project_id),
            sender,
        );
        (project, sender_clone)
    }

    #[tokio::test]
    async fn test_register_new_project() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-1".to_string(), "channel-2".to_string()];

        registry.register(project, channels.clone()).await.unwrap();

        // Verify project is registered
        assert!(registry.is_registered(&"project-1".to_string()).await);

        // Verify channel subscriptions
        let projects = registry.projects_for_channel("channel-1").await;
        assert_eq!(projects, vec!["project-1".to_string()]);

        let projects = registry.projects_for_channel("channel-2").await;
        assert_eq!(projects, vec!["project-1".to_string()]);
    }

    #[tokio::test]
    async fn test_unregister_project() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-1".to_string()];

        registry.register(project, channels).await.unwrap();
        assert!(registry.is_registered(&"project-1".to_string()).await);

        registry.unregister(&"project-1".to_string()).await.unwrap();
        assert!(!registry.is_registered(&"project-1".to_string()).await);

        // Verify project is removed from channel
        let projects = registry.projects_for_channel("channel-1").await;
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn test_projects_for_channel() {
        let registry = ChannelRegistry::new();

        // Register multiple projects to same channel
        let (project1, _sender1) = create_test_project("project-1");
        let (project2, _sender2) = create_test_project("project-2");

        registry
            .register(project1, vec!["channel-1".to_string()])
            .await
            .unwrap();
        registry
            .register(project2, vec!["channel-1".to_string()])
            .await
            .unwrap();

        let projects = registry.projects_for_channel("channel-1").await;
        assert_eq!(projects.len(), 2);
        assert!(projects.contains(&"project-1".to_string()));
        assert!(projects.contains(&"project-2".to_string()));
    }

    #[tokio::test]
    async fn test_add_channel_subscription() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");

        registry
            .register(project, vec!["channel-1".to_string()])
            .await
            .unwrap();

        registry
            .add_channel_subscription(&"project-1".to_string(), "channel-2")
            .await
            .unwrap();

        let projects = registry.projects_for_channel("channel-2").await;
        assert!(projects.contains(&"project-1".to_string()));
    }

    #[tokio::test]
    async fn test_remove_channel_subscription() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");

        registry
            .register(
                project,
                vec!["channel-1".to_string(), "channel-2".to_string()],
            )
            .await
            .unwrap();

        registry
            .remove_channel_subscription(&"project-1".to_string(), "channel-1")
            .await
            .unwrap();

        let projects = registry.projects_for_channel("channel-1").await;
        assert!(projects.is_empty());

        let projects = registry.projects_for_channel("channel-2").await;
        assert!(projects.contains(&"project-1".to_string()));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;
        use tokio::task;

        let registry = Arc::new(ChannelRegistry::new());

        // Spawn multiple tasks registering projects concurrently
        let mut handles = vec![];
        for i in 0..10 {
            let reg = registry.clone();
            let handle = task::spawn(async move {
                let (project, _sender) = create_test_project(&format!("project-{}", i));
                reg.register(project, vec![format!("channel-{}", i % 3)])
                    .await
                    .unwrap();
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task failed");
        }

        // Verify all projects registered
        assert_eq!(registry.all_projects().await.len(), 10);
    }

    #[tokio::test]
    async fn test_update_existing_registration() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");

        // Initial registration
        registry
            .register(project, vec!["channel-1".to_string()])
            .await
            .unwrap();

        // Update with new channels
        let (project2, _sender2) = create_test_project("project-1");
        registry
            .register(project2, vec!["channel-2".to_string()])
            .await
            .unwrap();

        // Should only be in channel-2 now
        let projects = registry.projects_for_channel("channel-1").await;
        assert!(projects.is_empty());

        let projects = registry.projects_for_channel("channel-2").await;
        assert!(projects.contains(&"project-1".to_string()));
    }

    #[tokio::test]
    async fn test_get_project() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");

        registry
            .register(project, vec!["channel-1".to_string()])
            .await
            .unwrap();

        let retrieved = registry
            .get_project(&"project-1".to_string())
            .await
            .unwrap();
        assert_eq!(retrieved.project_id, "project-1");
    }

    #[tokio::test]
    async fn test_get_nonexistent_project() {
        let registry = ChannelRegistry::new();
        let result = registry.get_project(&"nonexistent".to_string()).await;
        assert!(result.is_err());
    }

    // === Required tests for ChannelRegistry routing ===

    #[tokio::test]
    async fn should_return_projects_for_subscribed_channel() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-x".to_string()];

        registry.register(project, channels).await.unwrap();

        let projects = registry.projects_for_channel("channel-x").await;
        assert_eq!(projects, vec!["project-1".to_string()]);
    }

    #[tokio::test]
    async fn should_return_empty_for_unsubscribed_channel() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-a".to_string()];

        registry.register(project, channels).await.unwrap();

        let projects = registry.projects_for_channel("channel-b").await;
        assert!(projects.is_empty());
    }

    #[tokio::test]
    async fn should_return_multiple_projects_for_same_channel() {
        let registry = ChannelRegistry::new();

        let (project1, _sender1) = create_test_project("project-1");
        let (project2, _sender2) = create_test_project("project-2");
        let (project3, _sender3) = create_test_project("project-3");

        registry
            .register(project1, vec!["shared-channel".to_string()])
            .await
            .unwrap();
        registry
            .register(project2, vec!["shared-channel".to_string()])
            .await
            .unwrap();
        registry
            .register(project3, vec!["shared-channel".to_string()])
            .await
            .unwrap();

        let projects = registry.projects_for_channel("shared-channel").await;
        assert_eq!(projects.len(), 3);
        assert!(projects.contains(&"project-1".to_string()));
        assert!(projects.contains(&"project-2".to_string()));
        assert!(projects.contains(&"project-3".to_string()));
    }

    #[tokio::test]
    async fn should_not_return_project_after_unsubscribing() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-1".to_string(), "channel-2".to_string()];

        registry.register(project, channels).await.unwrap();

        // Verify project is in both channels
        assert!(!registry.projects_for_channel("channel-1").await.is_empty());
        assert!(!registry.projects_for_channel("channel-2").await.is_empty());

        // Remove subscription to channel-1
        registry
            .remove_channel_subscription(&"project-1".to_string(), "channel-1")
            .await
            .unwrap();

        // channel-1 should no longer have the project
        let projects = registry.projects_for_channel("channel-1").await;
        assert!(projects.is_empty());

        // channel-2 should still have the project
        let projects = registry.projects_for_channel("channel-2").await;
        assert_eq!(projects, vec!["project-1".to_string()]);
    }

    // === Heartbeat Protocol Tests ===

    /// Test that update_heartbeat updates the timestamp
    #[tokio::test]
    async fn should_update_heartbeat_timestamp() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-1".to_string()];

        registry.register(project, channels).await.unwrap();

        // Get initial project state
        let project_before = registry.get_project(&"project-1".to_string()).await.unwrap();
        let initial_heartbeat = project_before.last_heartbeat;

        // Wait a small amount to ensure time difference
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Update heartbeat
        registry
            .update_heartbeat(&"project-1".to_string())
            .await
            .unwrap();

        // Verify heartbeat was updated
        let project_after = registry.get_project(&"project-1".to_string()).await.unwrap();
        assert!(
            project_after.last_heartbeat > initial_heartbeat,
            "Heartbeat should be updated to a newer timestamp"
        );
    }

    /// Test that is_connection_stale returns false for fresh connections
    #[tokio::test]
    async fn should_return_not_stale_for_fresh_connection() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-1".to_string()];

        registry.register(project, channels).await.unwrap();

        // Check immediately - should not be stale
        let is_stale = registry
            .is_connection_stale(&"project-1".to_string(), chrono::Duration::seconds(90))
            .await;
        assert!(!is_stale, "Fresh connection should not be stale");
    }

    /// Test that is_connection_stale returns true after timeout
    #[tokio::test]
    async fn should_return_stale_after_timeout() {
        // Create a registry with a manually manipulated timestamp
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("project-1");
        let channels = vec!["channel-1".to_string()];

        registry.register(project, channels).await.unwrap();

        // Manually set the last heartbeat to 100 seconds ago (beyond 90s timeout)
        {
            let mut inner = registry.inner.write().await;
            if let Some(p) = inner.projects.get_mut("project-1") {
                p.last_heartbeat = Utc::now() - chrono::Duration::seconds(100);
            }
        }

        // Check with 90 second timeout - should be stale
        let is_stale = registry
            .is_connection_stale(&"project-1".to_string(), chrono::Duration::seconds(90))
            .await;
        assert!(is_stale, "Connection should be stale after timeout");
    }

    /// Test that is_connection_stale returns true for non-existent project
    #[tokio::test]
    async fn should_return_stale_for_nonexistent_project() {
        let registry = ChannelRegistry::new();

        // Non-existent project should be considered stale
        let is_stale = registry
            .is_connection_stale(&"nonexistent".to_string(), chrono::Duration::seconds(90))
            .await;
        assert!(is_stale, "Non-existent project should be considered stale");
    }

    /// Test complete heartbeat flow: register -> heartbeat -> unregister
    #[tokio::test]
    async fn should_handle_complete_heartbeat_flow() {
        let registry = ChannelRegistry::new();
        let (project, _sender) = create_test_project("heartbeat-project");
        let channels = vec!["heartbeat-channel".to_string()];

        // 1. Register project
        registry.register(project, channels.clone()).await.unwrap();
        assert!(registry.is_registered(&"heartbeat-project".to_string()).await);

        // 2. Update heartbeat
        registry
            .update_heartbeat(&"heartbeat-project".to_string())
            .await
            .unwrap();

        // Verify not stale immediately after heartbeat
        let is_stale = registry
            .is_connection_stale(&"heartbeat-project".to_string(), chrono::Duration::seconds(90))
            .await;
        assert!(!is_stale, "Connection should not be stale after heartbeat");

        // 3. Unregister project
        registry
            .unregister(&"heartbeat-project".to_string())
            .await
            .unwrap();
        assert!(!registry.is_registered(&"heartbeat-project".to_string()).await);

        // 4. Verify removed from channel
        let projects = registry.projects_for_channel("heartbeat-channel").await;
        assert!(projects.is_empty(), "Project should be removed from channel");
    }
}
