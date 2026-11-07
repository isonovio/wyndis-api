use std::time::Duration;

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
        Self {
            cache: Cache::builder().max_capacity(256).time_to_live(ttl).build(),
        }
    }
}

impl<S, K: RequestKey, V: CachedValue> Layer<S> for CacheLayer<K, V> {
    type Service = CacheService<S, K, V>;

    fn layer(&self, inner: S) -> Self::Service {
        CacheService::new(inner, self.cache.clone())
    }
}
