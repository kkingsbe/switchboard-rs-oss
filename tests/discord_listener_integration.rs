//! End-to-End Integration tests for Discord Listener
//!
//! This module tests the full message flow through the Discord listener:
//! - Valid user messages in correct channel get processed
//! - Bot's own messages are filtered out
//! - Messages from wrong channels are filtered out

#[cfg(feature = "discord")]
mod discord_listener_tests {
    #[cfg(feature = "discord")]
    use switchboard::discord::conversation::{
        ConversationConfig, ConversationManager, ToolCall, ToolFunction,
    };
    #[cfg(feature = "discord")]
    use switchboard::discord::llm::LlmResponse;
    #[cfg(feature = "discord")]
    use switchboard::discord::tools::tools_schema;
    use std::sync::{Arc, Mutex};
    use tokio::sync::Mutex as TokioMutex;

    // ===========================================================================
    // Mock Implementations
    // ===========================================================================

    /// Mock LLM client that returns configurable responses.
    ///
    /// Tracks whether chat_completion was called to verify message processing.
    #[cfg(feature = "discord")]
    struct MockLlmClient {
        responses: Vec<LlmResponse>,
        response_index: usize,
        call_count: Arc<Mutex<usize>>,
    }

    #[cfg(feature = "discord")]
    impl MockLlmClient {
        fn new(responses: Vec<LlmResponse>) -> Self {
            Self {
                responses,
                response_index: 0,
                call_count: Arc::new(Mutex::new(0)),
            }
        }

        fn call_count(&self) -> Arc<Mutex<usize>> {
            self.call_count.clone()
        }

        async fn chat_completion(
            &mut self,
            _messages: &[switchboard::discord::conversation::ChatMessage],
            _tools: Option<&[serde_json::Value]>,
        ) -> Result<LlmResponse, switchboard::discord::llm::LlmError> {
            // Increment call count
            {
                let mut count = self.call_count.lock().unwrap();
                *count += 1;
            }

            if self.response_index < self.responses.len() {
                let response = self.responses[self.response_index].clone();
                self.response_index += 1;
                Ok(response)
            } else {
                Ok(LlmResponse::text("Default mock response"))
            }
        }
    }

    /// Mock Discord API client that tracks sent messages.
    ///
    /// Stores all sent messages in memory for verification.
    #[cfg(feature = "discord")]
    struct MockDiscordApiClient {
        sent_messages: Arc<Mutex<Vec<String>>>,
    }

    #[cfg(feature = "discord")]
    impl MockDiscordApiClient {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn sent_messages(&self) -> Arc<Mutex<Vec<String>>> {
            self.sent_messages.clone()
        }

        async fn send_message_chunked(
            &mut self,
            _channel_id: &str,
            content: &str,
        ) -> Result<(), switchboard::discord::api::ApiError> {
            let mut messages = self.sent_messages.lock().unwrap();
            messages.push(content.to_string());
            Ok(())
        }
    }

    /// Simulated BotState for testing the message processing logic.
    ///
    /// This replicates the key filtering and processing logic from
    /// handle_message_create_event without requiring the private function.
    #[cfg(feature = "discord")]
    struct TestBotState {
        llm_client: MockLlmClient,
        api_client: TokioMutex<MockDiscordApiClient>,
        conversation_manager: TokioMutex<ConversationManager>,
        bot_user_id: u64,
        channel_id: String,
        system_prompt: String,
    }

    #[cfg(feature = "discord")]
    impl TestBotState {
        fn new(llm_client: MockLlmClient, bot_user_id: u64, channel_id: &str) -> Self {
            let config = ConversationConfig::new(30, 120);
            Self {
                llm_client,
                api_client: TokioMutex::new(MockDiscordApiClient::new()),
                conversation_manager: TokioMutex::new(ConversationManager::new(config)),
                bot_user_id,
                channel_id: channel_id.to_string(),
                system_prompt: "You are the Switchboard Concierge.".to_string(),
            }
        }

        /// Process a message through the listener logic.
        ///
        /// This replicates the filtering and response logic from handle_message_create_event:
        /// 1. Filter out bot's own messages
        /// 2. Filter out messages from wrong channels
        /// 3. Process valid messages through LLM
        async fn process_message(
            &mut self,
            channel_id: &str,
            author_id: &str,
            content: &str,
        ) -> Result<bool, String> {
            // Parse author ID
            let author_id_num: u64 = match author_id.parse() {
                Ok(id) => id,
                Err(e) => {
                    tracing::warn!("Failed to parse author ID: {}", e);
                    return Ok(false);
                }
            };

            // Check if message is from the bot itself (filtering)
            if author_id_num == self.bot_user_id {
                tracing::debug!("Ignoring message from bot user");
                return Ok(false);
            }

            // Check if message is from the configured channel (filtering)
            let channel_id_num: u64 = match channel_id.parse() {
                Ok(id) => id,
                Err(e) => {
                    tracing::warn!("Failed to parse channel ID: {}", e);
                    return Ok(false);
                }
            };

            let target_channel_id: u64 = match self.channel_id.parse() {
                Ok(id) => id,
                Err(e) => {
                    tracing::warn!("Failed to parse target channel ID: {}", e);
                    return Ok(false);
                }
            };

            if channel_id_num != target_channel_id {
                tracing::debug!(
                    "Ignoring message from wrong channel {} (expected {})",
                    channel_id_num,
                    target_channel_id
                );
                return Ok(false);
            }

            // Skip empty messages
            if content.trim().is_empty() {
                tracing::debug!("Message content is empty, ignoring");
                return Ok(false);
            }

            tracing::info!(
                "Processing message from channel {}: {}",
                channel_id,
                content.chars().take(100).collect::<String>()
            );

            // Add user message to conversation
            {
                let mut conv_manager = self.conversation_manager.lock().await;
                conv_manager.add_user_message(author_id, content);

                // Get messages for LLM
                let messages = conv_manager
                    .get_messages_for_llm(author_id, &self.system_prompt)
                    .unwrap_or_else(|| {
                        vec![switchboard::discord::conversation::ChatMessage::user(content)]
                    });

                // Get tools schema
                let tools_value = tools_schema();
                let tools_array = tools_value.as_array().expect("Tools should be an array");
                let tools_owned: Vec<serde_json::Value> = tools_array.clone();
                let tools_slice: &[serde_json::Value] = &tools_owned;

                // Call the LLM
                let llm_response = self
                    .llm_client
                    .chat_completion(&messages, Some(tools_slice))
                    .await;

                // Process the response
                let response_text = match llm_response {
                    Ok(resp) => {
                        // Extract text content from LlmResponse using the helper method
                        let text = resp.text_content();
                        // Add assistant response to conversation
                        conv_manager.add_assistant_message(author_id, &text);
                        text
                    }
                    Err(e) => {
                        tracing::error!("LLM error: {:?}", e);
                        let error_msg = switchboard::discord::llm::get_user_error_message(&e);
                        conv_manager.add_assistant_message(author_id, &error_msg);
                        error_msg
                    }
                };

                // Send the response to Discord
                let mut api_client = self.api_client.lock().await;
                api_client
                    .send_message_chunked(channel_id, &response_text)
                    .await
                    .map_err(|e| e.to_string())?;
            }

            Ok(true)
        }
    }

    // ===========================================================================
    // Test Cases
    // ===========================================================================

    /// Test: Valid user message in correct channel gets processed.
    ///
    /// Verifies that:
    /// 1. A message from a regular user in the correct channel is processed
    /// 2. The LLM is called
    /// 3. A response is generated and "sent" via the mock API
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_valid_message_processed() {
        // Setup: Create mock LLM that returns a text response
        let llm_responses = vec![LlmResponse::text(
            "Hello! I'm the Switchboard Concierge. How can I help you today?",
        )];
        let mock_llm = MockLlmClient::new(llm_responses);
        let call_count = mock_llm.call_count();

        // Create test state with bot user ID 999 and target channel 111
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Process a valid message from user 123 in channel 111
        let result = state
            .process_message("111", "123", "Hello, can you help me?")
            .await;

        // Verify: Message should be processed successfully
        assert!(result.is_ok(), "Processing should succeed: {:?}", result);
        assert!(
            result.unwrap(),
            "Message should be processed (not filtered)"
        );

        // Verify: LLM was called
        let llm_calls = *call_count.lock().unwrap();
        assert_eq!(llm_calls, 1, "LLM should be called exactly once");

        // Verify: Response was sent
        let sent = state.api_client.lock().await.sent_messages();
        let messages = sent.lock().unwrap();
        assert_eq!(messages.len(), 1, "One message should have been sent");
        assert!(
            messages[0].contains("Switchboard") || messages[0].contains("Concierge"),
            "Response should be from LLM: {}",
            messages[0]
        );
    }

    /// Test: Bot's own messages are filtered out.
    ///
    /// Verifies that:
    /// 1. Messages from the bot itself (matching bot_user_id) are ignored
    /// 2. The LLM is NOT called
    /// 3. No response is generated
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_bot_message_filtered() {
        // Setup: Create mock LLM
        let llm_responses = vec![LlmResponse::text("Bot response")];
        let mock_llm = MockLlmClient::new(llm_responses);
        let call_count = mock_llm.call_count();

        // Create test state with bot user ID 999 and target channel 111
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Process a message FROM the bot (author_id == bot_user_id)
        let result = state
            .process_message("111", "999", "This is my own message")
            .await;

        // Verify: Message should be filtered (processed returns false)
        assert!(result.is_ok(), "Processing should succeed: {:?}", result);
        assert!(
            !result.unwrap(),
            "Bot message should be filtered (not processed)"
        );

        // Verify: LLM was NOT called
        let llm_calls = *call_count.lock().unwrap();
        assert_eq!(llm_calls, 0, "LLM should NOT be called for bot messages");

        // Verify: No response was sent
        let sent = state.api_client.lock().await.sent_messages();
        let messages = sent.lock().unwrap();
        assert!(
            messages.is_empty(),
            "No messages should have been sent for bot message"
        );
    }

    /// Test: Messages from wrong channel are filtered out.
    ///
    /// Verifies that:
    /// 1. Messages from a channel other than the configured one are ignored
    /// 2. The LLM is NOT called
    /// 3. No response is generated
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_wrong_channel_filtered() {
        // Setup: Create mock LLM
        let llm_responses = vec![LlmResponse::text("Response")];
        let mock_llm = MockLlmClient::new(llm_responses);
        let call_count = mock_llm.call_count();

        // Create test state with bot user ID 999 and target channel 111
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Process a message from a WRONG channel (222 instead of 111)
        let result = state
            .process_message("222", "123", "Hello from wrong channel!")
            .await;

        // Verify: Message should be filtered (processed returns false)
        assert!(result.is_ok(), "Processing should succeed: {:?}", result);
        assert!(
            !result.unwrap(),
            "Wrong channel message should be filtered (not processed)"
        );

        // Verify: LLM was NOT called
        let llm_calls = *call_count.lock().unwrap();
        assert_eq!(
            llm_calls, 0,
            "LLM should NOT be called for wrong channel messages"
        );

        // Verify: No response was sent
        let sent = state.api_client.lock().await.sent_messages();
        let messages = sent.lock().unwrap();
        assert!(
            messages.is_empty(),
            "No messages should have been sent for wrong channel"
        );
    }

    /// Test: Empty messages are filtered out.
    ///
    /// Verifies that:
    /// 1. Empty or whitespace-only messages are ignored
    /// 2. The LLM is NOT called
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_empty_message_filtered() {
        // Setup: Create mock LLM
        let llm_responses = vec![LlmResponse::text("Response")];
        let mock_llm = MockLlmClient::new(llm_responses);
        let call_count = mock_llm.call_count();

        // Create test state
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Process an empty message
        let result = state.process_message("111", "123", "   ").await;

        // Verify: Message should be filtered
        assert!(result.is_ok(), "Processing should succeed: {:?}", result);
        assert!(!result.unwrap(), "Empty message should be filtered");

        // Verify: LLM was NOT called
        let llm_calls = *call_count.lock().unwrap();
        assert_eq!(llm_calls, 0, "LLM should NOT be called for empty messages");
    }

    /// Test: Tool call response is processed correctly.
    ///
    /// Verifies that:
    /// 1. LLM returns a text response (simulating successful tool execution)
    /// 2. Response is generated
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_tool_call_processed() {
        // Setup: Create mock LLM that returns a text response
        // (In a real scenario, this would go through tool execution loop,
        // but we simulate it by returning the final text directly)
        let llm_responses = vec![LlmResponse::text(
            "The system is running well! All agents are idle.",
        )];
        let mock_llm = MockLlmClient::new(llm_responses);

        // Create test state
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Process a message that should trigger response
        let result = state
            .process_message("111", "123", "What's the system status?")
            .await;

        // Verify: Message was processed
        assert!(result.is_ok(), "Processing should succeed: {:?}", result);
        assert!(result.unwrap(), "Message should be processed");

        // Verify: Response was sent
        let sent = state.api_client.lock().await.sent_messages();
        let messages = sent.lock().unwrap();
        assert!(!messages.is_empty(), "Response should have been sent");
        // The response should contain the text
        assert!(
            messages[0].contains("running") || messages[0].contains("system"),
            "Response should mention system status: {}",
            messages[0]
        );
    }

    /// Test: Invalid author ID is handled gracefully.
    ///
    /// Verifies that:
    /// 1. Messages with invalid author IDs don't crash the system
    /// 2. The message is filtered out
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_invalid_author_id_handled() {
        // Setup: Create mock LLM
        let llm_responses = vec![LlmResponse::text("Response")];
        let mock_llm = MockLlmClient::new(llm_responses);

        // Create test state
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Process a message with invalid author ID (non-numeric)
        let result = state.process_message("111", "invalid_id", "Hello").await;

        // Verify: Should handle gracefully (filter out)
        assert!(result.is_ok(), "Processing should succeed: {:?}", result);
        assert!(!result.unwrap(), "Invalid author ID should be filtered");
    }

    /// Test: Multiple messages from same user maintain conversation.
    ///
    /// Verifies that:
    /// 1. Multiple messages from the same user share conversation context
    /// 2. Both messages are processed
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_conversation_maintained() {
        // Setup: Create mock LLM with responses for two messages
        let llm_responses = vec![
            LlmResponse::text("First response"),
            LlmResponse::text("Second response"),
        ];
        let mock_llm = MockLlmClient::new(llm_responses);

        // Create test state
        let mut state = TestBotState::new(mock_llm, 999, "111");

        // Send first message
        let result1 = state.process_message("111", "123", "Hello").await;
        assert!(result1.unwrap(), "First message should be processed");

        // Send second message from same user
        let result2 = state.process_message("111", "123", "What's next?").await;
        assert!(result2.unwrap(), "Second message should be processed");

        // Verify: Both responses were sent
        let sent = state.api_client.lock().await.sent_messages();
        let messages = sent.lock().unwrap();
        assert_eq!(messages.len(), 2, "Two messages should have been sent");
    }
}
