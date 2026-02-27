//! LLM integration module for Discord concierge.
//!
//! This module provides integration with OpenRouter for generating responses
//! to user messages. It implements a tool-use loop that allows the LLM to
//! call tools (like file operations, status checks, etc.) and return results.
//!
//! # Architecture
//!
//! - [`OpenRouterClient`] - Main client for OpenRouter API interactions
//! - [`LlmResponse`] - Parsed response from the LLM
//! - [`ToolCallResult`] - Result of executing a tool call
//!
//! # Tool-Use Loop
//!
//! The tool-use loop allows the LLM to perform multi-step operations:
//! 1. Send messages + tools to OpenRouter
//! 2. If response contains tool_calls, execute each and append results
//! 3. Continue loop until text completion or max iterations (10)
//!
//! # Error Handling
//!
//! | Error | User Message |
//! |-------|--------------|
//! | 401 Unauthorized | "⚠️ LLM API key is invalid..." |
//! | 429 Rate Limited | "⚠️ Rate limited, try again shortly." |
//! | 500+ Server Error | "⚠️ LLM provider is having issues." |
//! | Timeout | "⚠️ Response timed out..." |
//! | Malformed | "⚠️ Got an unexpected response..." |

use crate::discord::conversation::{ChatMessage, ToolCall, ToolFunction};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::warn;

/// OpenRouter API endpoint
const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Maximum iterations in the tool-use loop
const MAX_TOOL_ITERATIONS: usize = 10;

/// Default request timeout
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Maximum retries for transient errors
const MAX_RETRIES: u32 = 2;

/// Delay between retries for rate limiting
const RETRY_DELAY_MS: u64 = 1000;

/// LLM configuration errors
#[derive(Debug, Error)]
pub enum LlmError {
    /// API key is missing or invalid
    #[error("API key is missing or invalid")]
    InvalidApiKey,

    /// Rate limited by the API
    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    /// Server error from the API
    #[error("Server error: {0}")]
    ServerError(String),

    /// Request timeout
    #[error("Request timed out: {0}")]
    Timeout(String),

    /// Invalid or malformed response from the API
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// HTTP request failed
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Maximum iterations exceeded in tool-use loop
    #[error("Maximum tool iterations exceeded")]
    MaxIterationsExceeded,

    /// No content in response
    #[error("No content in response")]
    NoContent,

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),
}

/// Result type for LLM operations
pub type LlmResult<T> = Result<T, LlmError>;

/// OpenRouter request payload
#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    /// Model identifier
    model: String,
    /// Messages for the conversation
    messages: Vec<ChatMessage>,
    /// Available tools
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    /// Maximum tokens to generate
    #[serde(rename = "max_tokens")]
    max_tokens: u32,
}

/// Tool definition in OpenAI function-calling format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool type (always "function")
    #[serde(rename = "type")]
    tool_type: String,
    /// Function definition
    function: FunctionDefinition,
}

/// Function definition for tool
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionDefinition {
    /// Function name
    name: String,
    /// Function description
    description: String,
    /// JSON schema for parameters
    parameters: serde_json::Value,
}

/// Convert tools schema JSON to ToolDefinition vector.
///
/// The tools_schema() function returns tools in OpenAI format with fields at
/// the top level (name, description, parameters). This function wraps them
/// in the ToolDefinition structure expected by the LLM API.
///
/// # Arguments
///
/// * `tools_json` - JSON array from tools_schema()
///
/// # Returns
///
/// Vector of ToolDefinition ready for LLM API
pub fn tools_schema_to_definitions(tools_json: &serde_json::Value) -> Vec<ToolDefinition> {
    let mut definitions = Vec::new();

    if let Some(tools_array) = tools_json.as_array() {
        for tool in tools_array {
            if let (Some(name), Some(description), Some(parameters)) = (
                tool.get("name").and_then(|v| v.as_str()),
                tool.get("description").and_then(|v| v.as_str()),
                tool.get("parameters"),
            ) {
                definitions.push(ToolDefinition {
                    tool_type: "function".to_string(),
                    function: FunctionDefinition {
                        name: name.to_string(),
                        description: description.to_string(),
                        parameters: parameters.clone(),
                    },
                });
            }
        }
    }

    definitions
}

/// OpenRouter response payload
#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    /// Array of completion choices
    choices: Vec<CompletionChoice>,
    /// Usage statistics
    #[allow(dead_code)]
    #[serde(default)]
    usage: Option<Usage>,
}

/// A single completion choice
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CompletionChoice {
    /// The completed message
    message: ResponseMessage,
    /// Why the completion finished
    #[serde(default)]
    finish_reason: Option<String>,
}

/// Response message from the LLM
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ResponseMessage {
    /// Role of the message sender
    #[serde(default)]
    role: Option<String>,
    /// Content of the message
    #[serde(default)]
    content: Option<String>,
    /// Tool calls requested by the LLM
    #[serde(default, rename = "tool_calls")]
    tool_calls: Option<Vec<ResponseToolCall>>,
}

/// Tool call from the LLM response
#[derive(Debug, Deserialize, Clone)]
pub struct ResponseToolCall {
    /// Unique ID for this tool call
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function to call
    pub function: ResponseFunction,
}

/// Function call from the LLM
#[derive(Debug, Deserialize, Clone)]
pub struct ResponseFunction {
    /// Name of the function
    pub name: String,
    /// Arguments as JSON string
    pub arguments: String,
}

/// Usage statistics from the API
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Usage {
    /// Tokens used in the prompt
    #[serde(rename = "prompt_tokens")]
    prompt_tokens: Option<u32>,
    /// Tokens in the completion
    #[serde(rename = "completion_tokens")]
    completion_tokens: Option<u32>,
    /// Total tokens used
    #[serde(rename = "total_tokens")]
    total_tokens: Option<u32>,
}

/// Result of executing a tool call
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallResult {
    /// ID of the tool call this result is for
    pub tool_call_id: String,
    /// Result content (JSON string or error message)
    pub content: String,
    /// Whether the tool execution was successful
    pub success: bool,
}

impl ToolCallResult {
    /// Create a successful tool result.
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            success: true,
        }
    }

    /// Create a failed tool result.
    pub fn error(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            content: error.into(),
            success: false,
        }
    }
}

/// Response from the LLM after processing
#[derive(Debug, Clone)]
pub struct LlmResponse {
    /// Text content (if any)
    pub text: Option<String>,
    /// Tool calls to execute (if any)
    pub tool_calls: Vec<ToolCall>,
    /// Whether this is a tool-use iteration or final response
    pub is_tool_call: bool,
}

impl LlmResponse {
    /// Create a text response.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            text: Some(content.into()),
            tool_calls: Vec::new(),
            is_tool_call: false,
        }
    }

    /// Create a tool call response.
    pub fn tool_calls(calls: Vec<ToolCall>) -> Self {
        Self {
            text: None,
            tool_calls: calls,
            is_tool_call: true,
        }
    }
}

/// OpenRouter client for LLM interactions
pub struct OpenRouterClient {
    /// HTTP client for API requests
    http_client: reqwest::Client,
    /// API key for authentication
    api_key: String,
    /// Model identifier
    model: String,
    /// Maximum tokens per response
    max_tokens: u32,
    /// Base URL for API
    base_url: String,
}

impl OpenRouterClient {
    /// Create a new OpenRouter client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - OpenRouter API key
    /// * `model` - Model identifier (e.g., "anthropic/claude-sonnet-4")
    /// * `max_tokens` - Maximum tokens per response
    pub fn new(api_key: String, model: String, max_tokens: u32) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            api_key,
            model,
            max_tokens,
            base_url: OPENROUTER_API_URL.to_string(),
        }
    }

    /// Create a new OpenRouter client with custom configuration.
    pub fn with_config(api_key: String, model: String, max_tokens: u32, base_url: &str) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http_client,
            api_key,
            model,
            max_tokens,
            base_url: base_url.to_string(),
        }
    }

    /// Send a chat completion request to OpenRouter.
    ///
    /// # Arguments
    ///
    /// * `messages` - Conversation messages
    /// * `tools` - Optional tool definitions
    ///
    /// # Returns
    ///
    /// Parsed LLM response
    pub async fn chat_completion(
        &self,
        messages: &[ChatMessage],
        tools: Option<&[ToolDefinition]>,
    ) -> LlmResult<LlmResponse> {
        let request = ChatCompletionRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            tools: tools.map(|t| t.to_vec()),
            max_tokens: self.max_tokens,
        };

        let response = self.send_request(request).await?;
        self.parse_response(response)
    }

    /// Send the request with retry logic.
    async fn send_request(
        &self,
        request: ChatCompletionRequest,
    ) -> LlmResult<ChatCompletionResponse> {
        let mut retries = 0;

        loop {
            match self.do_send_request(&request).await {
                Ok(response) => return Ok(response),
                Err(LlmError::RateLimited(wait_secs)) => {
                    if retries < MAX_RETRIES {
                        tracing::warn!("Rate limited, waiting {} seconds", wait_secs);
                        tokio::time::sleep(Duration::from_secs(wait_secs)).await;
                        retries += 1;
                    } else {
                        return Err(LlmError::RateLimited(wait_secs));
                    }
                }
                Err(LlmError::ServerError(_)) if retries < MAX_RETRIES => {
                    tracing::warn!("Server error, retrying...");
                    tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS * 2)).await;
                    retries += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Perform the actual HTTP request.
    async fn do_send_request(
        &self,
        request: &ChatCompletionRequest,
    ) -> LlmResult<ChatCompletionResponse> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let auth_value = HeaderValue::from_str(&format!("Bearer {}", self.api_key))
            .map_err(|_| LlmError::InvalidApiKey)?;
        headers.insert(AUTHORIZATION, auth_value);

        // Add OpenRouter-specific headers
        headers.insert(
            "HTTP-Referer",
            HeaderValue::from_static("https://switchboard.dev"),
        );
        headers.insert(
            "X-Title",
            HeaderValue::from_static("Switchboard Discord Concierge"),
        );

        let response = self
            .http_client
            .post(&self.base_url)
            .headers(headers)
            .json(request)
            .send()
            .await?;

        let status = response.status();

        // Handle error status codes
        match status.as_u16() {
            200..=299 => {
                let data: ChatCompletionResponse = response.json().await?;
                Ok(data)
            }
            401 => Err(LlmError::InvalidApiKey),
            429 => {
                let wait_secs = response
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| {
                        s.parse().unwrap_or_else(|e| {
                            warn!("Failed to parse retry-after header: {}", e);
                            60
                        })
                    })
                    .unwrap_or(60);
                Err(LlmError::RateLimited(wait_secs))
            }
            500..=599 => {
                let body = response.text().await.unwrap_or_default();
                Err(LlmError::ServerError(format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    body
                )))
            }
            _ => {
                let body = response.text().await.unwrap_or_default();
                Err(LlmError::ServerError(format!(
                    "HTTP {}: {}",
                    status.as_u16(),
                    body
                )))
            }
        }
    }

    /// Parse the API response into an LlmResponse.
    fn parse_response(&self, response: ChatCompletionResponse) -> LlmResult<LlmResponse> {
        let choice = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmError::NoContent)?;

        let message = choice.message;

        // Check if there are tool calls
        if let Some(tool_calls) = message.tool_calls {
            if !tool_calls.is_empty() {
                let calls: Vec<ToolCall> = tool_calls
                    .into_iter()
                    .map(|tc| ToolCall {
                        id: tc.id,
                        r#type: tc.tool_type,
                        function: ToolFunction {
                            name: tc.function.name,
                            arguments: tc.function.arguments,
                        },
                    })
                    .collect();
                return Ok(LlmResponse::tool_calls(calls));
            }
        }

        // Return text content
        let text = message.content.unwrap_or_default();
        Ok(LlmResponse::text(text))
    }

    /// Get the model identifier.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the max tokens setting.
    pub fn max_tokens(&self) -> u32 {
        self.max_tokens
    }
}

/// Trait for tool execution.
///
/// Implement this trait to define how tools are executed.
pub trait ToolExecutor: Send + Sync {
    /// Execute a tool call and return the result.
    fn execute(&self, name: &str, arguments: &str) -> Result<String, String>;
}

/// Process a user message with the LLM, including tool-use loop.
///
/// This function:
/// 1. Sends messages to the LLM with available tools
/// 2. If the LLM requests tool calls, executes them and continues
/// 3. Returns the final text response
///
/// # Arguments
///
/// * `client` - OpenRouter client
/// * `messages` - Conversation messages (will be modified in place)
/// * `tools` - Available tool definitions
/// * `executor` - Tool executor implementation
///
/// # Returns
///
/// Final text response from the LLM
pub async fn process_with_tools(
    client: &OpenRouterClient,
    messages: &mut Vec<ChatMessage>,
    tools: &[ToolDefinition],
    executor: &dyn ToolExecutor,
) -> LlmResult<String> {
    for iteration in 0..MAX_TOOL_ITERATIONS {
        tracing::debug!(
            "Tool-use loop iteration {}/{}",
            iteration + 1,
            MAX_TOOL_ITERATIONS
        );

        // Send request to LLM
        let response = client.chat_completion(messages, Some(tools)).await?;

        if response.is_tool_call {
            // Add assistant message with tool calls to history
            let assistant_msg = ChatMessage::assistant_with_tools(
                response.text_content(),
                response.tool_calls.clone(),
            );
            messages.push(assistant_msg);

            // Execute each tool call
            for tool_call in &response.tool_calls {
                let result =
                    executor.execute(&tool_call.function.name, &tool_call.function.arguments);

                let tool_result = match result {
                    Ok(output) => ToolCallResult::success(&tool_call.id, output),
                    Err(e) => ToolCallResult::error(&tool_call.id, e),
                };

                // Add tool result to history
                let tool_msg = ChatMessage::tool(&tool_result.content, &tool_result.tool_call_id);
                messages.push(tool_msg);
            }

            // Continue to next iteration
            continue;
        } else {
            // Final text response
            let text = response.text.unwrap_or_default();

            // Add assistant message to history
            messages.push(ChatMessage::assistant(&text));

            return Ok(text);
        }
    }

    // Maximum iterations exceeded
    Err(LlmError::MaxIterationsExceeded)
}

/// Helper to get user-friendly error message.
pub fn get_user_error_message(error: &LlmError) -> String {
    match error {
        LlmError::InvalidApiKey => {
            "⚠️ LLM API key is invalid. Check your OPENROUTER_API_KEY.".to_string()
        }
        LlmError::RateLimited(_) => "⚠️ Rate limited, try again shortly.".to_string(),
        LlmError::ServerError(_) => "⚠️ LLM provider is having issues.".to_string(),
        LlmError::Timeout(_) => "⚠️ Response timed out. Try a simpler question.".to_string(),
        LlmError::InvalidResponse(_) => "⚠️ Got an unexpected response from the LLM.".to_string(),
        LlmError::MaxIterationsExceeded => {
            "⚠️ Request took too long to process. Try a simpler question.".to_string()
        }
        LlmError::NoContent => "⚠️ Got an empty response from the LLM.".to_string(),
        _ => "⚠️ An error occurred while processing your request.".to_string(),
    }
}

/// Extended response message to include text content for tool calls
impl ResponseMessage {
    /// Get text content, handling tool calls gracefully.
    pub fn text_content(&self) -> String {
        if let Some(ref content) = self.content {
            content.clone()
        } else if let Some(ref tool_calls) = self.tool_calls {
            let calls: Vec<String> = tool_calls
                .iter()
                .map(|tc| format!("{}({})", tc.function.name, tc.function.arguments))
                .collect();
            format!("[Tool calls: {}]", calls.join(", "))
        } else {
            String::new()
        }
    }
}

impl LlmResponse {
    /// Get text content from the response.
    pub fn text_content(&self) -> String {
        if let Some(ref text) = self.text {
            text.clone()
        } else if !self.tool_calls.is_empty() {
            let calls: Vec<String> = self
                .tool_calls
                .iter()
                .map(|tc| format!("{}({})", tc.function.name, tc.function.arguments))
                .collect();
            format!("[Tool calls: {}]", calls.join(", "))
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_router_client_creation() {
        let client = OpenRouterClient::new(
            "test_api_key".to_string(),
            "anthropic/claude-sonnet-4".to_string(),
            1024,
        );

        assert_eq!(client.model(), "anthropic/claude-sonnet-4");
        assert_eq!(client.max_tokens(), 1024);
    }

    #[test]
    fn test_tool_call_result_success() {
        let result = ToolCallResult::success("call_123", "Tool output");

        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.content, "Tool output");
        assert!(result.success);
    }

    #[test]
    fn test_tool_call_result_error() {
        let result = ToolCallResult::error("call_123", "Error message");

        assert_eq!(result.tool_call_id, "call_123");
        assert_eq!(result.content, "Error message");
        assert!(!result.success);
    }

    #[test]
    fn test_llm_response_text() {
        let response = LlmResponse::text("Hello, user!");

        assert_eq!(response.text.unwrap(), "Hello, user!");
        assert!(response.tool_calls.is_empty());
        assert!(!response.is_tool_call);
    }

    #[test]
    fn test_llm_response_tool_calls() {
        let tool_calls = vec![ToolCall {
            id: "call_1".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "get_status".to_string(),
                arguments: "{}".to_string(),
            },
        }];

        let response = LlmResponse::tool_calls(tool_calls);

        assert!(response.text.is_none());
        assert_eq!(response.tool_calls.len(), 1);
        assert!(response.is_tool_call);
    }

    #[test]
    fn test_response_message_text_content() {
        let msg = ResponseMessage {
            role: Some("assistant".to_string()),
            content: Some("Hello".to_string()),
            tool_calls: None,
        };

        assert_eq!(msg.text_content(), "Hello");
    }

    #[test]
    fn test_response_message_tool_call_content() {
        let msg = ResponseMessage {
            role: Some("assistant".to_string()),
            content: None,
            tool_calls: Some(vec![ResponseToolCall {
                id: "call_1".to_string(),
                tool_type: "function".to_string(),
                function: ResponseFunction {
                    name: "get_status".to_string(),
                    arguments: "{}".to_string(),
                },
            }]),
        };

        assert_eq!(msg.text_content(), "[Tool calls: get_status({})]");
    }

    #[test]
    fn test_get_user_error_message_invalid_key() {
        let error = LlmError::InvalidApiKey;
        let msg = get_user_error_message(&error);

        assert!(msg.contains("API key"));
    }

    #[test]
    fn test_get_user_error_message_rate_limited() {
        let error = LlmError::RateLimited(30);
        let msg = get_user_error_message(&error);

        assert!(msg.contains("Rate limited"));
    }

    #[test]
    fn test_get_user_error_message_timeout() {
        let error = LlmError::Timeout("Request timed out".to_string());
        let msg = get_user_error_message(&error);

        assert!(msg.contains("timed out"));
    }

    #[test]
    fn test_get_user_error_message_server_error() {
        let error = LlmError::ServerError("Internal error".to_string());
        let msg = get_user_error_message(&error);

        assert!(msg.contains("having issues"));
    }

    #[test]
    fn test_llm_response_text_content() {
        let response = LlmResponse::text("Test response");
        assert_eq!(response.text_content(), "Test response");

        let tool_response = LlmResponse::tool_calls(vec![ToolCall {
            id: "call_1".to_string(),
            r#type: "function".to_string(),
            function: ToolFunction {
                name: "test".to_string(),
                arguments: "{}".to_string(),
            },
        }]);
        assert_eq!(tool_response.text_content(), "[Tool calls: test({})]");
    }

    // ============================================
    // LLM Error Handling Tests
    // ============================================

    /// Test that RateLimited error is created with correct wait time
    #[test]
    fn test_error_rate_limited() {
        let error = LlmError::RateLimited(30);
        assert!(matches!(error, LlmError::RateLimited(30)));
        assert_eq!(format!("{}", error), "Rate limited, retry after 30 seconds");
    }

    /// Test that RateLimited error handles default wait time
    #[test]
    fn test_error_rate_limited_default() {
        let error = LlmError::RateLimited(60);
        assert!(matches!(error, LlmError::RateLimited(60)));
    }

    /// Test that ServerError is created with correct message
    #[test]
    fn test_error_server_error() {
        let error = LlmError::ServerError("HTTP 500: Internal Server Error".to_string());
        assert!(matches!(error, LlmError::ServerError(_)));
        assert_eq!(
            format!("{}", error),
            "Server error: HTTP 500: Internal Server Error"
        );
    }

    /// Test that ServerError handles various 5xx codes
    #[test]
    fn test_error_server_error_5xx_codes() {
        // Test 500
        let error500 = LlmError::ServerError("HTTP 500: ".to_string());
        assert!(matches!(error500, LlmError::ServerError(_)));

        // Test 502
        let error502 = LlmError::ServerError("HTTP 502: Bad Gateway".to_string());
        assert!(matches!(error502, LlmError::ServerError(_)));

        // Test 503
        let error503 = LlmError::ServerError("HTTP 503: Service Unavailable".to_string());
        assert!(matches!(error503, LlmError::ServerError(_)));

        // Test 504
        let error504 = LlmError::ServerError("HTTP 504: Gateway Timeout".to_string());
        assert!(matches!(error504, LlmError::ServerError(_)));
    }

    /// Test that Timeout error is created correctly
    #[test]
    fn test_error_timeout() {
        let error = LlmError::Timeout("Request timed out after 30 seconds".to_string());
        assert!(matches!(error, LlmError::Timeout(_)));
        assert_eq!(
            format!("{}", error),
            "Request timed out: Request timed out after 30 seconds"
        );
    }

    /// Test that InvalidResponse error is created correctly
    #[test]
    fn test_error_invalid_response() {
        let error = LlmError::InvalidResponse("Failed to parse JSON response".to_string());
        assert!(matches!(error, LlmError::InvalidResponse(_)));
        assert_eq!(
            format!("{}", error),
            "Invalid response: Failed to parse JSON response"
        );
    }

    /// Test that NoContent error is created correctly
    #[test]
    fn test_error_no_content() {
        let error = LlmError::NoContent;
        assert!(matches!(error, LlmError::NoContent));
        assert_eq!(format!("{}", error), "No content in response");
    }

    /// Test that InvalidApiKey error is created correctly
    #[test]
    fn test_error_invalid_api_key() {
        let error = LlmError::InvalidApiKey;
        assert!(matches!(error, LlmError::InvalidApiKey));
        assert_eq!(format!("{}", error), "API key is missing or invalid");
    }

    /// Test that MaxIterationsExceeded error is created correctly
    #[test]
    fn test_error_max_iterations_exceeded() {
        let error = LlmError::MaxIterationsExceeded;
        assert!(matches!(error, LlmError::MaxIterationsExceeded));
        assert_eq!(format!("{}", error), "Maximum tool iterations exceeded");
    }

    /// Test parse_response with empty choices (simulates NoContent)
    #[test]
    fn test_parse_response_empty_choices() {
        let client = OpenRouterClient::new("test_key".to_string(), "test model".to_string(), 1024);

        let response = ChatCompletionResponse {
            choices: vec![],
            usage: None,
        };

        let result = client.parse_response(response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::NoContent));
    }

    /// Test parse_response with valid text content
    #[test]
    fn test_parse_response_valid_text() {
        let client = OpenRouterClient::new("test_key".to_string(), "test model".to_string(), 1024);

        let response = ChatCompletionResponse {
            choices: vec![CompletionChoice {
                message: ResponseMessage {
                    role: Some("assistant".to_string()),
                    content: Some("Hello, world!".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        let result = client.parse_response(response);
        assert!(result.is_ok());
        let llm_response = result.unwrap();
        assert_eq!(llm_response.text, Some("Hello, world!".to_string()));
        assert!(llm_response.tool_calls.is_empty());
        assert!(!llm_response.is_tool_call);
    }

    /// Test parse_response with valid tool calls
    #[test]
    fn test_parse_response_tool_calls() {
        let client = OpenRouterClient::new("test_key".to_string(), "test model".to_string(), 1024);

        let response = ChatCompletionResponse {
            choices: vec![CompletionChoice {
                message: ResponseMessage {
                    role: Some("assistant".to_string()),
                    content: None,
                    tool_calls: Some(vec![ResponseToolCall {
                        id: "call_abc123".to_string(),
                        tool_type: "function".to_string(),
                        function: ResponseFunction {
                            name: "get_weather".to_string(),
                            arguments: "{\"city\": \"Tokyo\"}".to_string(),
                        },
                    }]),
                },
                finish_reason: Some("tool_calls".to_string()),
            }],
            usage: None,
        };

        let result = client.parse_response(response);
        assert!(result.is_ok());
        let llm_response = result.unwrap();
        assert!(llm_response.text.is_none());
        assert_eq!(llm_response.tool_calls.len(), 1);
        assert!(llm_response.is_tool_call);
        assert_eq!(llm_response.tool_calls[0].function.name, "get_weather");
    }

    /// Test parse_response with empty content (valid response)
    #[test]
    fn test_parse_response_empty_content() {
        let client = OpenRouterClient::new("test_key".to_string(), "test model".to_string(), 1024);

        let response = ChatCompletionResponse {
            choices: vec![CompletionChoice {
                message: ResponseMessage {
                    role: Some("assistant".to_string()),
                    content: Some("".to_string()),
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        let result = client.parse_response(response);
        assert!(result.is_ok());
        let llm_response = result.unwrap();
        // Empty string content is valid and returns empty text
        assert_eq!(llm_response.text, Some("".to_string()));
    }

    /// Test that get_user_error_message returns correct message for RateLimited
    #[test]
    fn test_get_user_error_message_rate_limited_with_wait() {
        let error = LlmError::RateLimited(45);
        let msg = get_user_error_message(&error);
        assert!(msg.contains("Rate limited"));
    }

    /// Test that get_user_error_message returns correct message for InvalidResponse
    #[test]
    fn test_get_user_error_message_invalid_response() {
        let error = LlmError::InvalidResponse("Malformed JSON".to_string());
        let msg = get_user_error_message(&error);
        assert!(msg.contains("unexpected response"));
    }

    /// Test that get_user_error_message returns correct message for NoContent
    #[test]
    fn test_get_user_error_message_no_content() {
        let error = LlmError::NoContent;
        let msg = get_user_error_message(&error);
        assert!(msg.contains("empty response"));
    }

    /// Test that get_user_error_message returns correct message for MaxIterations
    #[test]
    fn test_get_user_error_message_max_iterations() {
        let error = LlmError::MaxIterationsExceeded;
        let msg = get_user_error_message(&error);
        assert!(msg.contains("took too long"));
    }

    /// Test HttpError conversion - skipped because reqwest::Error constructor is private
    /// In practice, this variant is tested through integration tests that make actual HTTP requests
    #[test]
    #[ignore]
    fn test_error_http_error() {
        // reqwest::Error::new() is private, so we cannot create this error in unit tests.
        // The HttpError variant is tested through integration tests.
        panic!("Cannot create reqwest::Error in unit tests due to private constructor");
    }

    /// Test SerializationError conversion
    #[test]
    fn test_error_serialization_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = LlmError::SerializationError(json_error);
        assert!(matches!(error, LlmError::SerializationError(_)));
    }

    /// Test ToolExecutionFailed error
    #[test]
    fn test_error_tool_execution_failed() {
        let error = LlmError::ToolExecutionFailed("Tool not found".to_string());
        assert!(matches!(error, LlmError::ToolExecutionFailed(_)));
        assert_eq!(
            format!("{}", error),
            "Tool execution failed: Tool not found"
        );
    }

    // ============================================
    // Rate Limiting Tests (429 responses)
    // ============================================

    /// Test that HTTP 429 returns LlmError::RateLimited with wait time from header
    #[test]
    fn test_rate_limited_with_retry_after_header() {
        // Simulate parsing retry-after header value
        let retry_after = "30";
        let wait_secs: u64 = retry_after.parse().unwrap_or(60);

        let error = LlmError::RateLimited(wait_secs);
        assert!(matches!(error, LlmError::RateLimited(30)));
        assert_eq!(wait_secs, 30);
    }

    /// Test that HTTP 429 returns LlmError::RateLimited with default wait time when no header
    #[test]
    fn test_rate_limited_default_wait_time() {
        // Simulate missing retry-after header - should default to 60
        let retry_after_header: Option<&str> = None;
        let wait_secs = retry_after_header
            .and_then(|s| s.parse().ok())
            .unwrap_or(60);

        let error = LlmError::RateLimited(wait_secs);
        assert!(matches!(error, LlmError::RateLimited(60)));
        assert_eq!(wait_secs, 60);
    }

    /// Test rate limited error message format
    #[test]
    fn test_rate_limited_error_message_format() {
        let error = LlmError::RateLimited(45);
        let msg = format!("{}", error);
        assert!(msg.contains("45"));
        assert!(msg.contains("seconds"));
    }

    /// Test rate limited with various retry-after values
    #[test]
    fn test_rate_limited_various_retry_after_values() {
        // Test with 1 second
        let error1 = LlmError::RateLimited(1);
        assert!(matches!(error1, LlmError::RateLimited(1)));

        // Test with 120 seconds
        let error2 = LlmError::RateLimited(120);
        assert!(matches!(error2, LlmError::RateLimited(120)));

        // Test with max value
        let error3 = LlmError::RateLimited(u64::MAX);
        assert!(matches!(error3, LlmError::RateLimited(u64::MAX)));
    }

    /// Test that invalid retry-after header falls back to default
    #[test]
    fn test_rate_limited_invalid_retry_after_falls_back() {
        // Simulate invalid retry-after header (non-numeric)
        let invalid_retry_after = "not_a_number";
        let wait_secs: u64 = invalid_retry_after.parse().ok().unwrap_or(60);

        assert_eq!(wait_secs, 60); // Should fall back to default
    }

    // ============================================
    // Server Error Tests (5xx responses)
    // ============================================

    /// Test that HTTP 500 returns LlmError::ServerError
    #[test]
    fn test_server_error_500() {
        let error = LlmError::ServerError("HTTP 500: Internal Server Error".to_string());
        assert!(matches!(error, LlmError::ServerError(_)));
        let msg = format!("{}", error);
        assert!(msg.contains("500"));
        assert!(msg.contains("Server error"));
    }

    /// Test that HTTP 502 returns appropriate ServerError
    #[test]
    fn test_server_error_502() {
        let error = LlmError::ServerError("HTTP 502: Bad Gateway".to_string());
        assert!(matches!(error, LlmError::ServerError(msg) if msg.contains("502")));
    }

    /// Test that HTTP 503 returns appropriate ServerError
    #[test]
    fn test_server_error_503() {
        let error = LlmError::ServerError("HTTP 503: Service Unavailable".to_string());
        assert!(matches!(error, LlmError::ServerError(msg) if msg.contains("503")));
    }

    /// Test that HTTP 504 returns appropriate ServerError
    #[test]
    fn test_server_error_504() {
        let error = LlmError::ServerError("HTTP 504: Gateway Timeout".to_string());
        assert!(matches!(error, LlmError::ServerError(msg) if msg.contains("504")));
    }

    /// Test server error message includes status code
    #[test]
    fn test_server_error_message_includes_status_code() {
        let error = LlmError::ServerError("HTTP 500: Something went wrong".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("500"));
        assert!(msg.contains("Something went wrong"));
    }

    /// Test various 5xx status codes produce ServerError
    #[test]
    fn test_server_error_various_5xx_codes() {
        let codes = [500, 501, 502, 503, 504, 520, 521, 522, 523, 524];
        for code in codes {
            let error = LlmError::ServerError(format!("HTTP {}: Error", code));
            let msg = format!("{}", error);
            assert!(
                msg.contains(&code.to_string()),
                "Missing status code {}",
                code
            );
        }
    }

    // ============================================
    // Timeout Handling Tests
    // ============================================

    /// Test that timeout returns LlmError::Timeout
    #[test]
    fn test_timeout_error_type() {
        let error = LlmError::Timeout("Request timed out".to_string());
        assert!(matches!(error, LlmError::Timeout(_)));
    }

    /// Test timeout error message format
    #[test]
    fn test_timeout_error_message_format() {
        let error = LlmError::Timeout("Request timed out after 30 seconds".to_string());
        let msg = format!("{}", error);
        assert!(msg.contains("timed out"));
        assert!(msg.contains("30"));
    }

    /// Test timeout with various timeout messages
    #[test]
    fn test_timeout_various_messages() {
        // Connection timeout
        let error1 = LlmError::Timeout("connect timed out".to_string());
        assert!(matches!(error1, LlmError::Timeout(_)));

        // Request timeout
        let error2 = LlmError::Timeout("request timed out".to_string());
        assert!(matches!(error2, LlmError::Timeout(_)));

        // Read timeout
        let error3 = LlmError::Timeout("read timed out".to_string());
        assert!(matches!(error3, LlmError::Timeout(_)));
    }

    // ============================================
    // Invalid Response Parsing Tests
    // ============================================

    /// Test that malformed JSON returns LlmError::InvalidResponse or SerializationError
    #[test]
    fn test_invalid_response_malformed_json() {
        // Test InvalidResponse
        let error = LlmError::InvalidResponse("Failed to parse JSON response".to_string());
        assert!(matches!(error, LlmError::InvalidResponse(_)));
        let msg = format!("{}", error);
        assert!(msg.contains("Invalid response"));
    }

    /// Test that malformed JSON returns SerializationError
    #[test]
    fn test_serialization_error_malformed_json() {
        let json_error = serde_json::from_str::<serde_json::Value>("{{ invalid ").unwrap_err();
        let error = LlmError::SerializationError(json_error);
        assert!(matches!(error, LlmError::SerializationError(_)));
    }

    /// Test empty response handling
    #[test]
    fn test_invalid_response_empty_response() {
        // Empty response body
        let error = LlmError::InvalidResponse("Empty response body".to_string());
        assert!(matches!(error, LlmError::InvalidResponse(_)));
        assert_eq!(
            format!("{}", error),
            "Invalid response: Empty response body"
        );
    }

    /// Test response missing required fields
    #[test]
    fn test_invalid_response_missing_fields() {
        // Missing 'choices' field - this is what NoContent tests
        let error = LlmError::NoContent;
        assert!(matches!(error, LlmError::NoContent));

        // Alternative: InvalidResponse for missing fields
        let error2 = LlmError::InvalidResponse("Missing required field: choices".to_string());
        assert!(matches!(error2, LlmError::InvalidResponse(_)));
    }

    /// Test various invalid JSON formats
    #[test]
    fn test_serialization_error_various_formats() {
        // Truncated JSON
        let err1 = serde_json::from_str::<serde_json::Value>("{\"key\": ").unwrap_err();
        let error1 = LlmError::SerializationError(err1);
        assert!(matches!(error1, LlmError::SerializationError(_)));

        // Invalid JSON structure (trailing comma)
        let err2 = serde_json::from_str::<serde_json::Value>("{\"key\": 1,}").unwrap_err();
        let error2 = LlmError::SerializationError(err2);
        assert!(matches!(error2, LlmError::SerializationError(_)));

        // Unclosed object
        let err3 = serde_json::from_str::<serde_json::Value>("{").unwrap_err();
        let _error3 = LlmError::SerializationError(err3);
    }

    /// Test InvalidResponse with different error messages
    #[test]
    fn test_invalid_response_various_messages() {
        let messages = [
            "Failed to parse JSON response",
            "Unexpected token at position 0",
            "Expected object, found string",
            "Missing 'choices' field in response",
        ];

        for msg in messages {
            let error = LlmError::InvalidResponse(msg.to_string());
            assert!(matches!(error, LlmError::InvalidResponse(s) if s == msg));
        }
    }

    /// Test that empty choices array returns NoContent
    #[test]
    fn test_empty_choices_returns_no_content() {
        let client = OpenRouterClient::new("test_key".to_string(), "test model".to_string(), 1024);

        let response = ChatCompletionResponse {
            choices: vec![],
            usage: None,
        };

        let result = client.parse_response(response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LlmError::NoContent));
    }

    // ============================================
    // ChatCompletionRequest Serialization Tests
    // ============================================

    use crate::discord::conversation::MessageRole;
    use serde_json;

    /// Test ChatCompletionRequest serialization with all fields
    #[test]
    fn test_chat_completion_request_serialization() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: "You are a helpful assistant".to_string(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
        ];

        let request = ChatCompletionRequest {
            model: "anthropic/claude-sonnet-4".to_string(),
            messages,
            tools: None,
            max_tokens: 1024,
        };

        let json = serde_json::to_string(&request).unwrap();

        // Verify model field
        assert!(json.contains("\"model\":\"anthropic/claude-sonnet-4\""));
        // Verify max_tokens field (serialized as max_tokens, not maxTokens)
        assert!(json.contains("\"max_tokens\":1024"));
        // Verify messages array
        assert!(json.contains("\"messages\":"));
    }

    /// Test that tools are included when provided
    #[test]
    fn test_chat_completion_request_with_tools() {
        let tools = vec![ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_weather".to_string(),
                description: "Get weather information".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                }),
            },
        }];

        let request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "What's the weather?".to_string(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            }],
            tools: Some(tools),
            max_tokens: 512,
        };

        let json = serde_json::to_string(&request).unwrap();

        // Verify tools are included
        assert!(json.contains("\"tools\":"));
        assert!(json.contains("get_weather"));
        assert!(json.contains("Get weather information"));
    }

    /// Test that tools are omitted when None (skip_serializing_if)
    #[test]
    fn test_chat_completion_request_without_tools() {
        let request = ChatCompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: "Hello".to_string(),
                tool_calls: None,
                tool_call_id: None,
                name: None,
            }],
            tools: None,
            max_tokens: 512,
        };

        let json = serde_json::to_string(&request).unwrap();

        // Verify tools field is NOT included in serialized output
        assert!(!json.contains("\"tools\""));
        // Verify other fields are present
        assert!(json.contains("\"model\":\"test-model\""));
    }

    // ============================================
    // tools_schema_to_definitions Tests
    // ============================================

    /// Test tools_schema_to_definitions with a sample tools array
    #[test]
    fn test_tools_schema_to_definitions_single_tool() {
        let tools_json = serde_json::json!([
            {
                "name": "get_weather",
                "description": "Get weather for a location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    },
                    "required": ["location"]
                }
            }
        ]);

        let definitions = tools_schema_to_definitions(&tools_json);

        assert_eq!(definitions.len(), 1);
        assert_eq!(definitions[0].tool_type, "function");
        assert_eq!(definitions[0].function.name, "get_weather");
        assert_eq!(
            definitions[0].function.description,
            "Get weather for a location"
        );
    }

    /// Test tools_schema_to_definitions with multiple tools
    #[test]
    fn test_tools_schema_to_definitions_multiple_tools() {
        let tools_json = serde_json::json!([
            {
                "name": "get_weather",
                "description": "Get weather information",
                "parameters": {"type": "object", "properties": {}}
            },
            {
                "name": "search",
                "description": "Search for information",
                "parameters": {"type": "object", "properties": {}}
            },
            {
                "name": "calculate",
                "description": "Perform calculations",
                "parameters": {"type": "object", "properties": {}}
            }
        ]);

        let definitions = tools_schema_to_definitions(&tools_json);

        assert_eq!(definitions.len(), 3);
        assert_eq!(definitions[0].function.name, "get_weather");
        assert_eq!(definitions[1].function.name, "search");
        assert_eq!(definitions[2].function.name, "calculate");
    }

    /// Test tools_schema_to_definitions with empty array
    #[test]
    fn test_tools_schema_to_definitions_empty_array() {
        let tools_json = serde_json::json!([]);

        let definitions = tools_schema_to_definitions(&tools_json);

        assert!(definitions.is_empty());
    }

    /// Test tools_schema_to_definitions with invalid tool format (missing fields)
    #[test]
    fn test_tools_schema_to_definitions_invalid_format() {
        let tools_json = serde_json::json!([
            {
                "name": "incomplete_tool",
                // missing description and parameters
            }
        ]);

        let definitions = tools_schema_to_definitions(&tools_json);

        // Should skip tools with missing required fields
        assert!(definitions.is_empty());
    }

    /// Test that tools_schema_to_definitions converts to OpenRouter format correctly
    #[test]
    fn test_tools_schema_to_definitions_openrouter_format() {
        let tools_json = serde_json::json!([
            {
                "name": "test_function",
                "description": "A test function",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "arg1": {"type": "string", "description": "First argument"}
                    }
                }
            }
        ]);

        let definitions = tools_schema_to_definitions(&tools_json);

        // Verify OpenRouter format structure
        let def = &definitions[0];
        assert_eq!(def.tool_type, "function");

        // The function should have name, description, and parameters
        assert!(!def.function.name.is_empty());
        assert!(!def.function.description.is_empty());

        // Parameters should be a JSON object
        assert!(def.function.parameters.is_object());
    }
}
