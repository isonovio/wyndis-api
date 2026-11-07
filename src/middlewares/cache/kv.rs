use std::{future::Future, hash::Hash};

use axum::{
    extract::Request,
    response::{IntoResponse, Response},
};

use super::buffer::BufferedResponse;

pub trait RequestKey: Clone + Eq + Hash + Send + Sync + 'static {
    /// Return None to bypass caching for this request.
    fn from_request(request: &Request) -> Option<Self>;
}

pub trait CachedValue: Clone + IntoResponse + Send + Sync + 'static {
    /// Return Err to share this response with waiting callers without caching it.
    fn from_response(
        response: Response,
    ) -> impl Future<Output = Result<Self, BufferedResponse>> + Send;
}
