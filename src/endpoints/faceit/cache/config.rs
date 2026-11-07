use std::{num::NonZeroU64, time::Duration};

use serde::{Deserialize, Deserializer};

// Moka rejects TTLs longer than 1000 years.
const MAX_CACHE_TTL: Duration = Duration::from_secs(1000 * 365 * 24 * 60 * 60);

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "ttl_secs", deserialize_with = "seconds")]
    pub ttl: Duration,
    pub max_entries: NonZeroU64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ttl: Duration::from_secs(30),
            max_entries: NonZeroU64::new(256).unwrap(),
        }
    }
}

fn seconds<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
    let ttl = Duration::from_secs(u64::deserialize(deserializer)?);
    match ttl <= MAX_CACHE_TTL {
        true => Ok(ttl),
        false => Err(serde::de::Error::custom("Cache TTL exceeds Moka's maximum")),
    }
}
