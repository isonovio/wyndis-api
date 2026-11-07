use super::*;

#[test]
fn defaults_and_overrides_use_typed_durations() {
    let config: Config = toml::from_str("").unwrap();
    assert_eq!(config.request_timeout(), Duration::from_secs(20));
    assert_eq!(config.settle_timeout(), Duration::from_millis(3000));
    let config: Config =
        toml::from_str("request_timeout_secs = 7\nsettle_timeout_ms = 1250").unwrap();
    assert_eq!(config.request_timeout(), Duration::from_secs(7));
    assert_eq!(config.settle_timeout(), Duration::from_millis(1250));
    let config: Config = toml::from_str("settle_timeout_ms = 0").unwrap();
    assert!(config.settle_timeout().is_zero());
}

#[test]
fn invalid_timing_budgets_are_rejected_at_deserialization() {
    for input in [
        "request_timeout_secs = 0",
        "request_timeout_secs = -1",
        "request_timeout_secs = 2",
        "request_timeout_secs = 2\nsettle_timeout_ms = 2001",
        "settle_timeout_ms = -1",
        "timeout = 20",
    ] {
        assert!(toml::from_str::<Config>(input).is_err(), "accepted {input}");
    }
    assert!(toml::from_str::<Config>("request_timeout_secs = 2\nsettle_timeout_ms = 2000").is_ok());
}
