use axum::http::StatusCode;

use super::*;

fn output(birthday: &str, today: &str) -> String {
    Age::from_dates(birthday.parse().unwrap(), today.parse().unwrap())
        .unwrap()
        .to_string()
}

#[test]
fn birthday_and_week_before_have_exact_chat_messages() {
    for (today, expected) in [
        ("2026-01-01", " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉"),
        ("2025-12-31", "I will be 26 tomorrow! 🥳"),
        ("2025-12-30", "I will be 26 in 2 days! 🥳"),
        ("2025-12-25", "I will be 26 in 7 days! 🥳"),
        ("2025-12-24", "25.9781"),
    ] {
        assert_eq!(output("2000-01-01", today), expected);
    }
}

#[test]
fn numeric_age_uses_actual_year_length_and_trims_zeros() {
    for (today, expected) in [
        ("2024-07-02", "24.5"),
        ("2025-01-02", "25.0027"),
        ("2025-03-15", "25.2"),
    ] {
        assert_eq!(output("2000-01-01", today), expected);
    }
}

#[test]
fn leap_day_anniversary_is_february_28_only_in_non_leap_years() {
    for (today, expected) in [
        ("2023-02-27", "I will be 23 tomorrow! 🥳"),
        ("2023-02-28", " 🥳 I am 23 🎂 Today is my 🎈BIRTHDAY🎉"),
        ("2023-03-01", "23.0027"),
        ("2024-02-28", "I will be 24 tomorrow! 🥳"),
        ("2024-02-29", " 🥳 I am 24 🎂 Today is my 🎈BIRTHDAY🎉"),
        ("2024-03-01", "24.0027"),
    ] {
        assert_eq!(output("2000-02-29", today), expected);
    }
}

#[test]
fn timezone_is_resolved_per_request_with_utc_as_default() {
    let now = "2026-01-01T00:30:00Z".parse().unwrap();
    let mut person: Person = toml::from_str("id = 'owner'\nbirthday = 2000-01-01").unwrap();
    assert_eq!(
        Age::at(&person, now).unwrap().to_string(),
        " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉"
    );
    person.timezone = Some(chrono_tz::America::New_York);
    assert_eq!(
        Age::at(&person, now).unwrap().to_string(),
        "I will be 26 tomorrow! 🥳"
    );
    person.timezone = Some(chrono_tz::Asia::Tokyo);
    assert_eq!(
        Age::at(&person, now).unwrap().to_string(),
        " 🥳 I am 26 🎂 Today is my 🎈BIRTHDAY🎉"
    );
}

#[test]
fn dst_changes_do_not_change_calendar_day_count() {
    let person: Person =
        toml::from_str("id = 'owner'\nbirthday = 2000-03-09\ntimezone = 'America/New_York'")
            .unwrap();
    for now in ["2026-03-08T06:30:00Z", "2026-03-08T07:30:00Z"] {
        assert_eq!(
            Age::at(&person, now.parse().unwrap()).unwrap().to_string(),
            "I will be 26 tomorrow! 🥳"
        );
    }
}

#[test]
fn missing_and_future_birthdays_are_errors() {
    let now = "2026-01-01T00:30:00Z".parse().unwrap();
    let mut person: Person = toml::from_str("id = 'owner'").unwrap();
    let error = Age::at(&person, now).err().unwrap();
    assert_eq!(error.status, StatusCode::NOT_FOUND);
    assert_eq!(error.message, "Missing birthday");
    person.birthday = Some("2026-01-01".parse().unwrap());
    person.timezone = Some(chrono_tz::America::New_York);
    let error = Age::at(&person, now).err().unwrap();
    assert_eq!(error.status, StatusCode::BAD_REQUEST);
    assert_eq!(error.message, "Birthday is in the future");
    person.timezone = None;
    assert_eq!(
        Age::at(&person, now).unwrap().to_string(),
        " 🥳 I am 0 🎂 Today is my 🎈BIRTHDAY🎉"
    );
}
