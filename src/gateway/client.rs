//! Gateway client library for connecting to the gateway server.
//!
//! This module provides a WebSocket client for projects to connect to the gateway,
//! handle message sending/receiving, and automatic heartbeat management.

use crate::gateway::protocol::GatewayMessage;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
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

    /// Client is already connected.
    #[error("Client is already connected")]
    AlreadyConnected,

    /// Heartbeat task error.
    #[error("Heartbeat error: {0}")]
    HeartbeatError(String),

    /// Channel receive error.
    #[error("Channel error: {0}")]
    ChannelError(String),
}

/// Result type for gateway client operations.
pub type GatewayClientResult<T> = Result<T, GatewayClientError>;

/// Message received from the gateway.
#[derive(Debug, Clone)]
pub enum ReceivedMessage {
    /// A gateway protocol message.
    Gateway(GatewayMessage),
    /// A text message from the WebSocket.
    Text(String),
    /// A close message.
    Close,
    /// A ping message.
    Ping,
    /// A pong message.
    Pong,
    /// An error occurred.
    Error(String),
}

/// Configuration for the gateway client.
#[derive(Debug, Clone)]
pub struct GatewayClientConfig {
    /// Heartbeat interval in seconds.
    pub heartbeat_interval_secs: u64,
    /// Connection timeout in seconds.
    pub connection_timeout_secs: u64,
}

impl Default for GatewayClientConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 10,
        }
    }
}

/// Internal state of the gateway client.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ClientState {
    /// Client is disconnected.
    Disconnected,
    /// Client is connecting.
    Connecting,
    /// Client is connected.
    Connected,
    /// Client is disconnected due to an error.
    Error,
}

/// Type alias for the WebSocket stream.
type WsStream = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Gateway client for connecting to the gateway server.
///
/// This client provides:
/// - WebSocket connection management
/// - Message sending and receiving
/// - Automatic heartbeat in the background
///
/// # Example
///
/// ```ignore
/// let client = GatewayClient::new();
/// client.connect("ws://localhost:9000").await?;
///
/// while let Some(msg) = client.recv().await {
///     // Handle message
/// }
/// ```
pub struct GatewayClient {
    /// The WebSocket URL to connect to.
    url: String,
    /// The internal WebSocket stream for sending.
    ws_sender: Arc<RwLock<Option<futures_util::stream::SplitSink<WsStream, WsMessage>>>>,
    /// Channel for sending received messages to the recv() method.
    message_tx: mpsc::Sender<ReceivedMessage>,
    /// Channel for receiving messages in recv().
    message_rx: Arc<RwLock<Option<mpsc::Receiver<ReceivedMessage>>>>,
    /// Current state of the client.
    state: Arc<RwLock<ClientState>>,
    /// Handle for the heartbeat task.
    heartbeat_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
    /// Flag to control heartbeat.
    heartbeat_running: Arc<RwLock<bool>>,
    /// Client configuration.
    config: GatewayClientConfig,
}

impl GatewayClient {
    /// Create a new GatewayClient with default configuration.
    ///
    /// # Returns
    ///
    /// A new GatewayClient instance in disconnected state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = GatewayClient::new();
    /// ```
    pub fn new() -> Self {
        Self::with_config(GatewayClientConfig::default())
    }

    /// Create a new GatewayClient with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The client configuration.
    ///
    /// # Returns
    ///
    /// A new GatewayClient instance in disconnected state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = GatewayClientConfig {
    ///     heartbeat_interval_secs: 60,
    ///     connection_timeout_secs: 15,
    /// };
    /// let client = GatewayClient::with_config(config);
    /// ```
    pub fn with_config(config: GatewayClientConfig) -> Self {
        let (message_tx, _) = mpsc::channel(100);
        
        Self {
            url: String::new(),
            ws_sender: Arc::new(RwLock::new(None)),
            message_tx,
            message_rx: Arc::new(RwLock::new(None)),
            state: Arc::new(RwLock::new(ClientState::Disconnected)),
            heartbeat_handle: Arc::new(RwLock::new(None)),
            heartbeat_running: Arc::new(RwLock::new(false)),
            config,
        }
    }

    /// Connect to the gateway server.
    ///
    /// Establishes a WebSocket connection to the specified URL and starts
    /// the background message receiver task.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL to connect to (e.g., "ws://localhost:9000").
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Connection established successfully.
    /// * `Err(GatewayClientError)` - Connection failed.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = GatewayClient::new();
    /// client.connect("ws://localhost:9000").await?;
    /// ```
    pub async fn connect(&mut self, url: &str) -> GatewayClientResult<()> {
        // Check if already connected
        let current_state = self.state.read().await;
        if *current_state == ClientState::Connected {
            return Err(GatewayClientError::AlreadyConnected);
        }
        drop(current_state);

        // Update state to connecting
        {
            let mut state = self.state.write().await;
            *state = ClientState::Connecting;
        }

        info!(target: "gateway::client", url = %url, "Connecting to gateway server");

        // Connect to the WebSocket server
        let (ws_stream, _response) = connect_async(url)
            .await
            .map_err(|e| GatewayClientError::ConnectionError {
                url: url.to_string(),
                source: e,
            })?;

        info!(target: "gateway::client", url = %url, "Connected to gateway server");

        // Split the WebSocket stream into sender and receiver
        let (mut write, mut read) = ws_stream.split();

        // Create a channel for message handling
        let (tx, rx) = mpsc::channel::<ReceivedMessage>(100);
        
        // Store the sender for message sending
        {
            let mut sender = self.ws_sender.write().await;
            *sender = Some(write);
        }

        // Store the receiver for the recv() method
        {
            let mut receiver = self.message_rx.write().await;
            *receiver = Some(rx);
        }

        // Create a background task to read messages from the WebSocket
        let message_tx = tx;
        
        // Spawn the background receiver task
        tokio::spawn(async move {
            while let Some(msg_result) = read.next().await {
                let received = match msg_result {
                    Ok(WsMessage::Text(text)) => {
                        debug!(target: "gateway::client", "Received text message: {}", text);
                        ReceivedMessage::Text(text)
                    }
                    Ok(WsMessage::Close(_)) => {
                        info!(target: "gateway::client", "Received close message");
                        ReceivedMessage::Close
                    }
                    Ok(WsMessage::Ping(data)) => {
                        debug!(target: "gateway::client", "Received ping");
                        ReceivedMessage::Ping
                    }
                    Ok(WsMessage::Pong(_)) => {
                        debug!(target: "gateway::client", "Received pong");
                        ReceivedMessage::Pong
                    }
                    Ok(WsMessage::Binary(data)) => {
                        debug!(target: "gateway::client", "Received binary data: {} bytes", data.len());
                        ReceivedMessage::Error("Binary data not supported".to_string())
                    }
                    Err(e) => {
                        error!(target: "gateway::client", "WebSocket error: {}", e);
                        ReceivedMessage::Error(e.to_string())
                    }
                    _ => continue,
                };

                if message_tx.send(received).await.is_err() {
                    warn!(target: "gateway::client", "Failed to send message to channel, receiver dropped");
                    break;
                }
            }
        });

        // Update URL
        self.url = url.to_string();

        // Update state to connected
        {
            let mut state = self.state.write().await;
            *state = ClientState::Connected;
        }

        Ok(())
    }

    /// Receive the next message from the gateway.
    ///
    /// This method waits for a message to be received from the WebSocket
    /// and returns it. Messages include both protocol messages and control
    /// messages (close, ping, pong).
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ReceivedMessage))` - A message was received.
    /// * `Ok(None)` - The channel was closed (connection closed).
    /// * `Err(GatewayClientError)` - An error occurred.
    ///
    /// # Example
    ///
    /// ```ignore
    /// while let Some(msg) = client.recv().await {
    ///     match msg {
    ///         ReceivedMessage::Text(text) => {
    ///             // Parse as GatewayMessage
    ///         }
    ///         ReceivedMessage::Close => break,
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn recv(&mut self) -> GatewayClientResult<Option<ReceivedMessage>> {
        // Check if connected
        let current_state = self.state.read().await;
        if *current_state != ClientState::Connected {
            return Err(GatewayClientError::NotConnected);
        }
        drop(current_state);

        // Get the receiver
        let mut rx_guard = self.message_rx.write().await;
        if let Some(ref mut rx) = *rx_guard {
            match rx.recv().await {
                Some(msg) => Ok(Some(msg)),
                None => Ok(None),
            }
        } else {
            Err(GatewayClientError::NotConnected)
        }
    }

    /// Receive and parse the next GatewayMessage from the gateway.
    ///
    /// This is a convenience method that combines recv() with parsing
    /// the text message as a GatewayMessage.
    ///
    /// # Returns
    ///
    /// * `Ok(Some(GatewayMessage))` - A gateway message was received and parsed.
    /// * `Ok(None)` - Connection closed.
    /// * `Err(GatewayClientError)` - An error occurred.
    ///
    /// # Example
    ///
    /// ```ignore
    /// while let Ok(Some(msg)) = client.recv_message().await {
    ///     match msg {
    ///         GatewayMessage::RegisterAck { session_id, .. } => {
    ///             println!("Registered with session: {}", session_id);
    ///         }
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn recv_message(&mut self) -> GatewayClientResult<Option<GatewayMessage>> {
        match self.recv().await? {
            Some(ReceivedMessage::Text(text)) => {
                let msg = serde_json::from_str::<GatewayMessage>(&text)?;
                Ok(Some(msg))
            }
            Some(ReceivedMessage::Close) => Ok(None),
            Some(ReceivedMessage::Error(e)) => Err(GatewayClientError::ReceiveError(e)),
            _ => Ok(None),
        }
    }

    /// Send a message to the gateway.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to send.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully.
    /// * `Err(GatewayClientError)` - Failed to send message.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let msg = GatewayMessage::Register {
    ///     project_name: "my-project".to_string(),
    ///     channels: vec!["channel1".to_string()],
    /// };
    /// client.send(msg).await?;
    /// ```
    pub async fn send(&mut self, message: GatewayMessage) -> GatewayClientResult<()> {
        // Check if connected
        let current_state = self.state.read().await;
        if *current_state != ClientState::Connected {
            return Err(GatewayClientError::NotConnected);
        }
        drop(current_state);

        // Serialize the message
        let json = serde_json::to_string(&message)
            .map_err(GatewayClientError::ParseError)?;

        debug!(target: "gateway::client", "Sending message: {}", json);

        // Send the message
        let mut sender_guard = self.ws_sender.write().await;
        if let Some(ref mut sender) = *sender_guard {
            sender
                .send(WsMessage::Text(json))
                .await
                .map_err(|e| GatewayClientError::SendError(e.to_string()))?;
            Ok(())
        } else {
            Err(GatewayClientError::NotConnected)
        }
    }

    /// Send a raw text message to the gateway.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to send.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Message sent successfully.
    /// * `Err(GatewayClientError)` - Failed to send message.
    pub async fn send_text(&mut self, text: &str) -> GatewayClientResult<()> {
        // Check if connected
        let current_state = self.state.read().await;
        if *current_state != ClientState::Connected {
            return Err(GatewayClientError::NotConnected);
        }
        drop(current_state);

        // Send the message
        let mut sender_guard = self.ws_sender.write().await;
        if let Some(ref mut sender) = *sender_guard {
            sender
                .send(WsMessage::Text(text.to_string()))
                .await
                .map_err(|e| GatewayClientError::SendError(e.to_string()))?;
            Ok(())
        } else {
            Err(GatewayClientError::NotConnected)
        }
    }

    /// Start the automatic heartbeat task.
    ///
    /// This method spawns a background task that sends heartbeat messages
    /// at the configured interval.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Heartbeat started successfully.
    /// * `Err(GatewayClientError)` - Heartbeat already running or not connected.
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.connect("ws://localhost:9000").await?;
    /// client.start_heartbeat().await?;
    /// ```
    pub async fn start_heartbeat(&mut self) -> GatewayClientResult<()> {
        // Check if already running
        {
            let running = self.heartbeat_running.read().await;
            if *running {
                return Err(GatewayClientError::HeartbeatError(
                    "Heartbeat already running".to_string(),
                ));
            }
        }

        // Check if connected
        {
            let current_state = self.state.read().await;
            if *current_state != ClientState::Connected {
                return Err(GatewayClientError::NotConnected);
            }
        }

        // Mark as running
        {
            let mut running = self.heartbeat_running.write().await;
            *running = true;
        }

        // Clone necessary data for the heartbeat task
        let ws_sender = self.ws_sender.clone();
        let heartbeat_running = self.heartbeat_running.clone();
        let state = self.state.clone();
        let interval = self.config.heartbeat_interval_secs;

        info!(target: "gateway::client", interval_secs = interval, "Starting heartbeat task");

        // Spawn the heartbeat task
        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(std::time::Duration::from_secs(interval));
            
            loop {
                interval_timer.tick().await;
                
                // Check if we should continue
                {
                    let running = heartbeat_running.read().await;
                    if !*running {
                        info!(target: "gateway::client", "Heartbeat task stopping");
                        break;
                    }
                }

                // Check if still connected
                {
                    let current_state = state.read().await;
                    if *current_state != ClientState::Connected {
                        warn!(target: "gateway::client", "Heartbeat task stopping - not connected");
                        break;
                    }
                }

                // Send heartbeat
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                
                let heartbeat = GatewayMessage::Heartbeat { timestamp };
                let json = match serde_json::to_string(&heartbeat) {
                    Ok(j) => j,
                    Err(e) => {
                        error!(target: "gateway::client", "Failed to serialize heartbeat: {}", e);
                        continue;
                    }
                };

                let mut sender_guard = ws_sender.write().await;
                if let Some(ref mut sender) = *sender_guard {
                    match sender.send(WsMessage::Text(json)).await {
                        Ok(_) => {
                            debug!(target: "gateway::client", "Heartbeat sent");
                        }
                        Err(e) => {
                            error!(target: "gateway::client", "Failed to send heartbeat: {}", e);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }

            // Mark as not running
            {
                let mut running = heartbeat_running.write().await;
                *running = false;
            }
        });

        // Store the handle
        {
            let mut handle_guard = self.heartbeat_handle.write().await;
            *handle_guard = Some(handle);
        }

        Ok(())
    }

    /// Stop the automatic heartbeat task.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Heartbeat stopped successfully.
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.stop_heartbeat().await?;
    /// ```
    pub async fn stop_heartbeat(&mut self) -> GatewayClientResult<()> {
        // Mark as not running
        {
            let mut running = self.heartbeat_running.write().await;
            *running = false;
        }

        // Wait for the task to finish
        {
            let mut handle_guard = self.heartbeat_handle.write().await;
            if let Some(handle) = handle_guard.take() {
                let _ = handle.await;
            }
        }

        info!(target: "gateway::client", "Heartbeat stopped");

        Ok(())
    }

    /// Check if the client is connected.
    ///
    /// # Returns
    ///
    /// * `true` - Client is connected.
    /// * `false` - Client is not connected.
    pub async fn is_connected(&self) -> bool {
        let state = self.state.read().await;
        *state == ClientState::Connected
    }

    /// Disconnect from the gateway server.
    ///
    /// This method stops the heartbeat task, closes the WebSocket connection,
    /// and cleans up resources.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Disconnected successfully.
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.disconnect().await?;
    /// ```
    pub async fn disconnect(&mut self) -> GatewayClientResult<()> {
        // Stop heartbeat
        let _ = self.stop_heartbeat().await;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = ClientState::Disconnected;
        }

        // Close the WebSocket stream
        {
            let mut sender_guard = self.ws_sender.write().await;
            if let Some(ref mut sender) = *sender_guard {
                let _ = sender.close().await;
                *sender_guard = None;
            }
        }

        // Clear the receiver
        {
            let mut receiver_guard = self.message_rx.write().await;
            *receiver_guard = None;
        }

        info!(target: "gateway::client", "Disconnected from gateway server");

        Ok(())
    }

    /// Get the gateway URL.
    ///
    /// # Returns
    ///
    /// The URL string, or empty string if not connected.
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Default for GatewayClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GatewayClient {
    fn drop(&mut self) {
        // We can't do async operations in Drop, but we can at least log
        // In practice, users should call disconnect() before dropping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that GatewayClient can be instantiated with default configuration
    #[tokio::test]
    async fn test_gateway_client_new_should_create_instance_with_default_config() {
        let client = GatewayClient::new();
        
        // Client should not be connected initially
        assert!(!client.is_connected().await);
        
        // URL should be empty
        assert!(client.url().is_empty());
    }

    /// Test that GatewayClient can be instantiated with custom configuration
    #[tokio::test]
    async fn test_gateway_client_with_config_should_create_instance_with_custom_config() {
        let config = GatewayClientConfig {
            heartbeat_interval_secs: 60,
            connection_timeout_secs: 15,
        };
        let client = GatewayClient::with_config(config);
        
        // Client should not be connected initially
        assert!(!client.is_connected().await);
    }

    /// Test default configuration values
    #[test]
    fn test_default_config_should_have_correct_values() {
        let config = GatewayClientConfig::default();
        
        assert_eq!(config.heartbeat_interval_secs, 30);
        assert_eq!(config.connection_timeout_secs, 10);
    }

    /// Test client state transitions
    #[tokio::test]
    async fn test_client_should_transition_between_states() {
        let client = GatewayClient::new();
        
        // Initially disconnected
        assert!(!client.is_connected().await);
        
        // State should be Disconnected
        let state = client.state.read().await;
        assert_eq!(*state, ClientState::Disconnected);
    }

    /// Test error type display implementation
    #[test]
    fn test_error_display_should_show_meaningful_message() {
        let error = GatewayClientError::NotConnected;
        assert_eq!(format!("{}", error), "Client is not connected");
        
        let error = GatewayClientError::AlreadyConnected;
        assert_eq!(format!("{}", error), "Client is already connected");
    }

    /// Test heartbeat error display
    #[test]
    fn test_heartbeat_error_display_should_show_message() {
        let error = GatewayClientError::HeartbeatError("test error".to_string());
        assert_eq!(format!("{}", error), "Heartbeat error: test error");
    }

    /// Test connection error includes URL
    #[test]
    fn test_connection_error_should_include_url() {
        let url = "ws://localhost:9000";
        let error = GatewayClientError::ConnectionError {
            url: url.to_string(),
            source: tokio_tungstenite::tungstenite::Error::ConnectionClosed,
        };
        
        assert!(format!("{}", error).contains(url));
    }

    /// Test send error when not connected
    #[tokio::test]
    async fn test_send_should_fail_when_not_connected() {
        let mut client = GatewayClient::new();
        
        let msg = GatewayMessage::Register {
            project_name: "test".to_string(),
            channels: vec![],
        };
        
        let result = client.send(msg).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            GatewayClientError::NotConnected => {}
            _ => panic!("Expected NotConnected error"),
        }
    }

    /// Test recv error when not connected
    #[tokio::test]
    async fn test_recv_should_fail_when_not_connected() {
        let mut client = GatewayClient::new();
        
        let result = client.recv().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            GatewayClientError::NotConnected => {}
            _ => panic!("Expected NotConnected error"),
        }
    }

    /// Test stop heartbeat when not running should succeed
    #[tokio::test]
    async fn test_stop_heartbeat_should_succeed_when_not_running() {
        let mut client = GatewayClient::new();
        
        // Should not error when heartbeat is not running
        let result = client.stop_heartbeat().await;
        assert!(result.is_ok());
    }

    /// Test start heartbeat when not connected should fail
    #[tokio::test]
    async fn test_start_heartbeat_should_fail_when_not_connected() {
        let mut client = GatewayClient::new();
        
        let result = client.start_heartbeat().await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            GatewayClientError::NotConnected => {}
            _ => panic!("Expected NotConnected error"),
        }
    }

    /// Test disconnect when not connected should succeed
    #[tokio::test]
    async fn test_disconnect_should_succeed_when_not_connected() {
        let mut client = GatewayClient::new();
        
        // Should not error when not connected
        let result = client.disconnect().await;
        assert!(result.is_ok());
    }

    /// Test GatewayMessage serialization for heartbeat
    #[test]
    fn test_heartbeat_message_serialization() {
        let msg = GatewayMessage::Heartbeat { timestamp: 1234567890 };
        let json = serde_json::to_string(&msg).unwrap();
        
        assert!(json.contains("\"type\":\"heartbeat\""));
        assert!(json.contains("1234567890"));
    }

    /// Test GatewayMessage deserialization for register
    #[test]
    fn test_register_message_deserialization() {
        let json = r#"{"type":"register","project_name":"test-project","channels":["ch1","ch2"]}"#;
        let msg: GatewayMessage = serde_json::from_str(json).unwrap();
        
        match msg {
            GatewayMessage::Register { project_name, channels } => {
                assert_eq!(project_name, "test-project");
                assert_eq!(channels, vec!["ch1", "ch2"]);
            }
            _ => panic!("Expected Register message"),
        }
    }

    /// Test ReceivedMessage enum variants
    #[test]
    fn test_received_message_variants() {
        let msg = ReceivedMessage::Text("test".to_string());
        let _ = format!("{:?}", msg);
        
        let msg = ReceivedMessage::Close;
        let _ = format!("{:?}", msg);
        
        let msg = ReceivedMessage::Ping;
        let _ = format!("{:?}", msg);
        
        let msg = ReceivedMessage::Pong;
        let _ = format!("{:?}", msg);
        
        let msg = ReceivedMessage::Error("error".to_string());
        let _ = format!("{:?}", msg);
        
        let msg = ReceivedMessage::Gateway(GatewayMessage::Heartbeat { timestamp: 0 });
        let _ = format!("{:?}", msg);
    }

    // ========== Integration Tests using WebSocket Echo Server ==========

    use tokio::net::TcpListener;
    use tokio::sync::oneshot;

    /// Start a simple WebSocket echo server for testing.
    /// Returns the server URL and a shutdown signal sender.
    async fn start_echo_server() -> (String, oneshot::Sender<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let url = format!("ws://127.0.0.1:{}", addr.port());

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            let mut shutdown_rx = shutdown_rx;
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        let (stream, _) = result.unwrap();
                        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
                        let (mut write, mut read) = ws_stream.split();

                        tokio::spawn(async move {
                            while let Some(msg) = read.next().await {
                                if let Ok(msg) = msg {
                                    if msg.is_text() || msg.is_binary() {
                                        let _ = write.send(msg).await;
                                    }
                                }
                            }
                        });
                    }
                    _ = &mut shutdown_rx => {
                        break;
                    }
                }
            }
        });

        (url, shutdown_tx)
    }

    /// Test that connect() establishes a WebSocket connection.
    #[tokio::test]
    async fn connect_should_establish_websocket_connection() {
        // Start echo server
        let (url, shutdown) = start_echo_server().await;

        // Create client and connect
        let mut client = GatewayClient::new();
        let result = client.connect(&url).await;

        // Verify connection succeeded
        assert!(result.is_ok(), "Expected connection to succeed: {:?}", result);
        assert!(client.is_connected().await, "Client should be connected");

        // Cleanup
        let _ = client.disconnect().await;
        let _ = shutdown.send(());
    }

    /// Test that recv() receives messages from the gateway.
    #[tokio::test]
    async fn recv_should_receive_messages_after_connection() {
        // Start echo server
        let (url, shutdown) = start_echo_server().await;

        // Create client and connect
        let mut client = GatewayClient::new();
        client.connect(&url).await.unwrap();

        // Send a message to the client via the echo server
        // First, we need to get a reference to the server to send a message
        // For this test, we'll use the echo server to echo back what we send
        let test_message = r#"{"type":"register","project_name":"test","channels":[]}"#;
        
        // Send a message and receive the echo
        client.send_text(test_message).await.unwrap();

        // Receive the echoed message
        let received = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.recv()
        ).await;

        // Verify we received a message
        assert!(received.is_ok(), "Should receive message within timeout");
        let result = received.unwrap();
        assert!(result.is_ok(), "recv should succeed: {:?}", result);
        let msg = result.unwrap();
        assert!(msg.is_some(), "Should receive a message");

        if let Some(ReceivedMessage::Text(text)) = msg {
            assert!(text.contains("register"), "Should receive echo of our message");
        }

        // Cleanup
        let _ = client.disconnect().await;
        let _ = shutdown.send(());
    }

    /// Test that heartbeat is sent automatically in the background.
    #[tokio::test]
    async fn heartbeat_should_send_periodically() {
        // Start echo server
        let (url, shutdown) = start_echo_server().await;

        // Create client with short heartbeat interval
        let config = GatewayClientConfig {
            heartbeat_interval_secs: 1,
            connection_timeout_secs: 10,
        };
        let mut client = GatewayClient::with_config(config);

        // Connect and start heartbeat
        client.connect(&url).await.unwrap();
        client.start_heartbeat().await.unwrap();

        // Wait for heartbeat to be sent (at least 1.5 seconds to ensure at least one heartbeat)
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;

        // Receive a message - should be the heartbeat
        let received = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.recv()
        ).await;

        // Verify heartbeat was received
        assert!(received.is_ok(), "Should receive heartbeat within timeout");
        let result = received.unwrap();
        assert!(result.is_ok(), "recv should succeed: {:?}", result);
        let msg = result.unwrap();
        assert!(msg.is_some(), "Should receive a heartbeat message");

        if let Some(ReceivedMessage::Text(text)) = msg {
            assert!(text.contains("heartbeat"), "Should receive heartbeat message: {}", text);
        }

        // Cleanup
        let _ = client.stop_heartbeat().await;
        let _ = client.disconnect().await;
        let _ = shutdown.send(());
    }

    /// Test that connect fails with meaningful error for invalid URL.
    #[tokio::test]
    async fn connect_should_fail_for_invalid_url() {
        let mut client = GatewayClient::new();
        
        // Try to connect to invalid URL (non-existent server)
        let result = client.connect("ws://127.0.0.1:99999").await;
        
        // Should fail with connection error
        assert!(result.is_err());
        match result.unwrap_err() {
            GatewayClientError::ConnectionError { .. } => {}
            _ => panic!("Expected ConnectionError"),
        }
    }

    /// Test that client can send and receive GatewayMessage.
    #[tokio::test]
    async fn send_and_recv_message_should_work() {
        // Start echo server
        let (url, shutdown) = start_echo_server().await;

        // Create client and connect
        let mut client = GatewayClient::new();
        client.connect(&url).await.unwrap();

        // Create and send a GatewayMessage
        let msg = GatewayMessage::Register {
            project_name: "test-project".to_string(),
            channels: vec!["channel1".to_string()],
        };
        client.send(msg).await.unwrap();

        // Receive the echoed message
        let received = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.recv()
        ).await;

        assert!(received.is_ok(), "Should receive message within timeout");
        let result = received.unwrap();
        assert!(result.is_ok(), "recv should succeed: {:?}", result);
        let msg = result.unwrap();
        assert!(msg.is_some(), "Should receive a message");

        // Parse as GatewayMessage
        if let Some(ReceivedMessage::Text(text)) = msg {
            let parsed: GatewayMessage = serde_json::from_str(&text).unwrap();
            match parsed {
                GatewayMessage::Register { project_name, channels } => {
                    assert_eq!(project_name, "test-project");
                    assert_eq!(channels, vec!["channel1".to_string()]);
                }
                _ => panic!("Expected Register message"),
            }
        }

        // Cleanup
        let _ = client.disconnect().await;
        let _ = shutdown.send(());
    }
}
