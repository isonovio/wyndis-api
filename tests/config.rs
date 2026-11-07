use std::path::Path;

use wyndis_api::config::Config;

#[test]
fn persons_load_from_repository_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml");
    let config = Config::load(path).unwrap();
    assert_eq!(config.global.rate_limit.requests_per_minute.get(), 60000);
    assert_eq!(config.faceit.rate_limit.requests_per_minute.get(), 6);
    assert_eq!(config.faceit.cache.ttl, std::time::Duration::from_secs(30));
    assert_eq!(config.faceit.cache.max_entries.get(), 256);
    assert_eq!(
        config.faceit.client.request_timeout(),
        std::time::Duration::from_secs(20)
    );
    assert_eq!(
        config.faceit.client.settle_timeout(),
        std::time::Duration::from_millis(3000)
    );
    for (channel, nickname) in [("qiyarah", "Qiyarah"), ("nayomy_cs", "Nayomy")] {
        let channel = channel.to_owned().try_into().unwrap();
        let person = config.persons.from_twitch(&channel).unwrap();
        assert_eq!(
            person.faceit_nickname.as_ref().unwrap().to_string(),
            nickname
        );
    }
}
