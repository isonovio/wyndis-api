use super::*;

use crate::config::Config;

fn config(persons: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(&format!(
        "bind_addr = '127.0.0.1:3000'\n[rate_limit.global]\nrequests_per_minute = 600\nburst = 100\n[rate_limit.faceit]\nrequests_per_minute = 2\nburst = 1\n{persons}"
    ))
}

#[test]
fn fields_are_optional_and_persons_without_channels_are_retained() {
    let config = config("[[persons]]\nid = 'internal'").unwrap();
    let person = &config.persons.by_id["internal"];
    assert!(person.twitch_channel.is_none());
    assert!(person.faceit_nickname.is_none());
    assert!(person.birthday.is_none());
    assert!(person.timezone.is_none());
    assert!(config.persons.by_twitch.is_empty());
}

#[test]
fn dates_timezones_and_site_names_are_loaded() {
    let config = config("[[persons]]\nid = 'owner'\ntwitch_channel = 'Streamer'\nfaceit_nickname = 'Donk666'\nbirthday = 2000-02-29\ntimezone = 'America/New_York'").unwrap();
    let person = config
        .persons
        .from_twitch(&"streamer".to_owned().try_into().unwrap())
        .unwrap();
    assert_eq!(person.id, "owner");
    assert_eq!(person.twitch_channel.as_deref(), Some("streamer"));
    assert_eq!(
        person.faceit_nickname.as_ref().unwrap().to_string(),
        "Donk666"
    );
    assert_eq!(person.birthday, Date::new(2000, 2, 29).ok());
    assert_eq!(
        person.timezone,
        Some(TimeZone::get("America/New_York").unwrap())
    );
}

#[test]
fn invalid_person_configuration_is_rejected() {
    for persons in [
        "[[persons]]\nid = ''",
        "[[persons]]\nid = '  '",
        "[[persons]]\ntwitch_channel = 'streamer'",
        "[[persons]]\nid = 'a'\n[[persons]]\nid = 'a'",
        "[[persons]]\nid = 'a'\ntwitch_channel = 'Streamer'\n[[persons]]\nid = 'b'\ntwitch_channel = 'streamer'",
        "[[persons]]\nid = 'a'\ntwitch_channel = 'bad name'",
        "[[persons]]\nid = 'a'\nfaceit_nickname = 'two names'",
        "[[persons]]\nid = 'a'\nfaceit_nickname = ''",
        "[[persons]]\nid = 'a'\nbirthday = 2001-02-29",
        "[[persons]]\nid = 'a'\nbirthday = 2000-01-01T12:00:00Z",
        "[[persons]]\nid = 'a'\ntimezone = 'Invalid/Zone'",
        "[[persons]]\nid = 'a'\nunknown = 'value'",
    ] {
        assert!(config(persons).is_err(), "accepted {persons}");
    }
}

#[test]
fn birthday_and_optional_timezone_are_stored_separately() {
    for (fields, timezone) in [
        ("birthday = 2000-01-31", None),
        (
            "timezone = 'America/New_York'\nbirthday = 2000-01-31",
            Some(TimeZone::get("America/New_York").unwrap()),
        ),
        (
            "birthday = 2000-01-31\ntimezone = 'UTC'",
            Some(TimeZone::UTC),
        ),
    ] {
        let person: Person = toml::from_str(&format!("id = 'owner'\n{fields}")).unwrap();
        assert_eq!(person.birthday, Date::new(2000, 1, 31).ok());
        assert_eq!(person.timezone, timezone);
    }
}

#[test]
fn standalone_person_deserialization_validates_fields() {
    for fields in [
        "id = ''",
        "id = 'owner'\nfaceit_nickname = 'two names'",
        "id = 'owner'\ntwitch_channel = 'bad name'",
        "id = 'owner'\ntimezone = 'Invalid/Zone'",
    ] {
        assert!(
            toml::from_str::<Person>(fields).is_err(),
            "accepted {fields}"
        );
    }
}

#[test]
fn persons_deserializes_directly_and_checks_uniqueness() {
    let persons: Persons = serde_json::from_value(serde_json::json!([
        {"id": "owner", "twitch_channel": "Streamer", "faceit_nickname": "Donk666"}
    ]))
    .unwrap();
    let channel = "streamer".to_owned().try_into().unwrap();
    assert_eq!(
        persons
            .from_twitch(&channel)
            .unwrap()
            .faceit_nickname
            .as_ref()
            .unwrap()
            .to_string(),
        "Donk666"
    );
    for value in [
        serde_json::json!([{"id": "owner"}, {"id": "owner"}]),
        serde_json::json!([{"id": "a", "twitch_channel": "Streamer"}, {"id": "b", "twitch_channel": "streamer"}]),
        serde_json::json!([{"id": "a", "faceit_nickname": "two names"}]),
    ] {
        assert!(serde_json::from_value::<Persons>(value).is_err());
    }
}

#[test]
fn person_ids_match_the_slug_pattern() {
    for id in ["owner", "person-123", "0", "-", "-owner--"] {
        let person: Person = toml::from_str(&format!("id = '{id}'")).unwrap();
        assert_eq!(person.id, id);
    }
    for id in [
        "",
        "Owner",
        "owner_name",
        "owner.name",
        "two names",
        " owner",
        "owner ",
        "é",
        "１２",
    ] {
        assert!(
            config(&format!("[[persons]]\nid = '{id}'")).is_err(),
            "accepted {id:?}"
        );
    }
}
