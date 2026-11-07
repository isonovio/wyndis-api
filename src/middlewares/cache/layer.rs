use std::{num::NonZeroU64, time::Duration};

use moka::future::Cache;
use tower::Layer;

use super::{
    kv::{CachedValue, RequestKey},
    service::CacheService,
};

#[derive(Clone)]
pub struct CacheLayer<K, V> {
    cache: Cache<K, V>,
}

impl<K: RequestKey, V: CachedValue> CacheLayer<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self::with_capacity(ttl, NonZeroU64::new(256).unwrap())
    }

    pub fn with_capacity(ttl: Duration, max_entries: NonZeroU64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_entries.get())
                .time_to_live(ttl)
                .build(),
        }
    }
}

impl<S, K: RequestKey, V: CachedValue> Layer<S> for CacheLayer<K, V> {
    type Service = CacheService<S, K, V>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheService::new(inner, self.cache.clone())
    }
}
