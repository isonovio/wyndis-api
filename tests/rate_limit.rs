use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use wyndis_api::{config::Config, routes, state::State};

#[tokio::test]
async fn elo_and_age_have_independent_budgets_shared_by_router_clones() {
    let config =
        Config::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml")).unwrap();
    let app = routes::compose(State::new(config.persons).unwrap(), config.rate_limit);
    for (path, status, retry) in [
        ("/missing", StatusCode::NOT_FOUND, None),
        ("/api/faceit/elo", StatusCode::BAD_REQUEST, None),
        ("/api/faceit/elo", StatusCode::TOO_MANY_REQUESTS, Some(30)),
        ("/api/age", StatusCode::BAD_REQUEST, None),
        ("/api/age", StatusCode::TOO_MANY_REQUESTS, Some(1)),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status, "{path}");
        if let Some(retry) = retry {
            assert_eq!(response.headers()[header::RETRY_AFTER], retry.to_string());
            assert_eq!(
                response.into_body().collect().await.unwrap().to_bytes(),
                "Rate limit exceeded"
            );
        }
    }
}
