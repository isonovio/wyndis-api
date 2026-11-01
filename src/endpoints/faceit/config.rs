use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de::Error};

use crate::sites::faceit::FaceitNickname;
use crate::sites::twitch::TwitchChannelName;

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
#[path = "../../tests/endpoints/faceit/config/tests.rs"]
mod tests;
