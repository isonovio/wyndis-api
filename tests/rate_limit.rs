use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use tower::ServiceExt;
use wyndis_api::{config::Config, routes, sites::faceit::client::Client, state::State};

fn config() -> Config {
    Config::load(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("config.toml")).unwrap()
}

async fn request(app: &Router, path: &str) -> axum::response::Response {
    app.clone()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn global_budget_covers_all_routes_and_fallback() {
    let mut config = config();
    config.global.rate_limit.requests_per_minute = 60.try_into().unwrap();
    config.global.rate_limit.burst = 1.try_into().unwrap();
    let app = routes::compose(
        State::new(config.persons, config.faceit.client).unwrap(),
        config.global,
        config.faceit,
    );
    assert_eq!(
        request(&app, "/missing").await.status(),
        StatusCode::NOT_FOUND
    );
    for path in ["/api/age", "/api/faceit/elo?id=owner"] {
        let response = request(&app, path).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "1");
    }
}

#[tokio::test]
async fn faceit_budget_precedes_upstream_work_and_does_not_limit_age() {
    let mut config = config();
    config.global.rate_limit.burst = 100.try_into().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}/", listener.local_addr().unwrap());
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(|| async { StatusCode::NOT_FOUND }),
        )
        .await
        .unwrap();
    });
    let client = Client::with_base_url(base_url.parse().unwrap(), config.faceit.client).unwrap();
    let app = routes::compose(
        State::with_client(client, config.persons),
        config.global,
        config.faceit,
    );
    assert_eq!(
        request(&app, "/api/faceit/elo").await.status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        request(&app, "/api/faceit/elo?id=owner").await.status(),
        StatusCode::NOT_FOUND
    );
    let response = request(&app, "/api/faceit/elo?id=other").await;
    server.abort();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry: u64 = response.headers()[header::RETRY_AFTER]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=10).contains(&retry));
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "Rate limit exceeded"
    );
    for _ in 0..2 {
        assert_eq!(
            request(&app, "/api/age").await.status(),
            StatusCode::BAD_REQUEST
        );
    }
}
