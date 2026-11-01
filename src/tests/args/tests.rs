use super::*;

#[test]
fn config_path_defaults_or_uses_argument() {
    assert_eq!(
        Args::try_parse_from(["wyndis-api"]).unwrap().config,
        PathBuf::from("config.toml")
    );
    assert_eq!(
        Args::try_parse_from(["wyndis-api", "custom.toml"])
            .unwrap()
            .config,
        PathBuf::from("custom.toml")
    );
    assert!(Args::try_parse_from(["wyndis-api", "config.toml", "extra"]).is_err());
}
