//! Gateway client module for auto-connect to gateway server.
//!
//! This module provides automatic connection management for switchboard projects
//! to connect to the gateway WebSocket server and receive Discord messages.

use crate::config::env::resolve_config_value;
use crate::discord::config::GatewayClientConfig;
use crate::gateway::protocol::GatewayMessage;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info, warn};

/// Error types for gateway client operations.
#[derive(Debug, Error)]
pub enum GatewayClientError {
    /// Failed to connect to the gateway server.
    #[error("Failed to connect to gateway at {url}: {source}")]
    ConnectionError {
        /// The URL we tried to connect to.
        url: String,
        /// The underlying connection error.
        #[source]
        source: tokio_tungstenite::tungstenite::Error,
    },

    /// Failed to send a message.
    #[error("Failed to send message: {0}")]
    SendError(String),

    /// Failed to receive a message.
    #[error("Failed to receive message: {0}")]
    ReceiveError(String),

    /// Failed to parse a message.
    #[error("Failed to parse message: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Client is not connected.
    #[error("Client is not connected")]
    NotConnected,

    /// Registration failed.
    #[error("Registration failed: {0}")]
    RegistrationError(String),

    /// Shutdown signal received.
    #[error("Shutdown signal received")]
    Shutdown,
}

/// Result type for gateway client operations.
pub type GatewayClientResult<T> = Result<T, GatewayClientError>;

/// Message received from the gateway.
#[derive(Debug, Clone)]
pub enum GatewayClientMessage {
    /// A gateway protocol message.
    Gateway(GatewayMessage),
    /// A text message from the WebSocket.
    Text(String),
    /// Connection established.
    Connected,
    /// Connection closed.
    Disconnected,
    /// An error occurred.
    Error(String),
}

/// Gateway connection manager for auto-connect functionality.
///
/// This struct manages the WebSocket connection to the gateway server,
/// including registration, reconnection, and message handling.
pub struct GatewayConnection {
    /// Configuration for the gateway connection.
    config: GatewayClientConfig,
    /// Channel for receiving messages from the gateway.
    message_rx: mpsc::Receiver<GatewayClientMessage>,
    /// Channel for sending messages to be processed by the caller.
    pub(crate) outgoing_tx: mpsc::Sender<GatewayClientMessage>,
    /// Flag indicating if the client is connected.
    is_connected: Arc<RwLock<bool>>,
    /// Handle to the connection task.
    task_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    /// Shutdown receiver.
    shutdown_rx: Arc<RwLock<Option<broadcast::Receiver<()>>>>,
}

impl GatewayConnection {
    /// Create a new gateway connection manager.
    ///
    /// # Arguments
    ///
    /// * `config` - Gateway client configuration
    /// * `shutdown_rx` - Optional broadcast receiver for shutdown signal
    ///
    /// # Returns
    ///
    /// * `Self` - The new gateway connection manager
    pub fn new(
        config: GatewayClientConfig,
        shutdown_rx: Option<broadcast::Receiver<()>>,
    ) -> Self {
        let (outgoing_tx, message_rx) = mpsc::channel(100);

        Self {
            config,
            message_rx,
            outgoing_tx,
            is_connected: Arc::new(RwLock::new(false)),
            task_handle: Arc::new(Mutex::new(None)),
            shutdown_rx: Arc::new(RwLock::new(shutdown_rx)),
        }
    }

    /// Check if the client is connected.
    ///
    /// # Returns
    ///
    /// * `bool` - True if connected, false otherwise
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    /// Start the gateway connection in the background.
    ///
    /// This method spawns a task that manages the WebSocket connection,
    /// including initial connection, registration, and reconnection logic.
    pub async fn start(&mut self) -> GatewayClientResult<()> {
        let config = self.config.clone();
        let is_connected = Arc::clone(&self.is_connected);
        let outgoing_tx = self.outgoing_tx.clone();
        let shutdown_rx = Arc::clone(&self.shutdown_rx);

        // Spawn the connection task
        let handle = tokio::spawn(async move {
            let mut retry_count = 0;
            let max_retries = 10;
            let base_delay = std::time::Duration::from_secs(1);

            loop {
                // Resolve the URL with environment variables
                let url = resolve_config_value(&config.url);
                info!("Gateway client: Connecting to {}", url);

                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        info!("Gateway client: Connected to {}", url);

                        // Update connected state
                        *is_connected.write().await = true;
                        let _ = outgoing_tx.send(GatewayClientMessage::Connected).await;

                        // Reset retry count on successful connection
                        retry_count = 0;

                        // Split the WebSocket stream into sender and receiver
                        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                        // Send registration message
                        let register_msg = GatewayMessage::Register {
                            project_name: config.project_name.clone(),
                            channels: config.channels.clone(),
                        };

                        if let Err(e) = ws_sender
                            .send(WsMessage::Text(
                                serde_json::to_string(&register_msg).unwrap(),
                            ))
                            .await
                        {
                            error!("Gateway client: Failed to send registration: {:?}", e);
                        } else {
                            info!("Gateway client: Registration sent");
                        }

                        // Process incoming messages
                        loop {
                            tokio::select! {
                                msg = ws_receiver.next() => {
                                    match msg {
                                        Some(Ok(WsMessage::Text(text))) => {
                                            debug!("Gateway client: Received: {}", text);

                                            // Try to parse as gateway message
                                            match serde_json::from_str::<GatewayMessage>(&text) {
                                                Ok(gateway_msg) => {
                                                    match &gateway_msg {
                                                        GatewayMessage::RegisterAck { status, session_id } => {
                                                            info!("Gateway client: Registration acknowledged - status: {}, session_id: {}", status, session_id);
                                                        }
                                                        GatewayMessage::RegisterError { error } => {
                                                            error!("Gateway client: Registration failed: {}", error);
                                                            let _ = outgoing_tx.send(GatewayClientMessage::Error(error.clone())).await;
                                                        }
                                                        _ => {}
                                                    }
                                                    let _ = outgoing_tx.send(GatewayClientMessage::Gateway(gateway_msg)).await;
                                                }
                                                Err(e) => {
                                                    // Not a gateway message, send as text
                                                    warn!("Gateway client: Failed to parse message: {}", e);
                                                    let _ = outgoing_tx.send(GatewayClientMessage::Text(text)).await;
                                                }
                                            }
                                        }
                                        Some(Ok(WsMessage::Close(_))) => {
                                            info!("Gateway client: Connection closed by server");
                                            break;
                                        }
                                        Some(Ok(WsMessage::Ping(data))) => {
                                            debug!("Gateway client: Received ping");
                                            // Note: tungstenite automatically responds with pong
                                        }
                                        Some(Ok(WsMessage::Pong(_))) => {
                                            debug!("Gateway client: Received pong");
                                        }
                                        Some(Err(e)) => {
                                            error!("Gateway client: WebSocket error: {}", e);
                                            let _ = outgoing_tx.send(GatewayClientMessage::Error(e.to_string())).await;
                                            break;
                                        }
                                        None => {
                                            info!("Gateway client: WebSocket stream ended");
                                            break;
                                        }
                                        _ => {}
                                    }
                                }
                                _ = tokio::time::sleep(std::time::Duration::from_secs(30)) => {
                                    // Heartbeat - send a heartbeat message
                                    let heartbeat = GatewayMessage::Heartbeat {
                                        timestamp: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs(),
                                    };
                                    // Note: In a full implementation, we'd need to send this
                                    // For now, we just keep the connection alive
                                    debug!("Gateway client: Heartbeat");
                                }
                            }
                        }

                        // Update connected state
                        *is_connected.write().await = false;
                        let _ = outgoing_tx.send(GatewayClientMessage::Disconnected).await;
                    }
                    Err(e) => {
                        error!("Gateway client: Failed to connect to {}: {}", url, e);
                        *is_connected.write().await = false;
                    }
                }

                // Exponential backoff for reconnection
                retry_count += 1;
                if retry_count > max_retries {
                    error!("Gateway client: Max retries exceeded, giving up");
                    let _ = outgoing_tx
                        .send(GatewayClientMessage::Error(
                            "Max connection retries exceeded".to_string(),
                        ))
                        .await;
                    break;
                }

                let delay = base_delay * 2u32.pow(retry_count.min(5) as u32);
                info!(
                    "Gateway client: Retrying in {} seconds (attempt {}/{})",
                    delay.as_secs(),
                    retry_count,
                    max_retries
                );
                tokio::time::sleep(delay).await;
            }

            info!("Gateway client: Connection task ended");
        });

        *self.task_handle.lock().await = Some(handle);
        Ok(())
    }

    /// Stop the gateway connection.
    ///
    /// This method signals the connection task to stop and waits for it to complete.
    pub async fn stop(&mut self) {
        info!("Gateway client: Stopping connection");

        // Update connected state
        *self.is_connected.write().await = false;

        // Cancel the task
        if let Some(handle) = self.task_handle.lock().await.take() {
            handle.abort();
        }
    }

    /// Receive a message from the gateway.
    ///
    /// # Returns
    ///
    /// * `Some(GatewayClientMessage)` - A message from the gateway
    /// * `None` - If the channel is closed
    pub async fn recv(&mut self) -> Option<GatewayClientMessage> {
        self.message_rx.recv().await
    }
}

/// Create a new gateway connection with the given configuration.
///
/// This is a convenience function that creates and starts a gateway connection.
///
/// # Arguments
///
/// * `config` - Gateway client configuration
/// * `shutdown_rx` - Optional broadcast receiver for shutdown signal
///
/// # Returns
///
/// * `GatewayClientResult<GatewayConnection>` - The started gateway connection
pub async fn create_gateway_connection(
    config: GatewayClientConfig,
    shutdown_rx: Option<broadcast::Receiver<()>>,
) -> GatewayClientResult<GatewayConnection> {
    let mut connection = GatewayConnection::new(config, shutdown_rx);
    connection.start().await?;
    Ok(connection)
}
