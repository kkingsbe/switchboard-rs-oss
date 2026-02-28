//! Signal handling utilities for the CLI
//!
//! This module provides cross-platform signal handling for gracefully shutting down
//! the scheduler. On Unix systems, it handles SIGTERM, SIGINT, and Ctrl+C. On Windows,
//! it only handles Ctrl+C.

#[cfg(unix)]
use tokio::signal::unix::{signal, SignalKind};

/// Waits for a shutdown signal (Ctrl+C, SIGTERM, or SIGINT).
///
/// On Unix systems, waits for any of:
/// - Ctrl+C (via tokio's ctrl_c)
/// - SIGTERM
/// - SIGINT
///
/// On Windows, waits only for Ctrl+C.
///
/// When a signal is received, logs an info message and returns.
///
/// # Returns
/// * `Ok(())` if signal handling completed successfully
/// * `Err(...)` if there was an error setting up signal handlers
pub fn setup_signal_handlers() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        // On Unix, handle SIGTERM and SIGINT in addition to ctrl_c()
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?;

        tokio::select! {
            // Wait for Ctrl+C signal
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received Ctrl+C, stopping scheduler...");
            }
            // Wait for SIGTERM
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM, stopping scheduler...");
            }
            // Wait for SIGINT
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT, stopping scheduler...");
            }
        }
    }

    #[cfg(windows)]
    {
        // On Windows, only ctrl_c() is available
        tokio::select! {
            // Wait for Ctrl+C signal
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received Ctrl+C, stopping scheduler...");
            }
        }
    }

    Ok(())
}
