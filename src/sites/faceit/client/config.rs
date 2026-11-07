use std::{num::NonZeroU64, time::Duration};

use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(try_from = "RawConfig")]
pub struct Config {
    request_timeout: Duration,
    settle_timeout: Duration,
}

impl Config {
    pub fn request_timeout(self) -> Duration {
        self.request_timeout
    }

    pub fn settle_timeout(self) -> Duration {
        self.settle_timeout
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(20),
            settle_timeout: Duration::from_millis(3000),
        }
    }
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawConfig {
    request_timeout_secs: NonZeroU64,
    settle_timeout_ms: u64,
}

impl Default for RawConfig {
    fn default() -> Self {
        let Config {
            request_timeout,
            settle_timeout,
        } = Config::default();
        Self {
            request_timeout_secs: NonZeroU64::new(request_timeout.as_secs()).expect("nonzero"),
            settle_timeout_ms: settle_timeout.as_millis() as u64,
        }
    }
}

impl TryFrom<RawConfig> for Config {
    type Error = &'static str;

    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        let request_timeout = Duration::from_secs(raw.request_timeout_secs.get());
        let settle_timeout = Duration::from_millis(raw.settle_timeout_ms);
        if std::time::Instant::now()
            .checked_add(request_timeout)
            .is_none()
        {
            return Err("FACEIT request timeout is out of range");
        }
        if settle_timeout > request_timeout {
            return Err("FACEIT settle timeout exceeds request timeout");
        }
        Ok(Self {
            request_timeout,
            settle_timeout,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/sites/faceit/client/config/tests.rs"]
mod tests;
