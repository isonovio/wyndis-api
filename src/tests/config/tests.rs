use super::*;

const LIMITS: &str = "[global.rate_limit]\nrequests_per_minute = 600\nburst = 1\n[faceit.rate_limit]\nrequests_per_minute = 2\nburst = 1";

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
    assert_eq!(config.global.rate_limit.requests_per_minute.get(), 600);
    assert_eq!(config.faceit.rate_limit.requests_per_minute.get(), 2);
    assert_eq!(config.global.rate_limit.burst.get(), 1);
    assert_eq!(config.faceit.rate_limit.burst.get(), 1);
    assert!(toml::from_str::<Config>("bind_addr = '127.0.0.1:3000'").is_err());
    for invalid in [
        input.replace("requests_per_minute = 600", "requests_per_minute = 0"),
        input.replace("requests_per_minute = 2", "requests_per_minute = 0"),
        input.replace("burst = 1", "burst = 0"),
        input.replace("global.rate_limit", "rate_limit.age"),
        input.replace("faceit.rate_limit", "rate_limit.elo"),
        input.replace("global.rate_limit", "rate_limit.global"),
        input.replace("faceit.rate_limit", "rate_limit.faceit"),
        input.replace(
            "[faceit.rate_limit]\nrequests_per_minute = 2\nburst = 1",
            "[faceit.cache]\nttl_secs = 30",
        ),
        input.replace(
            "[global.rate_limit]\nrequests_per_minute = 600\nburst = 1",
            "[global]",
        ),
    ] {
        assert!(toml::from_str::<Config>(&invalid).is_err());
    }
}

#[test]
fn faceit_settings_load_with_defaults_and_explicit_overrides() {
    let base = format!("bind_addr = '127.0.0.1:3000'\n{LIMITS}");
    let config: Config = toml::from_str(&base).unwrap();
    assert_eq!(config.faceit.cache.ttl, std::time::Duration::from_secs(30));
    assert_eq!(config.faceit.cache.max_entries.get(), 256);
    for ttl in [0, 15] {
        let config: Config = toml::from_str(&format!("{base}\n[faceit.cache]\nttl_secs = {ttl}\nmax_entries = 12\n[faceit.client]\nrequest_timeout_secs = 8\nsettle_timeout_ms = 500")).unwrap();
        assert_eq!(config.faceit.cache.ttl, std::time::Duration::from_secs(ttl));
        assert_eq!(config.faceit.cache.max_entries.get(), 12);
        assert_eq!(
            config.faceit.client.request_timeout(),
            std::time::Duration::from_secs(8)
        );
        assert_eq!(
            config.faceit.client.settle_timeout(),
            std::time::Duration::from_millis(500)
        );
    }
}

#[test]
fn invalid_faceit_cache_settings_are_rejected() {
    for fields in [
        "max_entries = 0",
        "max_entries = -1",
        "ttl_secs = -1",
        "ttl_secs = 0.5",
        "ttl_secs = 31536000001",
        "unknown = 1",
    ] {
        let input = format!("bind_addr = '127.0.0.1:3000'\n{LIMITS}\n[faceit.cache]\n{fields}");
        assert!(
            toml::from_str::<Config>(&input).is_err(),
            "accepted {fields}"
        );
    }
}
