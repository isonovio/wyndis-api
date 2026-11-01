use std::{borrow::Borrow, fmt};

use serde::Deserialize;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub struct TwitchChannelName(String);

impl TryFrom<String> for TwitchChannelName {
    type Error = &'static str;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        match !name.is_empty() && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_') {
            true => Ok(Self(name.to_ascii_lowercase())),
            false => Err("expected a Twitch channel name"),
        }
    }
}

impl Borrow<str> for TwitchChannelName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TwitchChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
