use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;

use crate::TARGET;

pub struct Env {
    pub log_filter: EnvFilter,
}

impl Env {
    pub fn load() -> Result<Self> {
        let log_filter = match std::env::var("RUST_LOG") {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(error) => return Err(error).context("RUST_LOG must be valid Unicode"),
        };
        Self::from_values(log_filter.as_deref())
    }

    fn from_values(log_filter: Option<&str>) -> Result<Self> {
        let log_filter = EnvFilter::try_new(log_filter.unwrap_or(&format!("{TARGET}=info")))
            .context("invalid RUST_LOG filter")?;
        Ok(Self { log_filter })
    }
}

#[cfg(test)]
#[path = "tests/env/tests.rs"]
mod tests;
