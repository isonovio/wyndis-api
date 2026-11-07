use serde::Deserialize;

use crate::middlewares::rate_limit;
use crate::sites::faceit::client;

use super::cache;

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rate_limit: rate_limit::Config,
    #[serde(default)]
    pub client: client::Config,
    #[serde(default)]
    pub cache: cache::Config,
}
