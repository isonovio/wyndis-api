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
async fn normal_and_nightbot_budgets_are_independent() {
    for nightbot_first in [false, true] {
        let mut config = config();
        config.global.rate_limit.requests_per_minute = 1.try_into().unwrap();
        config.global.rate_limit.burst = 1.try_into().unwrap();
        let app = routes::compose(
            State::new(config.persons, config.faceit.client).unwrap(),
            config.global,
            config.faceit,
        );
        let normal = async {
            assert_eq!(
                request(&app, "/missing").await.status(),
                StatusCode::NOT_FOUND
            );
            let response = request(&app, "/api/faceit/elo").await;
            assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
            assert!(response.headers().contains_key(header::RETRY_AFTER));
        };
        let nightbot = async {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/nightbot/age")
                        .header("nightbot-channel", "provider=twitch&name=qiyarah")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert!(
                response
                    .into_body()
                    .collect()
                    .await
                    .unwrap()
                    .to_bytes()
                    .len()
                    > 1
            );
            for path in [
                "/api/nightbot/age",
                "/api/nightbot/elo?id=owner",
                "/api/nightbot/missing",
                "/api/nightbot",
            ] {
                let response = request(&app, path).await;
                assert_eq!(response.status(), StatusCode::OK);
                assert!(!response.headers().contains_key(header::RETRY_AFTER));
                assert_eq!(
                    response.into_body().collect().await.unwrap().to_bytes(),
                    " "
                );
            }
        };
        match nightbot_first {
            true => {
                nightbot.await;
                normal.await;
            }
            false => {
                normal.await;
                nightbot.await;
            }
        }
    }
}

#[tokio::test]
async fn faceit_budgets_are_independent_and_do_not_limit_age() {
    let mut config = config();
    config.global.rate_limit.burst = 100.try_into().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base_url = format!("http://{}/", listener.local_addr().unwrap());
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counter = calls.clone();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().fallback(move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                async { StatusCode::NOT_FOUND }
            }),
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
    let nightbot = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/nightbot/elo?id=third")
                .header("nightbot-channel", "provider=twitch&name=qiyarah")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(nightbot.status(), StatusCode::OK);
    assert_eq!(
        nightbot.into_body().collect().await.unwrap().to_bytes(),
        " "
    );
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
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
            request(&app, "/api/nightbot/age").await.status(),
            StatusCode::OK
        );
    }
}
