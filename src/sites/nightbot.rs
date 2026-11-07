use axum::http::HeaderMap;
use serde::Deserialize;

use crate::error::Error;

use super::twitch::TwitchChannelName;

#[derive(Deserialize)]
struct ChannelHeader {
    provider: String,
    name: String,
}

pub fn parse_channel(headers: &HeaderMap) -> Result<TwitchChannelName, Error> {
    let header = headers
        .get("nightbot-channel")
        .ok_or_else(|| Error::bad_request("Missing Nightbot channel header"))?;
    let channel = (header.to_str().ok())
        .and_then(|value| serde_urlencoded::from_str::<ChannelHeader>(value).ok())
        .filter(|channel| channel.provider == "twitch")
        .ok_or_else(|| Error::bad_request("Invalid Nightbot channel header"))?;
    TwitchChannelName::try_from(channel.name)
        .map_err(|_| Error::bad_request("Invalid Nightbot channel header"))
}
