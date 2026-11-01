use super::*;

fn utc(value: &str) -> Zoned {
    value
        .parse::<jiff::Timestamp>()
        .unwrap()
        .to_zoned(jiff::tz::TimeZone::UTC)
}

#[test]
fn calendar_offset_borrows_time_and_clamps_month_ends() {
    for (start, end, expected) in [
        ("2025-01-31T12:00:00Z", "2025-02-28T12:00:00Z", "1 month"),
        (
            "2025-01-31T12:00:00Z",
            "2025-02-28T11:59:59Z",
            "27 days 23 hours 59 minutes 59 seconds",
        ),
        (
            "2000-02-29T00:00:00Z",
            "2023-03-28T00:00:00Z",
            "23 years 1 month",
        ),
        (
            "2025-01-01T12:30:15Z",
            "2026-02-02T13:31:16Z",
            "1 year 1 month 1 day 1 hour 1 minute 1 second",
        ),
    ] {
        assert_eq!(
            Offset::by_years(&utc(start), &utc(end))
                .unwrap()
                .to_string(),
            expected
        );
    }
}

#[test]
fn remaining_time_uses_days_and_rounds_fractional_seconds_up() {
    assert_eq!(
        Offset::by_days(&utc("2025-12-01T00:00:00Z"), &utc("2026-01-01T00:00:00Z"))
            .unwrap()
            .to_string(),
        "31 days"
    );
    assert_eq!(
        Offset::by_days(&utc("2025-12-31T23:59:59.9Z"), &utc("2026-01-01T00:00:00Z"))
            .unwrap()
            .to_string(),
        "1 second"
    );
}

#[test]
fn reversed_intervals_return_none() {
    let start = utc("2026-01-01T00:00:00Z");
    let end = utc("2025-12-31T23:59:59Z");
    assert!(Offset::by_years(&start, &end).is_none());
    assert!(Offset::by_days(&start, &end).is_none());
}
