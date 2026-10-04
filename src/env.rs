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
mod tests {
    use super::*;

    #[test]
    fn environment_values_are_validated() {
        assert!(Env::from_values(Some("wyndis_api=invalid")).is_err());
        let env = Env::from_values(None).unwrap();
        assert_eq!(env.log_filter.to_string(), "wyndis_api=info");
        let env = Env::from_values(Some("debug")).unwrap();
        assert_eq!(env.log_filter.to_string(), "debug");
    }
}
