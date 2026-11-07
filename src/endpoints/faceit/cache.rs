use axum::{extract::Request, http::Method};

use crate::{middlewares::cache::RequestKey, sites::faceit::Nickname};

#[derive(Clone, Eq, Hash, PartialEq)]
pub(super) struct FaceitRequest {
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
#[path = "../../tests/endpoints/faceit/cache/tests.rs"]
mod tests;
