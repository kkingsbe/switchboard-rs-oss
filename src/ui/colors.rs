//! Color utilities for CLI output
//!
//! This module provides colored output support for the CLI, including:
//! - Color palette definitions
//! - Helper functions for colored text output
//! - Color mode detection (auto, always, never)

use clap::ValueEnum;
use std::env;

/// Color mode for CLI output
///
/// Controls when to use colored output:
/// - Auto: Detect terminal support (default)
/// - Always: Always use colors
/// - Never: Never use colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

impl ColorMode {
    /// Parse a string into a ColorMode
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(ColorMode::Auto),
            "always" => Some(ColorMode::Always),
            "never" => Some(ColorMode::Never),
            _ => None,
        }
    }
}

/// Color palette for CLI output
pub mod palette {
    use ansi_term::Style;

    /// Error color - Red (#FF5555)
    pub fn error() -> Style {
        Style::new().fg(ansi_term::Color::Fixed(255))
    }

    /// Warning color - Yellow (#F1FA8C)
    pub fn warning() -> Style {
        Style::new().fg(ansi_term::Color::Fixed(229))
    }

    /// Success color - Green (#50FA7B)
    pub fn success() -> Style {
        Style::new().fg(ansi_term::Color::Fixed(80))
    }

    /// Info color - Cyan (#8BE9FD)
    pub fn info() -> Style {
        Style::new().fg(ansi_term::Color::Fixed(139))
    }

    /// Header color - Bold Cyan
    pub fn header() -> Style {
        Style::new().bold().fg(ansi_term::Color::Fixed(139))
    }
}

/// Determine if colors should be used based on color mode and terminal
///
/// # Arguments
/// * `mode` - The color mode to use
///
/// # Returns
/// Returns true if colors should be displayed, false otherwise
pub fn should_use_colors(mode: ColorMode) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            // Check if stdout is a TTY (terminal)
            // Also check NO_COLOR environment variable
            if env::var("NO_COLOR").is_ok() {
                return false;
            }
            atty::is(atty::Stream::Stdout)
        }
    }
}

/// Colorize a message with the error style (red)
///
/// # Arguments
/// * `msg` - The message to colorize
///
/// # Returns
/// The colorized string
pub fn color_error(msg: &str) -> String {
    palette::error().paint(msg).to_string()
}

/// Colorize a message with the warning style (yellow)
///
/// # Arguments
/// * `msg` - The message to colorize
///
/// # Returns
/// The colorized string
pub fn color_warning(msg: &str) -> String {
    palette::warning().paint(msg).to_string()
}

/// Colorize a message with the success style (green)
///
/// # Arguments
/// * `msg` - The message to colorize
///
/// # Returns
/// The colorized string
pub fn color_success(msg: &str) -> String {
    palette::success().paint(msg).to_string()
}

/// Colorize a message with the info style (cyan)
///
/// # Arguments
/// * `msg` - The message to colorize
///
/// # Returns
/// The colorized string
pub fn color_info(msg: &str) -> String {
    palette::info().paint(msg).to_string()
}

/// Colorize a message with the header style (bold cyan)
///
/// # Arguments
/// * `msg` - The message to colorize
///
/// # Returns
/// The colorized string
pub fn color_header(msg: &str) -> String {
    palette::header().paint(msg).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_mode_from_str() {
        assert_eq!(ColorMode::from_str("auto"), Some(ColorMode::Auto));
        assert_eq!(ColorMode::from_str("always"), Some(ColorMode::Always));
        assert_eq!(ColorMode::from_str("never"), Some(ColorMode::Never));
        assert_eq!(ColorMode::from_str("invalid"), None);
        assert_eq!(ColorMode::from_str("AUTO"), Some(ColorMode::Auto));
    }

    #[test]
    fn test_color_output() {
        // These just verify the functions don't panic
        let _ = color_error("error message");
        let _ = color_warning("warning message");
        let _ = color_success("success message");
        let _ = color_info("info message");
        let _ = color_header("header message");
    }
}
