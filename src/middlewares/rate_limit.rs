use std::{num::NonZeroU32, time::Duration};

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
    match error {
        GovernorError::TooManyRequests { wait_time, .. } => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, wait_time.saturating_add(1).to_string())],
            "Too many requests. Try again shortly.",
        )
            .into_response(),
        error => error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::Request, routing::get};
    use tower::ServiceExt;

    #[tokio::test]
    async fn routes_and_clones_share_one_budget_including_fallback() {
        let app = Router::new()
            .route("/one", get(|| async { "one" }))
            .route("/two", get(|| async { "two" }))
            .layer(layer(Config::default()));
        for (path, expected) in [
            ("/one", StatusCode::OK),
            ("/missing", StatusCode::TOO_MANY_REQUESTS),
            ("/two", StatusCode::TOO_MANY_REQUESTS),
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), expected);
            if expected == StatusCode::TOO_MANY_REQUESTS {
                assert_eq!(response.headers()[header::RETRY_AFTER], "1");
            }
        }
    }

    #[test]
    fn configuration_rejects_zero_limits() {
        assert!(toml::from_str::<Config>("requests_per_minute = 0").is_err());
        assert!(toml::from_str::<Config>("burst = 0").is_err());
    }
}
