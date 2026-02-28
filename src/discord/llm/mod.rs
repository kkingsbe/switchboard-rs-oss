//! LLM integration module for Discord concierge.
//!
//! This module provides integration with OpenRouter for generating responses
//! to user messages. It implements a tool-use loop that allows the LLM to
//! call tools (like file operations, status checks, etc.) and return results.
//!
//! # Architecture
//!
//! - [`client::OpenRouterClient`] - Main client for OpenRouter API interactions
//! - [`response::LlmResponse`] - Parsed response from the LLM
//! - [`response::ToolCallResult`] - Result of executing a tool call
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

pub mod client;
pub mod response;

// Re-export all public types from submodules for backward compatibility
pub use client::{
    process_with_tools, tools_schema_to_definitions, LlmError, LlmResult, OpenRouterClient,
    ToolDefinition, ToolExecutor,
};
pub use response::{
    get_user_error_message, ChatCompletionResponse, CompletionChoice, LlmResponse,
    ResponseFunction, ResponseMessage, ResponseToolCall, ToolCallResult, Usage,
};
