/// Converts 5-field Unix cron to 6-field cron format by prepending "0".
///
/// This function prepends "0 " (run at second 0) to 5-field expressions.
/// 6-field expressions are passed through unchanged.
///
/// # Arguments
///
/// * `schedule` - A cron schedule string (either 5-field or 6-field format)
///
/// # Returns
///
/// A 6-field cron schedule string with "0" prepended if input was 5-field,
/// or the original string if already 6-field
pub fn convert_to_6_field_cron(schedule: &str) -> String {
    let fields: Vec<&str> = schedule.split_whitespace().collect();
    if fields.len() == 5 {
        format!("0 {}", schedule.trim())
    } else {
        schedule.to_string()
    }
}
