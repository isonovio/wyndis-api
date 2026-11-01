use super::*;

#[test]
fn configuration_rejects_old_format_and_unknown_fields() {
    for extra in [
        "[faceit.channels]\nstreamer = 'owner'",
        "api_key = 'secret'",
    ] {
        assert!(
            toml::from_str::<Config>(&format!("bind_addr = '127.0.0.1:3000'\n{extra}")).is_err()
        );
    }
}

#[test]
fn endpoint_rate_limits_have_independent_defaults_and_overrides() {
    let config: Config = toml::from_str("bind_addr = '127.0.0.1:3000'").unwrap();
    assert_eq!(config.rate_limit.elo.requests_per_minute.get(), 2);
    assert_eq!(config.rate_limit.age.requests_per_minute.get(), 60);
    assert_eq!(config.rate_limit.elo.burst.get(), 1);
    assert_eq!(config.rate_limit.age.burst.get(), 1);
    let config: Config =
        toml::from_str("bind_addr = '127.0.0.1:3000'\n[rate_limit.age]\nrequests_per_minute = 120")
            .unwrap();
    assert_eq!(config.rate_limit.elo.requests_per_minute.get(), 2);
    assert_eq!(config.rate_limit.age.requests_per_minute.get(), 120);
    for limits in [
        "[rate_limit]\nrequests_per_minute = 60",
        "[rate_limit.elo]\nrequests_per_minute = 0",
        "[rate_limit.age]\nburst = 0",
    ] {
        assert!(
            toml::from_str::<Config>(&format!("bind_addr = '127.0.0.1:3000'\n{limits}")).is_err()
        );
    }
}
