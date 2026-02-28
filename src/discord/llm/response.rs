//! Response types and parsing for LLM API.
//!
//! This module contains all response types from the OpenRouter API,
//! as well as helper functions for processing and formatting responses.

use crate::discord::conversation::{ToolCall, ToolFunction};
use serde::{Deserialize, Serialize};

use super::client::LlmError;

/// OpenRouter response payload
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    /// Array of completion choices
    pub choices: Vec<CompletionChoice>,
    /// Usage statistics
    #[allow(dead_code)]
    #[serde(default)]
    pub usage: Option<Usage>,
}

/// A single completion choice
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CompletionChoice {
    /// The completed message
    pub message: ResponseMessage,
    /// Why the completion finished
    #[serde(default)]
    pub finish_reason: Option<String>,
}

/// Response message from the LLM
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ResponseMessage {
    /// Role of the message sender
    #[serde(default)]
    pub role: Option<String>,
    /// Content of the message
    #[serde(default)]
    pub content: Option<String>,
    /// Tool calls requested by the LLM
    #[serde(default, rename = "tool_calls")]
    pub tool_calls: Option<Vec<ResponseToolCall>>,
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
pub struct Usage {
    /// Tokens used in the prompt
    #[serde(rename = "prompt_tokens")]
    pub prompt_tokens: Option<u32>,
    /// Tokens in the completion
    #[serde(rename = "completion_tokens")]
    pub completion_tokens: Option<u32>,
    /// Total tokens used
    #[serde(rename = "total_tokens")]
    pub total_tokens: Option<u32>,
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
