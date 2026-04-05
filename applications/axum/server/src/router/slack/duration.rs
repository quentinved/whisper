const MAX_DURATION_SECONDS: i64 = 7 * 86400; // 7 days

/// Parses a duration string like "1h", "30m", "24h", "7d" into seconds.
/// Returns None if the format is invalid or exceeds 7 days.
///
/// Supported suffixes:
/// - `m` for minutes (e.g., "30m" = 1800 seconds)
/// - `h` for hours (e.g., "1h" = 3600 seconds)
/// - `d` for days (e.g., "7d" = 604800 seconds)
pub fn parse_duration(input: &str) -> Option<i64> {
    let input = input.trim();
    if input.len() < 2 {
        return None;
    }

    let (num_str, suffix) = input.split_at(input.len() - 1);
    let value: i64 = num_str.parse().ok()?;

    if value <= 0 {
        return None;
    }

    let seconds = match suffix {
        "m" => value * 60,
        "h" => value * 3600,
        "d" => value * 86400,
        _ => return None,
    };

    if seconds > MAX_DURATION_SECONDS {
        return None;
    }

    Some(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minutes() {
        assert_eq!(parse_duration("30m"), Some(1800));
        assert_eq!(parse_duration("1m"), Some(60));
    }

    #[test]
    fn test_parse_hours() {
        assert_eq!(parse_duration("1h"), Some(3600));
        assert_eq!(parse_duration("24h"), Some(86400));
    }

    #[test]
    fn test_parse_days() {
        assert_eq!(parse_duration("7d"), Some(604800));
        assert_eq!(parse_duration("1d"), Some(86400));
    }

    #[test]
    fn test_invalid_formats() {
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("10s"), None);
        assert_eq!(parse_duration("0h"), None);
        assert_eq!(parse_duration("-1h"), None);
        assert_eq!(parse_duration("h"), None);
    }

    #[test]
    fn test_exceeds_max_duration() {
        assert_eq!(parse_duration("8d"), None);
        assert_eq!(parse_duration("169h"), None);
        assert_eq!(parse_duration("799d"), None);
    }
}
