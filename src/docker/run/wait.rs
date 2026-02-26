//! Container exit wait and timeout handling
//!
//! This module provides functionality for:
//! - Parsing timeout strings (e.g., "30s", "5m", "1h")
//! - Waiting for container exit with polling
//! - Handling container timeouts with automatic termination

pub use self::timeout::{parse_timeout, wait_for_exit, wait_with_timeout};
pub use self::types::{ExitStatus, TerminationSignal};

mod timeout;
mod types;
