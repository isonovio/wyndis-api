use std::{collections::HashMap, sync::Arc};

use super::config;
use super::sites::faceit::{Client, FaceitNickname};
use super::sites::twitch::TwitchChannelName;

pub type Channels = Arc<HashMap<TwitchChannelName, FaceitNickname>>;

#[derive(Clone)]
pub struct State {
    pub(super) client: Client,
    pub(super) channels: Channels,
}

impl State {
    pub fn new(config: config::Config, api_key: &str) -> anyhow::Result<Self> {
        Ok(Self::with_client(Client::new(api_key)?, config.channels))
    }

    pub fn with_client(
        client: Client,
        channels: HashMap<TwitchChannelName, FaceitNickname>,
    ) -> Self {
        let channels: Channels = Arc::new(channels);
        Self { client, channels }
    }
}
