//! Outbox Poller module - relays agent updates to Discord
//!
//! This module implements a background task that periodically scans the outbox
//! directory for markdown files and sends them to a Discord channel.

use crate::discord::api::DiscordApiClient;
use std::path::{Path, PathBuf};
use tokio::fs;

/// OutboxPoller configuration and state
pub struct OutboxPoller {
    /// Discord channel ID to send messages to
    channel_id: String,
    /// Discord API client for sending messages
    api_client: DiscordApiClient,
    /// Path to the outbox directory to scan
    outbox_path: PathBuf,
    /// Path to the archive directory for processed files
    archive_path: PathBuf,
}

impl OutboxPoller {
    /// Create a new OutboxPoller
    pub fn new(
        channel_id: String,
        api_client: DiscordApiClient,
        outbox_path: PathBuf,
        archive_path: PathBuf,
    ) -> Self {
        Self {
            channel_id,
            api_client,
            outbox_path,
            archive_path,
        }
    }

    /// Create a new OutboxPoller with default paths
    ///
    /// Uses `comms/outbox/` for scanning and `comms/archive/` for archiving.
    pub fn with_defaults(channel_id: String, api_client: DiscordApiClient) -> Self {
        Self::new(
            channel_id,
            api_client,
            PathBuf::from("comms/outbox"),
            PathBuf::from("comms/archive"),
        )
    }

    /// Start the outbox poller - runs an infinite loop with 60-second intervals
    ///
    /// On each interval:
    /// 1. Scan the outbox directory for `.md` files
    /// 2. For each file:
    ///    - Read the content
    ///    - Format as: "📬 **Agent Update** — `{filename}`\n\n{content}"
    ///    - Send to Discord channel
    ///    - Move the file to the archive directory
    /// 3. If no files, do nothing (no error)
    pub async fn start(&mut self) {
        use std::time::Duration;
        use tokio::time::interval;

        let mut ticker = interval(Duration::from_secs(60));

        tracing::info!(
            "Outbox poller started, scanning {} every 60 seconds",
            self.outbox_path.display()
        );

        loop {
            ticker.tick().await;

            if let Err(e) = self.poll().await {
                // Log errors but don't crash the poller
                tracing::error!("Error processing outbox: {}", e);
            }
        }
    }

    /// Poll the outbox directory for new messages and process them.
    ///
    /// This method:
    /// 1. Scans the outbox directory (`comms/outbox/`) for `.md` files
    /// 2. For each file:
    ///    - Reads the content
    ///    - Sends to Discord channel
    ///    - Moves the file to the archive directory
    /// 3. Returns Ok if successful, or an error if something fails
    ///
    /// # Returns
    ///
    /// * `Result<(), OutboxError>` - Ok on success, error on failure
    pub async fn poll(&mut self) -> Result<(), OutboxError> {
        self.process_outbox().await
    }

    /// Start the outbox poller with graceful shutdown support.
    ///
    /// This method runs an infinite loop with 60-second intervals until
    /// the shutdown signal is received.
    ///
    /// # Arguments
    ///
    /// * `shutdown_rx` - Broadcast receiver for shutdown signal
    pub async fn start_with_shutdown(
        &mut self,
        mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
    ) {
        use std::time::Duration;
        use tokio::time::interval;

        let mut ticker = interval(Duration::from_secs(60));

        tracing::info!(
            "Outbox poller started with shutdown support, scanning {} every 60 seconds",
            self.outbox_path.display()
        );

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = self.poll().await {
                        // Log errors but don't crash the poller
                        tracing::error!("Error processing outbox: {}", e);
                    }
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Outbox poller received shutdown signal");
                    break;
                }
            }
        }

        tracing::info!("Outbox poller stopped");
    }

    /// Process all pending files in the outbox
    async fn process_outbox(&mut self) -> Result<(), OutboxError> {
        // Ensure directories exist
        self.ensure_directories().await?;

        // Read all .md files from the outbox directory
        let mut entries = fs::read_dir(&self.outbox_path)
            .await
            .map_err(|e| OutboxError::IoError(format!("Failed to read outbox directory: {}", e)))?;

        let mut files_to_process: Vec<PathBuf> = Vec::new();

        // Collect all .md files
        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| OutboxError::IoError(format!("Failed to read directory entry: {}", e)))?
        {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md") {
                files_to_process.push(path);
            }
        }

        // Sort by filename for consistent ordering
        files_to_process.sort();

        // Process each file
        for file_path in files_to_process {
            match self.process_file(&file_path).await {
                Ok(_) => {
                    tracing::debug!(
                        "Successfully processed outbox file: {}",
                        file_path.display()
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to process file {}: {}", file_path.display(), e);
                    // Continue with other files even if one fails
                }
            }
        }

        Ok(())
    }

    /// Process a single outbox file
    async fn process_file(&mut self, file_path: &Path) -> Result<(), OutboxError> {
        // Read the file content
        let content = fs::read_to_string(file_path)
            .await
            .map_err(|e| OutboxError::IoError(format!("Failed to read file: {}", e)))?;

        // Get the filename
        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| {
                OutboxError::InvalidFilename("Could not extract filename".to_string())
            })?;

        // Format the message
        let message = format!("📬 **Agent Update** — `{}`\n\n{}", filename, content);

        // Send to Discord using the API client
        self.api_client
            .send_message_chunked(&self.channel_id, &message)
            .await
            .map_err(|e| OutboxError::DiscordApi(e.to_string()))?;

        // Move the file to the archive
        let archive_path = self.archive_path.join(filename);
        fs::rename(file_path, &archive_path)
            .await
            .map_err(|e| OutboxError::IoError(format!("Failed to move file to archive: {}", e)))?;

        tracing::info!(
            "Archived outbox file: {} -> {}",
            filename,
            archive_path.display()
        );

        Ok(())
    }

    /// Ensure the outbox and archive directories exist
    async fn ensure_directories(&self) -> Result<(), OutboxError> {
        // Create outbox directory if it doesn't exist
        if !self.outbox_path.exists() {
            fs::create_dir_all(&self.outbox_path).await.map_err(|e| {
                OutboxError::IoError(format!("Failed to create outbox directory: {}", e))
            })?;
        }

        // Create archive directory if it doesn't exist
        if !self.archive_path.exists() {
            fs::create_dir_all(&self.archive_path).await.map_err(|e| {
                OutboxError::IoError(format!("Failed to create archive directory: {}", e))
            })?;
        }

        Ok(())
    }
}

/// Errors that can occur during outbox processing
#[derive(Debug, thiserror::Error)]
pub enum OutboxError {
    /// I/O error (file read/write, directory operations)
    #[error("I/O error: {0}")]
    IoError(String),
    /// Discord API error
    #[error("Discord API error: {0}")]
    DiscordApi(String),
    /// Invalid filename
    #[error("Invalid filename: {0}")]
    InvalidFilename(String),
}
