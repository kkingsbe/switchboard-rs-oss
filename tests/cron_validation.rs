//! Unit tests for cron expression validation functionality

#[cfg(feature = "scheduler")]
use switchboard::config::validate_cron_expression;

/// Test that valid cron expressions pass validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_valid() {
    // Test standard valid cron expressions
    let valid_expressions = vec![
        "0 */6 * * *",    // Every 6 hours
        "* * * * *",      // Every minute
        "0 0 * * *",      // Daily at midnight
        "0 0 * * 0",      // Weekly on Sunday at midnight
        "0 0 1 * *",      // Monthly on the 1st at midnight
        "*/5 * * * *",    // Every 5 minutes
        "0 */2 * * *",    // Every 2 hours
        "0 9-17 * * 1-5", // Every hour from 9 AM to 5 PM on weekdays
        "30 4 1 Jan *",   // January 1st at 4:30 AM
    ];

    for expr in valid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_ok(),
            "Valid cron expression '{}' should pass validation, but got error: {:?}",
            expr,
            result
        );
    }
}

/// Test that invalid minute field fails validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_invalid_minute() {
    // Test invalid minute values (valid range: 0-59)
    let invalid_expressions = vec![
        "60 * * * *", // 60 is invalid (minutes 0-59)
        "70 * * * *", // 70 is invalid
        "99 * * * *", // 99 is invalid
        "-1 * * * *", // Negative value is invalid
    ];

    for expr in invalid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_err(),
            "Invalid minute expression '{}' should fail validation",
            expr
        );

        // Verify the error message includes the invalid expression
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains(expr),
            "Error message should contain the invalid expression '{}', got: {}",
            expr,
            error_msg
        );

        // Verify the error message includes a helpful description
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for invalid expression '{}'",
            expr
        );
    }
}

/// Test that invalid hour field fails validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_invalid_hour() {
    // Test invalid hour values (valid range: 0-23)
    let invalid_expressions = vec![
        "* 25 * * *", // 25 is invalid (hours 0-23)
        "* 24 * * *", // 24 is invalid
        "* 99 * * *", // 99 is invalid
        "* -1 * * *", // Negative value is invalid
    ];

    for expr in invalid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_err(),
            "Invalid hour expression '{}' should fail validation",
            expr
        );

        // Verify the error message includes the invalid expression
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains(expr),
            "Error message should contain the invalid expression '{}', got: {}",
            expr,
            error_msg
        );

        // Verify the error message includes a helpful description
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for invalid expression '{}'",
            expr
        );
    }
}

/// Test that invalid day of month field fails validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_invalid_day_of_month() {
    // Test invalid day of month values (valid range: 1-31)
    let invalid_expressions = vec![
        "* * 32 * *", // 32 is invalid (days 1-31)
        "* * 99 * *", // 99 is invalid
        "* * 0 * *",  // 0 is invalid (days start at 1)
        "* * -1 * *", // Negative value is invalid
    ];

    for expr in invalid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_err(),
            "Invalid day of month expression '{}' should fail validation",
            expr
        );

        // Verify the error message includes the invalid expression
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains(expr),
            "Error message should contain the invalid expression '{}', got: {}",
            expr,
            error_msg
        );

        // Verify the error message includes a helpful description
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for invalid expression '{}'",
            expr
        );
    }
}

/// Test that invalid month field fails validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_invalid_month() {
    // Test invalid month values (valid range: 1-12)
    let invalid_expressions = vec![
        "* * * 13 *", // 13 is invalid (months 1-12)
        "* * * 99 *", // 99 is invalid
        "* * * 0 *",  // 0 is invalid (months start at 1)
        "* * * -1 *", // Negative value is invalid
    ];

    for expr in invalid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_err(),
            "Invalid month expression '{}' should fail validation",
            expr
        );

        // Verify the error message includes the invalid expression
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains(expr),
            "Error message should contain the invalid expression '{}', got: {}",
            expr,
            error_msg
        );

        // Verify the error message includes a helpful description
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for invalid expression '{}'",
            expr
        );
    }
}

/// Test that invalid day of week field fails validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_invalid_day_of_week() {
    // Test invalid day of week values (valid range: 0-6 or 0-7 depending on implementation)
    let invalid_expressions = vec![
        "* * * * 8",  // 8 is invalid (days 0-6 or 0-7)
        "* * * * 99", // 99 is invalid
        "* * * * -1", // Negative value is invalid
    ];

    for expr in invalid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_err(),
            "Invalid day of week expression '{}' should fail validation",
            expr
        );

        // Verify the error message includes the invalid expression
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains(expr),
            "Error message should contain the invalid expression '{}', got: {}",
            expr,
            error_msg
        );

        // Verify the error message includes a helpful description
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for invalid expression '{}'",
            expr
        );
    }
}

/// Test that completely invalid format fails validation
#[test]
#[cfg(feature = "scheduler")]
fn test_validate_cron_expression_invalid_format() {
    // Test completely invalid formats
    let invalid_expressions = vec![
        "not-a-cron",          // Not a cron expression at all
        "",                    // Empty string
        "* * * *",             // Missing a field
        "* * * * * * *",       // Too many fields (7 fields instead of 5 or 6)
        "abc def ghi jkl mno", // Non-numeric fields
    ];

    for expr in invalid_expressions {
        let result = validate_cron_expression(expr);
        assert!(
            result.is_err(),
            "Invalid format expression '{}' should fail validation",
            expr
        );

        // Verify the error message includes the invalid expression
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(
            error_msg.contains(expr),
            "Error message should contain the invalid expression '{}', got: {}",
            expr,
            error_msg
        );

        // Verify the error message includes a helpful description
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for invalid expression '{}'",
            expr
        );
    }
}
