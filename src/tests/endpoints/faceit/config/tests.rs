use super::*;

#[test]
fn channel_names_are_normalized_and_nicknames_preserve_case() {
    let config: Config = toml::from_str("channels = { Streamer = 'Donk666' }").unwrap();
    assert_eq!(config.channels["streamer"].to_string(), "Donk666");
}

#[test]
fn invalid_names_and_case_duplicate_channels_are_rejected() {
    for config in [
        "channels = { 'bad name' = 'donk666' }",
        "channels = { streamer = 'two names' }",
        "channels = { streamer = '' }",
        "channels = { Streamer = 'owner', streamer = 'other' }",
    ] {
        assert!(toml::from_str::<Config>(config).is_err());
    }
}
