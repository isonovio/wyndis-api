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
