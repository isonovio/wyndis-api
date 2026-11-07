use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::endpoints::faceit;
use crate::middlewares::rate_limit;
use crate::models::Persons;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub global: GlobalConfig,
    pub faceit: faceit::config::Config,
    #[serde(default)]
    pub persons: Persons,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GlobalConfig {
    pub rate_limit: rate_limit::Config,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_file(path)
    }

    fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = fs::read_to_string(path.as_ref())
            .with_context(|| format!("cannot read config {}", path.as_ref().display()))?;
        toml::from_str::<Self>(&contents).context("invalid configuration")
    }
}

#[cfg(test)]
#[path = "tests/config/tests.rs"]
mod tests;
