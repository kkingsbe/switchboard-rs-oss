//! UI module for CLI output styling
//!
//! This module provides colored output utilities for the Switchboard CLI.
//!
//! # Features
//!
//! - Color palette with predefined styles for errors, warnings, success, info, and headers
//! - Color mode detection (auto, always, never) via --color flag
//! - Helper functions for easy colored text output
//!
//! # Usage
//!
//! ```rust
//! use switchboard::ui::colors::{color_error, color_success, color_info};
//!
//! // Print colored output
//! println!("{}", color_error("This is an error"));
//! println!("{}", color_success("Operation succeeded"));
//! println!("{}", color_info("Some information"));
//! ```

pub mod colors;

// Re-export commonly used items for easier access
pub use colors::{
    color_error, color_header, color_info, color_success, color_warning, should_use_colors,
    ColorMode,
};
