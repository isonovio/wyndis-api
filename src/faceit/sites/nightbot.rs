use axum::http::HeaderMap;
use serde::Deserialize;

use crate::error::Error;

use super::twitch::TwitchChannelName;

#[derive(Clone)]
pub enum TwitchChannel {
    Name(TwitchChannelName),
    Missing,
    Invalid,
}

impl TwitchChannel {
    pub fn name(&self) -> Result<&TwitchChannelName, Error> {
        match self {
            Self::Name(name) => Ok(name),
            Self::Missing => Err(Error::bad_request("Missing Nightbot channel header")),
            Self::Invalid => Err(Error::bad_request("Invalid Nightbot channel header")),
        }
    }
}

#[derive(Deserialize)]
struct ChannelHeader {
    provider: String,
    name: String,
}

pub fn parse_channel(headers: &HeaderMap) -> TwitchChannel {
    let Some(header) = headers.get("nightbot-channel") else {
        return TwitchChannel::Missing;
    };
    let channel = header
        .to_str()
        .ok()
        .and_then(|value| serde_urlencoded::from_str::<ChannelHeader>(value).ok());
    match channel {
        Some(channel) => match channel.provider.as_str() {
            "twitch" => match TwitchChannelName::try_from(channel.name) {
                Ok(name) => TwitchChannel::Name(name),
                Err(_) => TwitchChannel::Invalid,
            },
            _ => TwitchChannel::Invalid,
        },
        None => TwitchChannel::Invalid,
    }
}
