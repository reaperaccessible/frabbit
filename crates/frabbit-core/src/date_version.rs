//! Helpers that turn a Unix timestamp (or an ISO-8601 string) into a
//! `YYYY.MM.DD` version literal. Used by date-based version-comparison
//! detectors / providers (today: CSI).
//!
//! No external dependencies — the timestamp → year/month/day conversion
//! is implemented with Howard Hinnant's `civil_from_days` algorithm,
//! which works for any date in the proleptic Gregorian calendar.

/// Convert a Unix timestamp (seconds since 1970-01-01 UTC) to a
/// `YYYY.MM.DD` literal. Negative timestamps (pre-1970) are clamped to
/// the epoch.
pub fn unix_timestamp_to_version(secs: i64) -> String {
    let (year, month, day) = civil_from_unix_timestamp(secs);
    format!("{year:04}.{month:02}.{day:02}")
}

/// Extract `YYYY-MM-DD` from an ISO-8601 timestamp like
/// `2026-05-30T12:34:56Z` and return `YYYY.MM.DD`. Returns `None` when
/// the input doesn't begin with a valid `YYYY-MM-DD` prefix.
pub fn parse_iso8601_to_version(iso: &str) -> Option<String> {
    let trimmed = iso.trim();
    if trimmed.len() < 10 {
        return None;
    }
    let date = &trimmed[..10];
    let bytes = date.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: u32 = date[..4].parse().ok()?;
    let month: u32 = date[5..7].parse().ok()?;
    let day: u32 = date[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(format!("{year:04}.{month:02}.{day:02}"))
}

/// Howard Hinnant's `civil_from_days` algorithm, adapted for Unix
/// timestamps. Returns `(year, month, day)` for the calendar date in
/// UTC corresponding to `secs`. Negative inputs are treated as `0`.
fn civil_from_unix_timestamp(secs: i64) -> (i32, u32, u32) {
    let secs = secs.max(0);
    let days = secs / 86_400;
    // Shift epoch from 1970-01-01 to 0000-03-01 (Hinnant's "z" origin).
    let z = days + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::{parse_iso8601_to_version, unix_timestamp_to_version};

    #[test]
    fn epoch_renders_as_1970_01_01() {
        assert_eq!(unix_timestamp_to_version(0), "1970.01.01");
    }

    #[test]
    fn known_timestamps_round_trip_to_known_dates() {
        // 2026-05-22 00:00:00 UTC = 1779408000
        assert_eq!(unix_timestamp_to_version(1_779_408_000), "2026.05.22");
        // 2000-01-01 00:00:00 UTC = 946684800
        assert_eq!(unix_timestamp_to_version(946_684_800), "2000.01.01");
        // 2024-02-29 (leap day) 00:00:00 UTC = 1709164800
        assert_eq!(unix_timestamp_to_version(1_709_164_800), "2024.02.29");
    }

    #[test]
    fn negative_timestamps_clamp_to_epoch() {
        assert_eq!(unix_timestamp_to_version(-1), "1970.01.01");
    }

    #[test]
    fn parses_iso8601_prefix() {
        assert_eq!(
            parse_iso8601_to_version("2026-05-30T12:34:56Z").as_deref(),
            Some("2026.05.30")
        );
        assert_eq!(
            parse_iso8601_to_version("2026-05-30").as_deref(),
            Some("2026.05.30")
        );
        assert_eq!(
            parse_iso8601_to_version("  2026-05-30T00:00:00+00:00  ").as_deref(),
            Some("2026.05.30")
        );
    }

    #[test]
    fn rejects_malformed_iso8601() {
        assert!(parse_iso8601_to_version("2026/05/30").is_none());
        assert!(parse_iso8601_to_version("not-a-date").is_none());
        assert!(parse_iso8601_to_version("").is_none());
        assert!(parse_iso8601_to_version("2026-13-30").is_none());
        assert!(parse_iso8601_to_version("2026-00-30").is_none());
        assert!(parse_iso8601_to_version("2026-05-32").is_none());
    }
}
