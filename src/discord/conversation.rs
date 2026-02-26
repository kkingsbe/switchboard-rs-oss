//! Conversation management module.
//!
//! This module provides per-user conversation state management for the Discord
//! concierge. It maintains conversation history, handles trimming to prevent
//! unbounded memory growth, and implements TTL-based expiration for inactive
//! conversations.
//!
//! # Architecture
//!
//! - [`Conversation`] - Represents a single user's conversation state
//! - [`ConversationManager`] - Manages multiple conversations with TTL and trimming
//! - [`ChatMessage`] - OpenAI-format message structure for LLM communication
//!
//! # Usage
//!
//! ```ignore
//! use crate::discord::conversation::{ConversationManager, ChatMessage};
//!
//! let mut manager = ConversationManager::new(30, std::time::Duration::from_secs(7200));
//!
//! // Get or create a conversation for a user
//! let conv = manager.get_or_create_conversation("user123");
//!
//! // Add a user message
//! conv.add_message(ChatMessage::user("Hello, bot!"));
//!
//! // Get messages for LLM (includes system prompt)
//! let messages = conv.get_messages_for_llm(&system_prompt);
//! ```

use std::collections::HashMap;
use std::time::{Duration, Instant};

use tokio::time::interval;
use tokio::time::Duration as TokioDuration;

/// Configuration for conversation management.
///
/// Controls the maximum history size and TTL for conversations.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationConfig {
    /// Maximum messages to keep per conversation (default: 30)
    pub max_history: usize,
    /// Time-to-live in minutes before conversation expires (default: 120)
    pub ttl_minutes: u64,
}

impl Default for ConversationConfig {
    fn default() -> Self {
        Self {
            max_history: 30,
            ttl_minutes: 120,
        }
    }
}

impl ConversationConfig {
    /// Create a new config with custom values.
    pub fn new(max_history: usize, ttl_minutes: u64) -> Self {
        Self {
            max_history,
            ttl_minutes,
        }
    }

    /// Get the TTL as a Duration.
    pub fn ttl(&self) -> Duration {
        Duration::from_secs(self.ttl_minutes * 60)
    }
}

/// A single chat message in OpenAI format.
///
/// This enum represents the different roles in a conversation:
/// - System: Initial system prompt (injected at call time)
/// - User: Messages from the Discord user
/// - Assistant: Responses from the LLM
/// - Tool: Results from tool executions
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (system prompt)
    System,
    /// User message
    User,
    /// Assistant (LLM) message
    Assistant,
    /// Tool result message
    Tool,
}

/// A chat message with role and content.
///
/// # OpenAI Format
///
/// This struct is compatible with the OpenAI chat completions API format.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    /// The role of the message sender
    pub role: MessageRole,
    /// The content of the message
    pub content: String,
    /// Optional tool call information (for assistant messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Optional tool call ID (for tool result messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Optional name field (for tool messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// A tool call from the LLM.
///
/// Represents a request to execute a specific tool with given arguments.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// The tool to be called
    pub r#type: String,
    /// The function to execute
    pub function: ToolFunction,
}

/// Function call information from the LLM.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolFunction {
    /// The name of the function to call
    pub name: String,
    /// The arguments to pass to the function (JSON string)
    pub arguments: String,
}

impl ChatMessage {
    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Create an assistant message with tool calls.
    pub fn assistant_with_tools(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            name: None,
        }
    }

    /// Create a new tool result message.
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            name: None,
        }
    }

    /// Create a new tool result message with function name.
    pub fn tool_with_name(
        content: impl Into<String>,
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            name: Some(name.into()),
        }
    }

    /// Check if this message is a tool call request.
    pub fn is_tool_call(&self) -> bool {
        self.tool_calls.is_some() && !self.tool_calls.as_ref().unwrap().is_empty()
    }

    /// Get the text content, handling tool calls gracefully.
    pub fn text_content(&self) -> String {
        if self.is_tool_call() {
            if let Some(tool_calls) = &self.tool_calls {
                let calls: Vec<String> = tool_calls
                    .iter()
                    .map(|tc| format!("{}({})", tc.function.name, tc.function.arguments))
                    .collect();
                format!("[Tool calls: {}]", calls.join(", "))
            } else {
                String::new()
            }
        } else {
            self.content.clone()
        }
    }
}

/// A single user's conversation state.
///
/// Stores the conversation history and tracks the last activity time
/// for TTL expiration purposes.
#[derive(Debug)]
pub struct Conversation {
    /// The Discord user ID this conversation belongs to
    pub user_id: String,
    /// Message history (excluding system prompt - injected at call time)
    messages: Vec<ChatMessage>,
    /// Last activity timestamp
    last_active: Instant,
}

impl Conversation {
    /// Create a new conversation for a user.
    pub fn new(user_id: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            messages: Vec::new(),
            last_active: Instant::now(),
        }
    }

    /// Add a message to the conversation history.
    pub fn add_message(&mut self, message: ChatMessage) {
        self.last_active = Instant::now();
        self.messages.push(message);
    }

    /// Get a reference to the message history.
    pub fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    /// Get the last activity time.
    pub fn last_active(&self) -> Instant {
        self.last_active
    }

    /// Update the last active timestamp.
    pub fn touch(&mut self) {
        self.last_active = Instant::now();
    }

    /// Get messages formatted for LLM API calls.
    ///
    /// This prepends the system prompt to the conversation history.
    /// The system prompt is NOT stored in the conversation (to avoid
    /// duplication during trimming), but is injected at call time.
    pub fn get_messages_for_llm(&self, system_prompt: &str) -> Vec<ChatMessage> {
        let mut result = Vec::with_capacity(self.messages.len() + 1);

        // Add system prompt as first message
        result.push(ChatMessage {
            role: MessageRole::System,
            content: system_prompt.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        // Add conversation history
        result.extend(self.messages.iter().cloned());

        result
    }

    /// Trim the conversation history to max_history messages.
    ///
    /// Strategy:
    /// - Always keep the most recent messages
    /// - This preserves context from recent interactions
    /// - Note: System prompt is handled at call time, not stored here
    pub fn trim(&mut self, max_history: usize) {
        if self.messages.len() > max_history {
            // Keep the most recent messages
            let to_remove = self.messages.len() - max_history;
            self.messages.drain(0..to_remove);
        }
    }

    /// Check if the conversation has expired based on TTL.
    pub fn is_expired(&self, ttl: Duration) -> bool {
        Instant::now().duration_since(self.last_active) > ttl
    }

    /// Get the number of messages in the conversation.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the conversation is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

/// Manages multiple user conversations with TTL and trimming.
///
/// This is the main entry point for conversation management. It provides:
/// - Per-user conversation state
/// - Automatic trimming of old messages
/// - TTL-based conversation expiration
/// - Background cleanup support
#[derive(Debug)]
pub struct ConversationManager {
    /// Map of user_id to conversation
    conversations: HashMap<String, Conversation>,
    /// Maximum messages to keep per conversation
    max_history: usize,
    /// Time-to-live for conversations
    ttl: Duration,
}

impl ConversationManager {
    /// Create a new conversation manager with configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for max history and TTL
    pub fn new(config: ConversationConfig) -> Self {
        Self {
            conversations: HashMap::new(),
            max_history: config.max_history,
            ttl: config.ttl(),
        }
    }

    /// Create a new conversation manager (legacy constructor).
    ///
    /// # Arguments
    ///
    /// * `max_history` - Maximum messages to keep per conversation
    /// * `ttl` - Time-to-live for conversations
    #[deprecated(
        since = "0.1.0",
        note = "Use ConversationManager::new(ConversationConfig) instead"
    )]
    pub fn new_with_params(max_history: usize, ttl: Duration) -> Self {
        Self {
            conversations: HashMap::new(),
            max_history,
            ttl,
        }
    }

    /// Get or create a conversation for a user.
    ///
    /// If the conversation exists and hasn't expired, returns it.
    /// If it has expired, creates a new conversation.
    /// If it doesn't exist, creates a new one.
    pub fn get_or_create_conversation(&mut self, user_id: &str) -> &mut Conversation {
        use std::collections::hash_map::Entry;

        let ttl = self.ttl;
        let user_id_owned = user_id.to_string();

        match self.conversations.entry(user_id_owned) {
            Entry::Occupied(entry) => {
                let conv = entry.into_mut();
                if conv.is_expired(ttl) {
                    *conv = Conversation::new(user_id);
                } else {
                    conv.touch();
                }
                conv
            }
            Entry::Vacant(entry) => entry.insert(Conversation::new(user_id)),
        }
    }

    /// Add a user message to a conversation.
    ///
    /// Creates the conversation if it doesn't exist.
    /// Also performs trimming if history exceeds max_history.
    pub fn add_user_message(&mut self, user_id: &str, content: &str) {
        let max_history = self.max_history;
        let conv = self.get_or_create_conversation(user_id);
        conv.add_message(ChatMessage::user(content));
        conv.trim(max_history);
    }

    /// Add an assistant message to a conversation.
    pub fn add_assistant_message(&mut self, user_id: &str, content: &str) {
        let max_history = self.max_history;
        let conv = self.get_or_create_conversation(user_id);
        conv.add_message(ChatMessage::assistant(content));
        conv.trim(max_history);
    }

    /// Add an assistant message with tool calls to a conversation.
    pub fn add_assistant_message_with_tools(
        &mut self,
        user_id: &str,
        content: &str,
        tool_calls: Vec<ToolCall>,
    ) {
        let max_history = self.max_history;
        let conv = self.get_or_create_conversation(user_id);
        conv.add_message(ChatMessage::assistant_with_tools(content, tool_calls));
        conv.trim(max_history);
    }

    /// Add a tool result message to a conversation.
    pub fn add_tool_result(&mut self, user_id: &str, content: &str, tool_call_id: &str) {
        let max_history = self.max_history;
        let conv = self.get_or_create_conversation(user_id);
        conv.add_message(ChatMessage::tool(content, tool_call_id));
        conv.trim(max_history);
    }

    /// Get messages for LLM, including system prompt.
    pub fn get_messages_for_llm(
        &self,
        user_id: &str,
        system_prompt: &str,
    ) -> Option<Vec<ChatMessage>> {
        self.conversations
            .get(user_id)
            .map(|conv| conv.get_messages_for_llm(system_prompt))
    }

    /// Clean up expired conversations.
    ///
    /// Returns the number of conversations removed.
    pub fn cleanup_expired(&mut self) -> usize {
        let ttl = self.ttl;
        let before = self.conversations.len();

        self.conversations.retain(|_, conv| !conv.is_expired(ttl));

        before - self.conversations.len()
    }

    /// Get the number of active conversations.
    pub fn conversation_count(&self) -> usize {
        self.conversations.len()
    }

    /// Get the configuration max history.
    pub fn max_history(&self) -> usize {
        self.max_history
    }

    /// Get the TTL duration.
    pub fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Clear all conversations.
    pub fn clear(&mut self) {
        self.conversations.clear();
    }

    /// Check if a conversation exists for a user.
    pub fn has_conversation(&self, user_id: &str) -> bool {
        self.conversations.contains_key(user_id)
    }

    /// Add a message to a conversation by role and content.
    ///
    /// Creates the conversation if it doesn't exist.
    /// Also performs trimming if history exceeds max_history.
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user ID
    /// * `role` - The message role ("system", "user", "assistant", "tool")
    /// * `content` - The message content
    pub fn add_message(&mut self, user_id: &str, role: &str, content: &str) {
        let role = match role.to_lowercase().as_str() {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "tool" => MessageRole::Tool,
            _ => MessageRole::User, // Default to user
        };

        let message = ChatMessage {
            role,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        };

        let max_history = self.max_history;
        let conv = self.get_or_create_conversation(user_id);
        conv.add_message(message);
        conv.trim(max_history);
    }

    /// Get the user's message history.
    ///
    /// Returns None if no conversation exists for the user.
    pub fn get_history(&self, user_id: &str) -> Option<Vec<ChatMessage>> {
        self.conversations
            .get(user_id)
            .map(|conv| conv.messages().to_vec())
    }

    /// Trim the conversation history to max_history.
    ///
    /// This ensures the conversation doesn't exceed the maximum history size.
    /// Strategy: Always keep the most recent messages.
    pub fn trim_history(&mut self, conversation: &mut Conversation) {
        conversation.trim(self.max_history);
    }

    /// Start background cleanup task.
    ///
    /// Runs a background task that periodically cleans up expired conversations.
    /// The cleanup runs every 5 minutes.
    ///
    /// # Arguments
    ///
    /// * `manager` - The conversation manager to clean up (Arc wrapper for sharing)
    ///
    /// # Returns
    ///
    /// A handle to the background task
    pub fn start_background_cleanup(
        manager: std::sync::Arc<tokio::sync::Mutex<Self>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut cleanup_interval = interval(TokioDuration::from_secs(300)); // 5 minutes

            loop {
                cleanup_interval.tick().await;

                let mut guard = manager.lock().await;
                let cleaned = guard.cleanup_expired();
                if cleaned > 0 {
                    tracing::debug!("Cleaned up {} expired conversations", cleaned);
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_user() {
        let msg = ChatMessage::user("Hello");
        assert!(matches!(msg.role, MessageRole::User));
        assert_eq!(msg.content, "Hello");
        assert!(!msg.is_tool_call());
    }

    #[test]
    fn test_chat_message_assistant() {
        let msg = ChatMessage::assistant("Hi there!");
        assert!(matches!(msg.role, MessageRole::Assistant));
        assert_eq!(msg.content, "Hi there!");
    }

    #[test]
    fn test_chat_message_tool() {
        let msg = ChatMessage::tool("Tool result", "call_123");
        assert!(matches!(msg.role, MessageRole::Tool));
        assert_eq!(msg.content, "Tool result");
        assert_eq!(msg.tool_call_id, Some("call_123".to_string()));
    }

    #[test]
    fn test_conversation_creation() {
        let conv = Conversation::new("user123");
        assert_eq!(conv.user_id, "user123");
        assert!(conv.is_empty());
        assert_eq!(conv.len(), 0);
    }

    #[test]
    fn test_conversation_add_message() {
        let mut conv = Conversation::new("user123");
        conv.add_message(ChatMessage::user("Hello"));
        conv.add_message(ChatMessage::assistant("Hi!"));

        assert_eq!(conv.len(), 2);
    }

    #[test]
    fn test_conversation_trim() {
        let mut conv = Conversation::new("user123");

        // Add more messages than max_history
        for i in 0..35 {
            conv.add_message(ChatMessage::user(format!("Message {}", i)));
        }

        assert_eq!(conv.len(), 35);

        // Trim to max 30
        conv.trim(30);

        assert_eq!(conv.len(), 30);
        // Should keep most recent messages
        assert!(conv.messages()[0].content.contains("Message 5"));
    }

    #[test]
    fn test_conversation_get_messages_for_llm() {
        let mut conv = Conversation::new("user123");
        conv.add_message(ChatMessage::user("Hello"));
        conv.add_message(ChatMessage::assistant("Hi!"));

        let messages = conv.get_messages_for_llm("You are a helpful assistant.");

        assert_eq!(messages.len(), 3); // system + 2 messages
        assert!(matches!(messages[0].role, MessageRole::System));
        assert_eq!(messages[0].content, "You are a helpful assistant.");
    }

    #[test]
    fn test_conversation_manager_new() {
        let config = ConversationConfig::new(30, 2); // 2 minutes TTL
        let manager = ConversationManager::new(config);
        assert_eq!(manager.conversation_count(), 0);
        assert_eq!(manager.max_history(), 30);
    }

    #[test]
    fn test_conversation_manager_get_or_create() {
        let config = ConversationConfig::new(30, 2); // 2 minutes TTL
        let mut manager = ConversationManager::new(config);

        let conv1 = manager.get_or_create_conversation("user1");
        assert_eq!(conv1.user_id, "user1");
        assert_eq!(manager.conversation_count(), 1);

        // Getting same user should return existing conversation
        let conv2 = manager.get_or_create_conversation("user1");
        assert_eq!(conv2.user_id, "user1");
        assert_eq!(manager.conversation_count(), 1);
    }

    #[test]
    fn test_conversation_manager_add_messages() {
        let config = ConversationConfig::new(30, 2); // 2 minutes TTL
        let mut manager = ConversationManager::new(config);

        manager.add_user_message("user1", "Hello");
        manager.add_assistant_message("user1", "Hi!");

        let messages = manager
            .get_messages_for_llm("user1", "System prompt")
            .unwrap();
        assert_eq!(messages.len(), 3); // system + 2
    }

    #[test]
    fn test_conversation_manager_concurrent_conversations() {
        let config = ConversationConfig::new(30, 2); // 2 minutes TTL
        let mut manager = ConversationManager::new(config);

        // Create multiple conversations for different users simultaneously
        manager.add_user_message("user1", "Hello from user1");
        manager.add_user_message("user2", "Hello from user2");
        manager.add_user_message("user3", "Hello from user3");

        // Verify all conversations exist
        assert_eq!(manager.conversation_count(), 3);
        assert!(manager.has_conversation("user1"));
        assert!(manager.has_conversation("user2"));
        assert!(manager.has_conversation("user3"));

        // Add messages to each conversation independently
        manager.add_assistant_message("user1", "Response to user1");
        manager.add_assistant_message("user2", "Response to user2");
        manager.add_user_message("user1", "Another message from user1");

        // Verify conversation isolation - each should have its own messages
        let user1_messages = manager.get_history("user1").unwrap();
        let user2_messages = manager.get_history("user2").unwrap();
        let user3_messages = manager.get_history("user3").unwrap();

        // User1 should have 3 messages (2 user + 1 assistant)
        assert_eq!(user1_messages.len(), 3);
        // User2 should have 2 messages (1 user + 1 assistant)
        assert_eq!(user2_messages.len(), 2);
        // User3 should have 1 message (1 user)
        assert_eq!(user3_messages.len(), 1);

        // Verify message content - no leakage between conversations
        let user1_content: Vec<&str> = user1_messages.iter().map(|m| m.content.as_str()).collect();
        assert!(user1_content.contains(&"Hello from user1"));
        assert!(user1_content.contains(&"Response to user1"));
        assert!(user1_content.contains(&"Another message from user1"));
        // Verify user1 does NOT have user2's or user3's messages
        assert!(!user1_content.contains(&"Hello from user2"));
        assert!(!user1_content.contains(&"Response to user2"));
        assert!(!user1_content.contains(&"Hello from user3"));

        let user2_content: Vec<&str> = user2_messages.iter().map(|m| m.content.as_str()).collect();
        assert!(user2_content.contains(&"Hello from user2"));
        assert!(user2_content.contains(&"Response to user2"));
        // Verify user2 does NOT have user1's messages
        assert!(!user2_content.contains(&"Hello from user1"));
        assert!(!user2_content.contains(&"Response to user1"));

        // Verify LLM messages include correct system prompt
        let llm_messages_user1 = manager
            .get_messages_for_llm("user1", "You are a helpful bot.")
            .unwrap();
        assert_eq!(llm_messages_user1.len(), 4); // system + 3 messages
        assert!(matches!(llm_messages_user1[0].role, MessageRole::System));
        assert_eq!(llm_messages_user1[0].content, "You are a helpful bot.");

        // Verify each conversation maintains its own state independently
        // Adding more messages to user3 should not affect user1 or user2
        manager.add_user_message("user3", "More from user3");
        manager.add_user_message("user3", "Even more from user3");

        let user1_after = manager.get_history("user1").unwrap();
        let user3_after = manager.get_history("user3").unwrap();

        // User1 should still have 3 messages
        assert_eq!(user1_after.len(), 3);
        // User3 should now have 3 messages
        assert_eq!(user3_after.len(), 3);

        // Verify no cross-contamination
        let user1_content_after: Vec<&str> =
            user1_after.iter().map(|m| m.content.as_str()).collect();
        assert!(!user1_content_after.contains(&"More from user3"));
        assert!(!user1_content_after.contains(&"Even more from user3"));
    }

    #[test]
    fn test_conversation_manager_cleanup_expired() {
        let config = ConversationConfig::new(30, 0); // 0 minutes = immediate expiration
        let mut manager = ConversationManager::new(config);

        manager.add_user_message("user1", "Hello");
        manager.add_user_message("user2", "Hi");

        assert_eq!(manager.conversation_count(), 2);

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(100));

        let cleaned = manager.cleanup_expired();
        assert_eq!(cleaned, 2);
        assert_eq!(manager.conversation_count(), 0);
    }

    #[test]
    fn test_conversation_is_expired() {
        let conv = Conversation::new("user1");

        // New conversation should not be expired
        assert!(!conv.is_expired(Duration::from_secs(60)));

        // Old conversation should be expired
        let mut old_conv = Conversation::new("user2");
        // Manually set last_active to the past
        old_conv.last_active = Instant::now() - Duration::from_secs(120);
        assert!(old_conv.is_expired(Duration::from_secs(60)));
    }

    #[test]
    fn test_conversation_manager_clear() {
        let config = ConversationConfig::new(30, 2); // 2 minutes TTL
        let mut manager = ConversationManager::new(config);

        manager.add_user_message("user1", "Hello");
        manager.add_user_message("user2", "Hi");

        assert_eq!(manager.conversation_count(), 2);

        manager.clear();

        assert_eq!(manager.conversation_count(), 0);
    }

    #[test]
    fn test_message_text_content() {
        let user_msg = ChatMessage::user("Hello");
        assert_eq!(user_msg.text_content(), "Hello");

        let assistant_msg = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant_msg.text_content(), "Hi there!");
    }

    #[test]
    fn test_message_with_tool_calls() {
        let tool_calls = vec![ToolCall {
            id: "call_123".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "get_status".to_string(),
                arguments: "{}".to_string(),
            },
        }];

        let msg = ChatMessage::assistant_with_tools("I'll check that for you.", tool_calls);

        assert!(msg.is_tool_call());
        assert_eq!(msg.text_content(), "[Tool calls: get_status({})]");
    }
}
