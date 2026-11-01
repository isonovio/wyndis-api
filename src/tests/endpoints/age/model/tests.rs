use super::*;

fn output(birthday: &str, now: &str, timezone: Option<TimeZone>) -> String {
    Age::at(
        birthday.parse().unwrap(),
        now.parse().unwrap(),
        timezone.unwrap_or(TimeZone::UTC),
    )
    .and_then(|age| age.message())
    .unwrap()
    .to_string()
}

#[test]
fn elapsed_age_uses_calendar_units_and_omits_zeros() {
    for (birthday, now, expected) in [
        (
            "2000-01-01",
            "2026-03-04T05:06:07Z",
            "I am 26 years 2 months 3 days 5 hours 6 minutes 7 seconds old.",
        ),
        (
            "2025-01-01",
            "2026-02-02T01:01:01Z",
            "I am 1 year 1 month 1 day 1 hour 1 minute 1 second old.",
        ),
        (
            "2000-01-01",
            "2026-03-01T00:00:00Z",
            "I am 26 years 2 months old.",
        ),
        ("2026-01-01", "2026-01-02T00:00:00Z", "I am 1 day old."),
        (
            "2000-01-31",
            "2025-02-28T12:00:00Z",
            "I am 25 years 1 month 12 hours old.",
        ),
        (
            "2000-12-15",
            "2026-01-16T00:00:00Z",
            "I am 25 years 1 month 1 day old.",
        ),
    ] {
        assert_eq!(output(birthday, now, None), expected);
    }
}

#[test]
fn countdown_starts_one_calendar_month_before_midnight() {
    for (now, expected) in [
        (
            "2025-11-30T23:59:59Z",
            "I am 25 years 10 months 29 days 23 hours 59 minutes 59 seconds old.",
        ),
        (
            "2025-12-01T00:00:00Z",
            "I will be 26 years old in 31 days! 🥳",
        ),
        (
            "2025-12-30T22:58:59Z",
            "I will be 26 years old in 1 day 1 hour 1 minute 1 second! 🥳",
        ),
        (
            "2025-12-31T00:00:00Z",
            "I will be 26 years old in 1 day! 🥳",
        ),
        (
            "2025-12-31T23:59:59Z",
            "I will be 26 years old in 1 second! 🥳",
        ),
        (
            "2025-12-31T23:59:59.900Z",
            "I will be 26 years old in 1 second! 🥳",
        ),
        (
            "2026-01-01T00:00:00Z",
            " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉",
        ),
        (
            "2026-01-01T23:59:59Z",
            " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉",
        ),
        ("2026-01-02T00:00:00Z", "I am 26 years 1 day old."),
    ] {
        assert_eq!(output("2000-01-01", now, None), expected);
    }
    assert_eq!(
        output("2025-01-01", "2025-12-31T23:00:00Z", None),
        "I will be 1 year old in 1 hour! 🥳"
    );
    assert_eq!(
        output("2000-03-31", "2025-02-28T00:00:00Z", None),
        "I will be 25 years old in 31 days! 🥳"
    );
}

#[test]
fn leap_day_anniversary_is_february_28_only_in_non_leap_years() {
    for (now, expected) in [
        (
            "2023-02-27T00:00:00Z",
            "I will be 23 years old in 1 day! 🥳",
        ),
        (
            "2023-02-28T00:00:00Z",
            " 🥳 I am 23 🎂 Today is my 🎈BIRTHDAY🎉",
        ),
        ("2023-03-01T00:00:00Z", "I am 23 years 1 day old."),
        (
            "2024-02-28T00:00:00Z",
            "I will be 24 years old in 1 day! 🥳",
        ),
        (
            "2024-02-29T00:00:00Z",
            " 🥳 I am 24 🎂 Today is my 🎈BIRTHDAY🎉",
        ),
        ("2024-03-01T00:00:00Z", "I am 24 years 1 day old."),
    ] {
        assert_eq!(output("2000-02-29", now, None), expected);
    }
}

#[test]
fn timezone_is_resolved_per_request_with_utc_as_default() {
    let now = "2026-01-01T00:30:00Z";
    assert_eq!(
        output("2000-01-01", now, None),
        " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉"
    );
    assert_eq!(
        output(
            "2000-01-01",
            now,
            Some(TimeZone::get("America/New_York").unwrap())
        ),
        "I will be 26 years old in 4 hours 30 minutes! 🥳"
    );
    assert_eq!(
        output(
            "2000-01-01",
            now,
            Some(TimeZone::get("Asia/Tokyo").unwrap())
        ),
        " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉"
    );
    assert_eq!(
        output(
            "2000-01-01",
            "2026-03-02T00:30:00Z",
            Some(TimeZone::get("America/New_York").unwrap())
        ),
        "I am 26 years 2 months 19 hours 30 minutes old."
    );
}

#[test]
fn countdown_uses_actual_time_across_dst_changes() {
    for (birthday, now, expected) in [
        (
            "2000-03-09",
            "2026-03-08T05:00:00Z",
            "I will be 26 years old in 23 hours! 🥳",
        ),
        (
            "2000-03-09",
            "2026-03-08T06:30:00Z",
            "I will be 26 years old in 21 hours 30 minutes! 🥳",
        ),
        (
            "2000-03-09",
            "2026-03-08T07:30:00Z",
            "I will be 26 years old in 20 hours 30 minutes! 🥳",
        ),
        (
            "2000-11-02",
            "2026-11-01T04:00:00Z",
            "I will be 26 years old in 1 day 1 hour! 🥳",
        ),
    ] {
        assert_eq!(
            output(
                birthday,
                now,
                Some(TimeZone::get("America/New_York").unwrap())
            ),
            expected
        );
    }
}

#[test]
fn future_birthdays_return_none_in_the_selected_timezone() {
    let birthday = "2026-01-01".parse().unwrap();
    let now = "2026-01-01T00:30:00Z".parse().unwrap();
    assert!(Age::at(birthday, now, TimeZone::get("America/New_York").unwrap()).is_none());
    assert_eq!(
        Age::at(birthday, now, TimeZone::UTC)
            .and_then(|age| age.message())
            .unwrap()
            .to_string(),
        " 🥳 I am 0 🎂 Today is my 🎈BIRTHDAY🎉"
    );
}

#[test]
fn nonexistent_midnight_and_unrepresentable_anniversary_return_none() {
    assert!(
        Age::at(
            "2018-11-04".parse().unwrap(),
            "2026-01-01T00:00:00Z".parse().unwrap(),
            TimeZone::get("America/Sao_Paulo").unwrap()
        )
        .is_none()
    );
    let now = Timestamp::MAX;
    assert!(
        Age::at("2000-01-01".parse().unwrap(), now, TimeZone::UTC)
            .and_then(|age| age.message())
            .is_none()
    );
}

#[test]
fn birth_and_now_use_the_selected_timezone() {
    let age = Age::at(
        "2000-01-01".parse().unwrap(),
        "2026-03-01T12:30:00Z".parse().unwrap(),
        TimeZone::get("America/New_York").unwrap(),
    )
    .unwrap();
    assert_eq!(
        age.birth.time_zone().clone(),
        TimeZone::get("America/New_York").unwrap()
    );
    assert_eq!(
        age.now.time_zone().clone(),
        TimeZone::get("America/New_York").unwrap()
    );
    assert_eq!(
        age.birth.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        "2000-01-01T00:00:00-05:00"
    );
    assert_eq!(
        age.now.strftime("%Y-%m-%dT%H:%M:%S%:z").to_string(),
        "2026-03-01T07:30:00-05:00"
    );
}

#[test]
fn ambiguous_midnight_uses_the_earliest_occurrence() {
    let birth = midnight(
        "2020-11-01".parse().unwrap(),
        TimeZone::get("America/Havana").unwrap(),
    )
    .unwrap();
    assert_eq!(
        birth.timestamp(),
        "2020-11-01T04:00:00Z".parse::<Timestamp>().unwrap()
    );
}
