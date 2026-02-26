//! Unit tests for Switchboard CLI tool.
//!
//! This module contains unit tests that test individual components
//! in isolation without requiring external dependencies like Docker.
//!
//! # Using Test Utilities
//!
//! To use the common test utilities in your unit tests, add the following
//! to your test file:
//!
//! ```ignore
//! use switchboard::common::fixtures;\n use switchboard::common::assertions;\n ```
//!
//! Or reference them directly:
//!
//! ```ignore
//! // Use fixtures\n let config = switchboard::common::fixtures::sample_skill_config();\n\n // Use assertion macros\n switchboard::common::assertions::some_macro!(...);\n ```
//!
//! Note: The common utilities are exposed through the main library crate.
//! Make sure the `common` feature is enabled in your Cargo.toml:
//!
//! ```toml\n[dev-dependencies]\nswitchboard = { path = \"..\", features = [\"common\"] }\n```
