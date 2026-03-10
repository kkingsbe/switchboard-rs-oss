//! API Error types and handling.
//!
//! This module provides error types for the REST API, including
//! JSON serialization support for API responses.

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// API error response structure for JSON responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Error code for programmatic error handling.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional details about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiErrorResponse {
    /// Create a new API error response.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Create a new API error response with details.
    pub fn with_details(code: impl Into<String>, message: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }
}

/// API Result type for handler functions.
pub type ApiResult<T> = Result<T, ApiError>;

/// Main API error type.
#[derive(Debug)]
pub enum ApiError {
    /// Bad request error (400).
    BadRequest(String),
    /// Unauthorized error (401).
    Unauthorized(String),
    /// Forbidden error (403).
    Forbidden(String),
    /// Not found error (404).
    NotFound(String),
    /// Internal server error (500).
    Internal(String),
    /// Service unavailable error (503).
    ServiceUnavailable(String),
    /// Configuration error.
    Config(String),
    /// Serialization/Deserialization error.
    Serialization(String),
    /// Not implemented error (501).
    NotImplemented(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            ApiError::Internal(msg) => write!(f, "Internal Server Error: {}", msg),
            ApiError::ServiceUnavailable(msg) => write!(f, "Service Unavailable: {}", msg),
            ApiError::Config(msg) => write!(f, "Configuration Error: {}", msg),
            ApiError::Serialization(msg) => write!(f, "Serialization Error: {}", msg),
            ApiError::NotImplemented(msg) => write!(f, "Not Implemented: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ApiError::BadRequest(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg,
            ),
            ApiError::Unauthorized(msg) => (
                axum::http::StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                msg,
            ),
            ApiError::Forbidden(msg) => (
                axum::http::StatusCode::FORBIDDEN,
                "FORBIDDEN",
                msg,
            ),
            ApiError::NotFound(msg) => (
                axum::http::StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg,
            ),
            ApiError::Internal(msg) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg,
            ),
            ApiError::ServiceUnavailable(msg) => (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                msg,
            ),
            ApiError::Config(msg) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
                msg,
            ),
            ApiError::Serialization(msg) => (
                axum::http::StatusCode::BAD_REQUEST,
                "SERIALIZATION_ERROR",
                msg,
            ),
            ApiError::NotImplemented(msg) => (
                axum::http::StatusCode::NOT_IMPLEMENTED,
                "NOT_IMPLEMENTED",
                msg,
            ),
        };

        let body = Json(ApiErrorResponse::new(code, message));
        (status, body).into_response()
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::Serialization(err.to_string())
    }
}

impl From<toml::de::Error> for ApiError {
    fn from(err: toml::de::Error) -> Self {
        ApiError::Serialization(err.to_string())
    }
}
