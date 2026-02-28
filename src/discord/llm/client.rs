//! OpenRouter API client implementation.
//!
//! This module provides the client logic for interacting with the OpenRouter API,
//! including request building, HTTP communication, and retry logic.

use crate::discord::conversation::{ChatMessage, ToolCall, ToolFunction};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;
use tracing::warn;

use super::response::{ChatCompletionResponse, LlmResponse, ToolCallResult};

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
    pub tool_type: String,
    /// Function definition
    pub function: FunctionDefinition,
}

/// Function definition for tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
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
                    .unwrap_or_else(|| {
                        warn!("Missing or invalid retry-after header, using default 60s");
                        60
                    });
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
    pub fn parse_response(&self, response: ChatCompletionResponse) -> LlmResult<LlmResponse> {
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
