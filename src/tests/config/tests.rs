use super::*;

const LIMITS: &str = "[rate_limit.global]\nrequests_per_minute = 600\nburst = 1\n[rate_limit.faceit]\nrequests_per_minute = 2\nburst = 1";

#[test]
fn configuration_rejects_old_format_and_unknown_fields() {
    for extra in [
        "[faceit.channels]\nstreamer = 'owner'",
        "[unknown]\napi_key = 'secret'",
    ] {
        assert!(
            toml::from_str::<Config>(&format!("bind_addr = '127.0.0.1:3000'\n{LIMITS}\n{extra}"))
                .is_err()
        );
    }
}

#[test]
fn global_and_faceit_limits_are_required_and_validated() {
    let input = format!("bind_addr = '127.0.0.1:3000'\n{LIMITS}");
    let config: Config = toml::from_str(&input).unwrap();
    assert_eq!(config.rate_limit.global.requests_per_minute.get(), 600);
    assert_eq!(config.rate_limit.faceit.requests_per_minute.get(), 2);
    assert_eq!(config.rate_limit.global.burst.get(), 1);
    assert_eq!(config.rate_limit.faceit.burst.get(), 1);
    assert!(toml::from_str::<Config>("bind_addr = '127.0.0.1:3000'").is_err());
    for invalid in [
        input.replace("requests_per_minute = 600", "requests_per_minute = 0"),
        input.replace("requests_per_minute = 2", "requests_per_minute = 0"),
        input.replace("burst = 1", "burst = 0"),
        input.replace("rate_limit.global", "rate_limit.age"),
        input.replace("rate_limit.faceit", "rate_limit.elo"),
    ] {
        assert!(toml::from_str::<Config>(&invalid).is_err());
    }
}
