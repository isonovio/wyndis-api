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
