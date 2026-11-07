use axum::{extract::Request, http::Method};

use crate::{
    middlewares::cache::{BufferedResponse, CacheLayer, RequestKey},
    sites::faceit::Nickname,
};

use super::config::Config;

pub fn layer(config: Config) -> CacheLayer<FaceitRequest, BufferedResponse> {
    CacheLayer::with_capacity(config.ttl, config.max_entries)
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct FaceitRequest {
    path: String,
    nickname: String,
}

impl RequestKey for FaceitRequest {
    fn from_request(request: &Request) -> Option<Self> {
        (request.method() == Method::GET).then_some(())?;
        Some(Self {
            path: request.uri().path().to_owned(),
            nickname: request.extensions().get::<Nickname>()?.to_string(),
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/endpoints/faceit/cache/layer/tests.rs"]
mod tests;
