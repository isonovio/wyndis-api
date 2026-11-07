use std::{
    convert::Infallible,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};
use moka::future::Cache;
use tower::{Service, ServiceExt};

use super::kv::{CachedValue, RequestKey};

#[derive(Clone)]
pub struct CacheService<S, K, V> {
    inner: S,
    cache: Cache<K, V>,
}

impl<S, K, V> CacheService<S, K, V> {
    pub(super) fn new(inner: S, cache: Cache<K, V>) -> Self {
        Self { inner, cache }
    }
}

impl<S, K, V> Service<Request> for CacheService<S, K, V>
where
    S: Service<Request, Response = Response, Error = Infallible> + Clone + Send + 'static,
    S::Future: Send,
    K: RequestKey,
    V: CachedValue,
{
    type Response = Response;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Response, Infallible>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let inner = self.inner.clone();
        let cache = self.cache.clone();
        Box::pin(async move {
            let Some(key) = K::from_request(&request) else {
                return inner.oneshot(request).await;
            };
            let result = cache
                .try_get_with(key, async move {
                    let response = match inner.oneshot(request).await {
                        Ok(response) => response,
                        Err(error) => match error {},
                    };
                    V::from_response(response).await
                })
                .await;
            Ok(match result {
                Ok(value) => value.into_response(),
                Err(response) => response.as_ref().clone().into_response(),
            })
        })
    }
}
