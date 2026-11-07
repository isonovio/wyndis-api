use std::{num::NonZeroU32, time::Duration};
use tracing::warn;

use axum::{
    body::Body,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use governor::middleware::NoOpMiddleware;
use serde::Deserialize;
use tower_governor::{
    GovernorError, GovernorLayer, governor::GovernorConfigBuilder,
    key_extractor::GlobalKeyExtractor,
};

#[derive(Clone, Copy, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub requests_per_minute: NonZeroU32,
    pub burst: NonZeroU32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            requests_per_minute: NonZeroU32::new(60).unwrap(),
            burst: NonZeroU32::new(1).unwrap(),
        }
    }
}

pub fn layer(config: Config) -> GovernorLayer<GlobalKeyExtractor, NoOpMiddleware, Body> {
    let period = Duration::from_secs_f64(60.0 / f64::from(config.requests_per_minute.get()));
    let governor = GovernorConfigBuilder::default()
        .period(period)
        .burst_size(config.burst.get())
        .key_extractor(GlobalKeyExtractor)
        .finish()
        .expect("rate limit values are nonzero");
    GovernorLayer::new(governor).error_handler(error_response)
}

fn error_response(error: GovernorError) -> Response {
    warn!(%error, "Rate limit rejected request");
    match error {
        GovernorError::TooManyRequests { wait_time, .. } => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, wait_time.saturating_add(1).to_string())],
            "Rate limit exceeded",
        )
            .into_response(),
        error => error.into(),
    }
}

#[cfg(test)]
#[path = "../tests/middlewares/rate_limit/tests.rs"]
mod tests;
