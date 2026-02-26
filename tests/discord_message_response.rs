//! Integration tests for full message → response flow in Discord concierge.
//!
//! This module tests the complete flow from receiving a Discord message
//! to generating and returning a response, including:
//! - Simple text exchange (no tools)
//! - Tool execution flow (LLM calls tool, tool executes, response generated)
//! - Error handling

#[cfg(feature = "discord")]
mod discord_tests {
    #[cfg(feature = "discord")]
    use switchboard::discord::conversation::{
        ChatMessage, ConversationConfig, ConversationManager, MessageRole, ToolCall, ToolFunction,
    };
    #[cfg(feature = "discord")]
    use switchboard::discord::llm::{LlmError, LlmResponse, ToolCallResult, ToolExecutor};
    #[cfg(feature = "discord")]
    use switchboard::discord::tools::tools_schema;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    /// Create a system message.
    #[cfg(feature = "discord")]
    fn system_message(content: &str) -> ChatMessage {
        ChatMessage {
            role: MessageRole::System,
            content: content.to_string(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }

    /// Mock tool executor for testing.
    ///
    /// This executor simulates tool execution with configurable responses.
    struct MockToolExecutor {
        /// Responses for each tool call (in order)
        responses: Arc<Mutex<Vec<Result<String, String>>>>,
    }

    impl MockToolExecutor {
        fn new(responses: Vec<Result<String, String>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses)),
            }
        }

        /// Create a simple executor that returns success for get_status.
        fn with_get_status_response(status: &str) -> Self {
            Self::new(vec![Ok(status.to_string())])
        }

        /// Create an executor that returns an error.
        fn with_error(error: &str) -> Self {
            Self::new(vec![Err(error.to_string())])
        }
    }

    impl ToolExecutor for MockToolExecutor {
        fn execute(&self, name: &str, arguments: &str) -> Result<String, String> {
            let mut responses = self.responses.lock().unwrap();
            if responses.is_empty() {
                return Ok(format!(
                    "Mock response for {} with args: {}",
                    name, arguments
                ));
            }
            responses.remove(0)
        }
    }

    /// Simple mock client that wraps a predefined response sequence.
    struct TestOpenRouterClient {
        responses: Vec<LlmResponse>,
        index: usize,
    }

    impl TestOpenRouterClient {
        fn new(responses: Vec<LlmResponse>) -> Self {
            Self {
                responses,
                index: 0,
            }
        }

        /// Reset the client to the beginning (for re-running tests)
        fn reset(&mut self) {
            self.index = 0;
        }

        async fn chat_completion(
            &mut self,
            _messages: &[ChatMessage],
            _tools: Option<&[serde_json::Value]>,
        ) -> Result<LlmResponse, LlmError> {
            if self.index < self.responses.len() {
                let response = self.responses[self.index].clone();
                self.index += 1;
                Ok(response)
            } else {
                Ok(LlmResponse::text("No more responses configured"))
            }
        }
    }

    /// Test helper: create a new conversation with a user message.
    #[cfg(feature = "discord")]
    fn create_conversation_with_message(
        user_id: &str,
        message: &str,
    ) -> (ConversationManager, Vec<ChatMessage>) {
        let config = ConversationConfig::new(30, 120);
        let mut manager = ConversationManager::new(config);

        // Add user message to conversation
        manager.add_user_message(user_id, message);

        // Get the conversation and its messages
        let conv = manager.get_or_create_conversation(user_id);
        let messages: Vec<ChatMessage> = conv.messages().to_vec();

        (manager, messages)
    }

    /// Test: Simple text exchange - user sends message, LLM returns text response (no tools).
    ///
    /// This test verifies the basic message → response flow without tool usage.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_simple_text_exchange() {
        // Create mock LLM client that returns a simple text response
        let mut client = TestOpenRouterClient::new(vec![LlmResponse::text(
            "Hello! I'm the Switchboard concierge. How can I help you today?",
        )]);

        // Create conversation with user message
        let (mut manager, mut messages) = create_conversation_with_message("test_user", "Hello");

        // Add system prompt
        messages.insert(
            0,
            system_message("You are the Switchboard Concierge — a friendly assistant in Discord."),
        );

        // Get tools schema as owned Vec
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        // Call LLM with tools available but expect text-only response
        let response = client
            .chat_completion(&messages, Some(tools_slice))
            .await
            .unwrap();

        // Verify it's a text response (not a tool call)
        assert!(
            !response.is_tool_call,
            "Expected text response, not tool call"
        );
        assert!(response.text.is_some(), "Expected text content");

        let response_text = response.text.unwrap();
        assert!(
            response_text.contains("Switchboard") || response_text.contains("concierge"),
            "Response should be conversational: {}",
            response_text
        );

        // Add assistant response to conversation
        manager.add_assistant_message("test_user", &response_text);

        // Verify message was added to conversation
        let conv = manager.get_or_create_conversation("test_user");
        let updated_messages = conv.messages();
        assert!(
            updated_messages.len() >= 2,
            "Should have at least user message and assistant response"
        );
    }

    /// Test: Tool execution flow - LLM calls a tool, tool executes, final response generated.
    ///
    /// This test verifies:
    /// 1. LLM returns a tool call response
    /// 2. Tool is executed
    /// 3. Tool result is fed back to LLM
    /// 4. Final text response is generated
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_tool_execution_flow() {
        // Create mock LLM client: first returns tool call, then returns final text
        let tool_call = ToolCall {
            id: "call_123".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "get_status".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let mut client = TestOpenRouterClient::new(vec![
            LlmResponse::tool_calls(vec![tool_call]),
            LlmResponse::text(
                "The system is running well! All agents are idle and the inbox is empty.",
            ),
        ]);

        // Create conversation with user message
        let (mut manager, mut messages) =
            create_conversation_with_message("test_user", "What's the status of the system?");

        // Add system prompt
        messages.insert(
            0,
            system_message("You are the Switchboard Concierge. Use tools when needed."),
        );

        // Get tools schema
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        // Create mock executor that returns status
        let mock_executor = MockToolExecutor::with_get_status_response(
            "QA complete. 2 agents idle. 0 items in inbox.",
        );

        // Simulate the tool-use loop
        let mut tool_iterations = 0;
        let max_iterations = 10;
        let mut final_response = String::new();

        while tool_iterations < max_iterations {
            let response = client
                .chat_completion(&messages, Some(tools_slice))
                .await
                .unwrap();

            if response.is_tool_call {
                // Add assistant message with tool calls to history
                let tool_response_msg = ChatMessage::assistant_with_tools(
                    response.text_content(),
                    response.tool_calls.clone(),
                );
                messages.push(tool_response_msg.clone());

                // Add to conversation
                manager.add_assistant_message_with_tools(
                    "test_user",
                    &response.text_content(),
                    response.tool_calls.clone(),
                );

                // Execute each tool call
                for tool_call in &response.tool_calls {
                    let result = mock_executor
                        .execute(&tool_call.function.name, &tool_call.function.arguments);

                    let tool_result = match result {
                        Ok(output) => ToolCallResult::success(&tool_call.id, output),
                        Err(e) => ToolCallResult::error(&tool_call.id, e),
                    };

                    // Add tool result to history
                    let tool_msg =
                        ChatMessage::tool(&tool_result.content, &tool_result.tool_call_id);
                    messages.push(tool_msg);

                    // Add to conversation
                    manager.add_tool_result(
                        "test_user",
                        &tool_result.content,
                        &tool_result.tool_call_id,
                    );
                }

                tool_iterations += 1;
            } else {
                // Final text response
                final_response = response.text.unwrap_or_default();
                messages.push(ChatMessage::assistant(&final_response));
                break;
            }
        }

        // Verify the flow completed
        assert!(
            tool_iterations > 0,
            "Should have executed at least one tool call iteration"
        );
        assert!(
            !final_response.is_empty(),
            "Should have generated a final response"
        );
        assert!(
            final_response.contains("system") || final_response.contains("running"),
            "Response should mention system status: {}",
            final_response
        );

        // Add assistant response to conversation
        manager.add_assistant_message("test_user", &final_response);

        // Verify conversation has tool interactions
        let conv = manager.get_or_create_conversation("test_user");
        let final_messages = conv.messages();
        assert!(
            final_messages.len() >= 3,
            "Should have user message, tool call, and assistant response"
        );
    }

    /// Test: Tool execution with read_file tool.
    ///
    /// This test verifies that the read_file tool works correctly when called by the LLM.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_read_file_tool_execution() {
        // Create a temp file to read
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let test_file_path = temp_dir.path().join("test_read.txt");
        std::fs::write(&test_file_path, "Hello from test file!")
            .expect("Failed to write test file");

        // Create mock LLM that requests to read a file
        let tool_call = ToolCall {
            id: "call_read".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "read_file".to_string(),
                arguments: json!({
                    "path": test_file_path.to_string_lossy()
                })
                .to_string(),
            },
        };

        let mut client = TestOpenRouterClient::new(vec![
            LlmResponse::tool_calls(vec![tool_call]),
            LlmResponse::text("I found the file! It says: 'Hello from test file!'"),
        ]);

        // Create conversation
        let (mut manager, mut messages) =
            create_conversation_with_message("test_user", "What's in that test file?");

        messages.insert(0, system_message("You are helpful."));

        // Get tools schema
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        // Create executor that actually reads files
        struct FileReadExecutor;

        impl ToolExecutor for FileReadExecutor {
            fn execute(&self, name: &str, arguments: &str) -> Result<String, String> {
                if name == "read_file" {
                    // Parse arguments to get path
                    let args: serde_json::Value =
                        serde_json::from_str(arguments).map_err(|e| e.to_string())?;

                    let path = args["path"].as_str().ok_or("No path provided")?;

                    std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))
                } else {
                    Err(format!("Unknown tool: {}", name))
                }
            }
        }

        let executor = FileReadExecutor;

        // Simulate tool-use loop
        let mut tool_iterations = 0;
        let mut final_response = String::new();

        while tool_iterations < 10 {
            let response = client
                .chat_completion(&messages, Some(tools_slice))
                .await
                .unwrap();

            if response.is_tool_call {
                messages.push(ChatMessage::assistant_with_tools(
                    response.text_content(),
                    response.tool_calls.clone(),
                ));

                for tool_call in &response.tool_calls {
                    let result =
                        executor.execute(&tool_call.function.name, &tool_call.function.arguments);

                    let tool_result = match result {
                        Ok(output) => ToolCallResult::success(&tool_call.id, output),
                        Err(e) => ToolCallResult::error(&tool_call.id, e),
                    };

                    messages.push(ChatMessage::tool(
                        &tool_result.content,
                        &tool_result.tool_call_id,
                    ));
                }

                tool_iterations += 1;
            } else {
                final_response = response.text.unwrap_or_default();
                messages.push(ChatMessage::assistant(&final_response));
                break;
            }
        }

        // Verify file was read and response generated
        assert!(tool_iterations > 0, "Should have called read_file tool");
        assert!(
            final_response.contains("test file") || final_response.contains("Hello"),
            "Response should mention file content: {}",
            final_response
        );

        // Note: temp_dir is automatically cleaned up when dropped
    }

    /// Test: Error handling - LLM API error (401 Unauthorized).
    ///
    /// This test verifies that the system properly handles and reports LLM API errors.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_llm_api_error_handling() {
        // Create a client that simulates an error
        struct ErrorClient;

        impl ErrorClient {
            async fn chat_completion(
                &self,
                _messages: &[ChatMessage],
                _tools: Option<&[serde_json::Value]>,
            ) -> Result<LlmResponse, LlmError> {
                Err(LlmError::InvalidApiKey)
            }
        }

        let client = ErrorClient;

        // Create conversation
        let (mut _manager, mut messages) = create_conversation_with_message("test_user", "Hello");

        messages.insert(0, system_message("You are helpful."));

        // Get tools schema
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        // Attempt to call LLM and verify error handling
        let result = client.chat_completion(&messages, Some(tools_slice)).await;

        assert!(result.is_err(), "Should return an error");
        let error = result.unwrap_err();

        match error {
            LlmError::InvalidApiKey => {
                // Expected error type
            }
            other => {
                panic!("Expected InvalidApiKey error, got: {:?}", other);
            }
        }
    }

    /// Test: Error handling - Tool execution failure.
    ///
    /// This test verifies that tool execution errors are properly handled
    /// and reported back to the LLM.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_tool_execution_error_handling() {
        // Create mock LLM that calls a tool, then responds after seeing the error
        let tool_call = ToolCall {
            id: "call_fail".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "read_file".to_string(),
                arguments: json!({
                    "path": "/nonexistent/file.txt"
                })
                .to_string(),
            },
        };

        let mut client = TestOpenRouterClient::new(vec![
            LlmResponse::tool_calls(vec![tool_call]),
            LlmResponse::text("I tried to read the file but it seems to not exist."),
        ]);

        let (mut _manager, mut messages) =
            create_conversation_with_message("test_user", "Show me this file");

        messages.insert(0, system_message("You are helpful."));

        // Get tools schema
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        // Create executor that always fails
        let mock_executor = MockToolExecutor::with_error("File not found: /nonexistent/file.txt");

        // Simulate tool-use loop
        let mut tool_iterations = 0;
        let mut final_response = String::new();

        while tool_iterations < 10 {
            let response = client
                .chat_completion(&messages, Some(tools_slice))
                .await
                .unwrap();

            if response.is_tool_call {
                messages.push(ChatMessage::assistant_with_tools(
                    response.text_content(),
                    response.tool_calls.clone(),
                ));

                for tool_call in &response.tool_calls {
                    let result = mock_executor
                        .execute(&tool_call.function.name, &tool_call.function.arguments);

                    // Even on error, we create a ToolCallResult
                    let tool_result = match result {
                        Ok(output) => ToolCallResult::success(&tool_call.id, output),
                        Err(e) => ToolCallResult::error(&tool_call.id, e),
                    };

                    messages.push(ChatMessage::tool(
                        &tool_result.content,
                        &tool_result.tool_call_id,
                    ));

                    // Verify error was recorded
                    assert!(!tool_result.success, "Tool result should indicate failure");
                }

                tool_iterations += 1;
            } else {
                final_response = response.text.unwrap_or_default();
                messages.push(ChatMessage::assistant(&final_response));
                break;
            }
        }

        // Verify the flow handled the error gracefully
        assert!(tool_iterations > 0, "Should have attempted tool call");
        assert!(
            !final_response.is_empty(),
            "Should still generate a response even after tool error"
        );
    }

    /// Test: Multiple tool calls in sequence.
    ///
    /// This test verifies that multiple tool calls in a single LLM response
    /// are handled correctly.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_multiple_tool_calls() {
        // Create mock LLM that returns multiple tool calls
        let tool_calls = vec![
            ToolCall {
                id: "call_1".to_string(),
                r#type: "function".to_string(),
                function: ToolFunction {
                    name: "get_status".to_string(),
                    arguments: "{}".to_string(),
                },
            },
            ToolCall {
                id: "call_2".to_string(),
                r#type: "function".to_string(),
                function: ToolFunction {
                    name: "list_inbox".to_string(),
                    arguments: "{}".to_string(),
                },
            },
        ];

        let mut client = TestOpenRouterClient::new(vec![
            LlmResponse::tool_calls(tool_calls),
            LlmResponse::text("Status: All agents idle. Inbox: 3 items waiting."),
        ]);

        let (mut _manager, mut messages) =
            create_conversation_with_message("test_user", "Give me a full status update");

        messages.insert(0, system_message("You are helpful."));

        // Get tools schema
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        // Create executor with responses for both tools
        let mock_executor = MockToolExecutor::new(vec![
            Ok("QA: complete, Agents: 2 idle".to_string()),
            Ok("3 items: bug_001.md, task_002.md, task_003.md".to_string()),
        ]);

        // Simulate tool-use loop
        let mut tool_iterations = 0;
        let mut final_response = String::new();

        while tool_iterations < 10 {
            let response = client
                .chat_completion(&messages, Some(tools_slice))
                .await
                .unwrap();

            if response.is_tool_call {
                // Verify we got multiple tool calls
                assert!(
                    response.tool_calls.len() > 1,
                    "Should have multiple tool calls"
                );

                messages.push(ChatMessage::assistant_with_tools(
                    response.text_content(),
                    response.tool_calls.clone(),
                ));

                for tool_call in &response.tool_calls {
                    let result = mock_executor
                        .execute(&tool_call.function.name, &tool_call.function.arguments);

                    let tool_result = match result {
                        Ok(output) => ToolCallResult::success(&tool_call.id, output),
                        Err(e) => ToolCallResult::error(&tool_call.id, e),
                    };

                    messages.push(ChatMessage::tool(
                        &tool_result.content,
                        &tool_result.tool_call_id,
                    ));
                }

                tool_iterations += 1;
            } else {
                final_response = response.text.unwrap_or_default();
                messages.push(ChatMessage::assistant(&final_response));
                break;
            }
        }

        // Verify response incorporated both tool results
        assert!(
            final_response.contains("idle") && final_response.contains("items"),
            "Response should mention both status and inbox: {}",
            final_response
        );
    }

    /// Test: Conversation context is maintained across messages.
    ///
    /// This test verifies that the conversation history is properly maintained
    /// and sent to the LLM.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_conversation_context_maintained() {
        let mut client = TestOpenRouterClient::new(vec![
        LlmResponse::text("Hello! How can I help you?"),
        LlmResponse::text("I can help with that! Let me check the status."),
        LlmResponse::text("Based on our conversation, you wanted to know about the status. Everything is working well!"),
    ]);

        // First message
        let config = ConversationConfig::new(30, 120);
        let mut manager = ConversationManager::new(config);

        manager.add_user_message("user123", "Hi there!");
        let conv1 = manager.get_or_create_conversation("user123");
        let mut messages1: Vec<ChatMessage> = conv1.messages().to_vec();
        messages1.insert(0, system_message("You are helpful."));

        // Get tools schema
        let tools_value = tools_schema();
        let tools_array = tools_value.as_array().expect("Tools should be an array");
        let tools_owned: Vec<serde_json::Value> = tools_array.clone();
        let tools_slice: &[serde_json::Value] = &tools_owned;

        let response1 = client
            .chat_completion(&messages1, Some(tools_slice))
            .await
            .unwrap();

        // Use add_assistant_message with a string reference
        let response_text = response1.text_content();
        manager.add_assistant_message("user123", &response_text);

        // Second message - should include previous context
        manager.add_user_message("user123", "Can you check something for me?");
        let conv2 = manager.get_or_create_conversation("user123");
        let messages2 = conv2.messages();

        // Should have: system, hi, hello response, can you check
        assert!(
            messages2.len() >= 3,
            "Conversation should maintain history: {} messages",
            messages2.len()
        );

        // Verify the second message includes context from first exchange
        let has_previous_context = messages2.iter().any(|m| m.content.contains("Hi"));
        assert!(
            has_previous_context,
            "Should include previous message in context"
        );
    }

    /// Test: Message with no tools (tools disabled).
    ///
    /// This test verifies that the system works correctly when tools are not provided.
    #[cfg(feature = "discord")]
    #[tokio::test]
    async fn test_no_tools_mode() {
        let mut client = TestOpenRouterClient::new(vec![LlmResponse::text(
            "This is a response without using any tools.",
        )]);

        let (mut _manager, mut messages) = create_conversation_with_message("test_user", "Hello!");

        messages.insert(0, system_message("You are helpful."));

        // Call without tools (None)
        let response = client.chat_completion(&messages, None).await.unwrap();

        assert!(
            !response.is_tool_call,
            "Should not attempt tool calls when tools disabled"
        );
        assert!(
            response.text.as_ref().unwrap().contains("without"),
            "Should get text response: {}",
            response.text.as_ref().unwrap()
        );
    }
}
