use super::*;

const CONFIG: &str = "bind_addr = '127.0.0.1:3000'\n[faceit]\nchannels = { streamer = 'donk666' }";

#[test]
fn configuration_rejects_embedded_api_keys() {
    let contents = CONFIG.replace("[faceit]", "[faceit]\napi_key = \"secret\"");
    assert!(toml::from_str::<Config>(&contents).is_err());
}

#[test]
fn config_composes_faceit_settings() {
    let config: Config = toml::from_str(CONFIG).unwrap();
    assert_eq!(config.bind_addr.port(), 3000);
    assert_eq!(config.faceit.channels["streamer"].to_string(), "donk666");
}
