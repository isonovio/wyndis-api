use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de::Error};

use super::sites::faceit::FaceitNickname;
use super::sites::twitch::TwitchChannelName;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default, deserialize_with = "channels")]
    pub channels: HashMap<TwitchChannelName, FaceitNickname>,
}

fn channels<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<HashMap<TwitchChannelName, FaceitNickname>, D::Error> {
    let raw = HashMap::<String, FaceitNickname>::deserialize(deserializer)?;
    let mut channels = HashMap::new();
    for (twitch, faceit) in raw {
        let twitch = TwitchChannelName::try_from(twitch).map_err(D::Error::custom)?;
        if let Some(dup) = channels.insert(twitch, faceit) {
            return Err(D::Error::custom(format!("duplicate Twitch channel: {dup}")));
        }
    }
    Ok(channels)
}

#[cfg(test)]
mod tests {
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
}
