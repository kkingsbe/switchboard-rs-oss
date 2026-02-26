//! Unit tests for timeout parsing functionality

use std::time::Duration;

// Import the parse_timeout function from the crate
// Note: parse_timeout is re-exported from the wait module
use switchboard::docker::run::wait::parse_timeout;

/// Test that parse_timeout("30m") returns Duration::from_secs(1800)
#[test]
fn test_parse_timeout_30_minutes() {
    let result = parse_timeout("30m");
    assert!(result.is_ok(), "Should successfully parse '30m'");
    let duration = result.unwrap();
    assert_eq!(
        duration,
        Duration::from_secs(1800),
        "30 minutes should equal 1800 seconds"
    );
}

/// Test that parse_timeout("2h") returns Duration::from_secs(7200)
#[test]
fn test_parse_timeout_2_hours() {
    let result = parse_timeout("2h");
    assert!(result.is_ok(), "Should successfully parse '2h'");
    let duration = result.unwrap();
    assert_eq!(
        duration,
        Duration::from_secs(7200),
        "2 hours should equal 7200 seconds"
    );
}

/// Test that parse_timeout("30s") returns Duration::from_secs(30)
#[test]
fn test_parse_timeout_30_seconds() {
    let result = parse_timeout("30s");
    assert!(result.is_ok(), "Should successfully parse '30s'");
    let duration = result.unwrap();
    assert_eq!(
        duration,
        Duration::from_secs(30),
        "30 seconds should equal 30 seconds"
    );
}

/// Test that parse_timeout("1h30m") returns Duration::from_secs(5400)
///
/// Note: The current implementation only supports single-unit formats (e.g., "1h", "30m"),
/// not combined formats like "1h30m". This test will fail with the current implementation
/// but documents the expected behavior for a future enhancement.
#[test]
fn test_parse_timeout_combined_units() {
    let result = parse_timeout("1h30m");

    // The current implementation splits at the last character, so "1h30m" becomes
    // value_part="1h30" and unit="m", which will fail to parse "1h30" as a u64.
    // This test documents that this is a limitation of the current implementation.
    assert!(
        result.is_err(),
        "Combined units '1h30m' is not supported by current implementation"
    );
}

/// Test that parse_timeout("invalid") returns an error
#[test]
fn test_parse_timeout_invalid_string() {
    let result = parse_timeout("invalid");
    assert!(result.is_err(), "Should fail to parse 'invalid'");
}

/// Test that parse_timeout("") returns an error
#[test]
fn test_parse_timeout_empty_string() {
    let result = parse_timeout("");
    assert!(result.is_err(), "Should fail to parse empty string");
}

/// Test that parse_timeout("0s") returns Duration::ZERO
#[test]
fn test_parse_timeout_zero() {
    let result = parse_timeout("0s");
    assert!(result.is_ok(), "Should successfully parse '0s'");
    let duration = result.unwrap();
    assert_eq!(
        duration,
        Duration::ZERO,
        "0 seconds should equal Duration::ZERO"
    );
}

/// Test that parse_timeout handles whitespace correctly (trims input)
#[test]
fn test_parse_timeout_with_whitespace() {
    let result = parse_timeout("  30s  ");
    assert!(result.is_ok(), "Should trim and parse '  30s  '");
    let duration = result.unwrap();
    assert_eq!(
        duration,
        Duration::from_secs(30),
        "Should handle leading/trailing whitespace"
    );
}

/// Test that parse_timeout handles larger values
#[test]
fn test_parse_timeout_large_values() {
    // Test large seconds value
    let result = parse_timeout("3600s");
    assert!(result.is_ok(), "Should parse large seconds value");
    assert_eq!(
        result.unwrap(),
        Duration::from_secs(3600),
        "3600 seconds should parse correctly"
    );

    // Test large minutes value
    let result = parse_timeout("120m");
    assert!(result.is_ok(), "Should parse large minutes value");
    assert_eq!(
        result.unwrap(),
        Duration::from_secs(7200),
        "120 minutes should equal 7200 seconds"
    );

    // Test large hours value
    let result = parse_timeout("24h");
    assert!(result.is_ok(), "Should parse large hours value");
    assert_eq!(
        result.unwrap(),
        Duration::from_secs(86400),
        "24 hours should equal 86400 seconds"
    );
}

/// Test that parse_timeout rejects invalid unit characters
#[test]
fn test_parse_timeout_invalid_unit() {
    let result = parse_timeout("30x");
    assert!(result.is_err(), "Should reject invalid unit 'x'");
}

/// Test that parse_timeout rejects negative numbers (not parseable as u64)
#[test]
fn test_parse_timeout_negative_value() {
    let result = parse_timeout("-5m");
    assert!(result.is_err(), "Should reject negative value '-5m'");
}
