//! Test utilities module for the Switchboard REST API.
//!
//! This module provides common utilities for testing API handlers including
//! mock state builders, test fixtures, and HTTP client utilities.
//!
//! # Usage
//!
//! ```ignore
//! use switchboard::api::tests::{
//!     TestApiStateBuilder, test_fixtures,
//! };
//!
//! // Create a test state with mocked dependencies
//! let state = TestApiStateBuilder::new()
//!     .with_test_instance()
//!     .build();
//! ```

pub mod client;
pub mod fixtures;
pub mod state;

// Re-export commonly used types
pub use fixtures::*;
pub use state::{
    create_test_state,
    create_test_state_with_config,
    create_test_state_with_id,
    TestApiStateBuilder,
};
