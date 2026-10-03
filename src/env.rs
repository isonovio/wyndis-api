use anyhow::{Context, Result, ensure};
use tracing_subscriber::EnvFilter;

use crate::TARGET;

pub struct Env {
    pub faceit_api_key: String,
    pub log_filter: EnvFilter,
}

impl Env {
    pub fn load() -> Result<Self> {
        let api_key = std::env::var("FACEIT_API_KEY").context("FACEIT_API_KEY is required")?;
        let log_filter = match std::env::var("RUST_LOG") {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(error) => return Err(error).context("RUST_LOG must be valid Unicode"),
        };
        Self::from_values(api_key, log_filter.as_deref())
    }

    fn from_values(faceit_api_key: String, log_filter: Option<&str>) -> Result<Self> {
        ensure!(
            !faceit_api_key.trim().is_empty(),
            "FACEIT_API_KEY must not be empty"
        );
        let log_filter = EnvFilter::try_new(log_filter.unwrap_or(&format!("{TARGET}=info")))
            .context("invalid RUST_LOG filter")?;
        Ok(Self {
            faceit_api_key,
            log_filter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_values_are_validated() {
        assert!(Env::from_values(" ".into(), None).is_err());
        assert!(Env::from_values("test-key".into(), Some("wyndis_api=invalid")).is_err());
        let env = Env::from_values("test-key".into(), None).unwrap();
        assert_eq!(env.log_filter.to_string(), "wyndis_api=info");
        let env = Env::from_values("test-key".into(), Some("debug")).unwrap();
        assert_eq!(env.log_filter.to_string(), "debug");
    }
}
