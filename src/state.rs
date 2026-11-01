use std::{collections::HashMap, sync::Arc};

use crate::endpoints::faceit::config;
use crate::sites::faceit::{Client, FaceitNickname};
use crate::sites::twitch::TwitchChannelName;

pub type Channels = Arc<HashMap<TwitchChannelName, FaceitNickname>>;

#[derive(Clone)]
pub struct State {
    pub(crate) client: Client,
    pub(crate) channels: Channels,
}

impl State {
    pub fn new(config: config::Config) -> anyhow::Result<Self> {
        Ok(Self::with_client(Client::new()?, config.channels))
    }

    pub fn with_client(
        client: Client,
        channels: HashMap<TwitchChannelName, FaceitNickname>,
    ) -> Self {
        let channels: Channels = Arc::new(channels);
        Self { client, channels }
    }
}
