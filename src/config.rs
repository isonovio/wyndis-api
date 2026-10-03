use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::faceit;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub faceit: faceit::Config,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let config = Self::from_file(path)?;
        config.faceit.validate()?;
        Ok(config)
    }

    fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .with_context(|| format!("cannot read config {}", path.as_ref().display()))?;
        toml::from_str::<Self>(&contents).context("invalid configuration")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str =
        "bind_addr = '127.0.0.1:3000'\n[faceit]\nchannels = { streamer = 'donk666' }";

    #[test]
    fn configuration_rejects_embedded_api_keys() {
        let contents = CONFIG.replace("[faceit]", "[faceit]\napi_key = \"secret\"");
        assert!(toml::from_str::<Config>(&contents).is_err());
    }

    #[test]
    fn config_composes_faceit_settings() {
        let config: Config = toml::from_str(CONFIG).unwrap();
        config.faceit.validate().unwrap();
        assert_eq!(config.bind_addr.port(), 3000);
        assert_eq!(config.faceit.channels["streamer"], "donk666");
    }
}
