use std::path::Path;

use wyndis_api::config::Config;

#[test]
fn persons_load_from_repository_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml");
    let config = Config::load(path).unwrap();
    for (channel, nickname) in [("qiyarah", "Qiyarah"), ("nayomy_cs", "Nayomy")] {
        let channel = channel.to_owned().try_into().unwrap();
        let person = config.persons.from_twitch(&channel).unwrap();
        assert_eq!(
            person.faceit_nickname.as_ref().unwrap().to_string(),
            nickname
        );
    }
}
