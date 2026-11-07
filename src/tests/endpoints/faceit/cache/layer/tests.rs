use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    http::{Request as HttpRequest, header},
    middleware,
    routing::get,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::middlewares::{
    faceit_nickname::extract_faceit_nickname, person::ResolvedPerson, rate_limit,
};
use crate::models::Person;

use super::*;
use crate::{
    error::Error,
    middlewares::cache::{BufferedResponse, CacheLayer},
};
use axum::{Extension, body::Body, http::StatusCode, response::Response};

type Cache = CacheLayer<FaceitRequest, BufferedResponse>;

fn app(cache: Cache, calls: Arc<AtomicUsize>, limited: bool) -> Router {
    let endpoint = move |Extension(nickname): Extension<Nickname>| {
        let calls = calls.clone();
        async move {
            calls.fetch_add(1, Ordering::SeqCst);
            tokio::task::yield_now().await;
            match nickname.to_string().as_str() {
                "missing" => Err(Error::not_found("Player not found")),
                _ => Ok(nickname.to_string()),
            }
        }
    };
    let mut app = Router::new()
        .route("/elo", get(endpoint.clone()))
        .route("/other", get(endpoint));
    if limited {
        app = app.route_layer(rate_limit::layer(rate_limit::Config {
            requests_per_minute: 6.try_into().unwrap(),
            burst: 1.try_into().unwrap(),
        }));
    }
    let person: Person = toml::from_str("id = 'owner'\nfaceit_nickname = 'owner'").unwrap();
    app.route_layer(cache)
        .route_layer(middleware::from_fn(extract_faceit_nickname))
        .layer(Extension(ResolvedPerson::Ok(person)))
}

async fn request(app: &Router, path: &str) -> Response {
    app.clone()
        .oneshot(
            HttpRequest::builder()
                .uri(path)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn hits_bypass_limit_and_share_resolved_nickname_across_request_forms() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = app(Cache::new(Duration::from_secs(30)), calls.clone(), true);
    let first = request(&app, "/elo").await;
    assert_eq!(first.status(), StatusCode::OK);
    let headers = first.headers().clone();
    for path in ["/elo?id=owner", "/elo?id=%20owner%20", "/elo?id="] {
        let response = request(&app, path).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers(), &headers);
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            "owner"
        );
    }
    for path in ["/elo?id=other", "/other?id=owner"] {
        let response = request(&app, path).await;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().contains_key(header::RETRY_AFTER));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn expired_entries_refresh_and_errors_are_not_cached() {
    let cache = Cache::new(Duration::ZERO);
    let calls = Arc::new(AtomicUsize::new(0));
    let app = app(cache.clone(), calls.clone(), false);
    assert_eq!(request(&app, "/elo").await.status(), StatusCode::OK);
    assert_eq!(request(&app, "/elo").await.status(), StatusCode::OK);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    for _ in 0..2 {
        assert_eq!(
            request(&app, "/elo?id=missing").await.status(),
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn concurrent_misses_fetch_once() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = app(Cache::new(Duration::from_secs(30)), calls.clone(), true);
    let (first, second) = tokio::join!(request(&app, "/elo"), request(&app, "/elo?id=owner"));
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::OK);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invalid_nickname_does_not_consume_the_miss_budget() {
    let calls = Arc::new(AtomicUsize::new(0));
    let app = app(Cache::new(Duration::from_secs(30)), calls.clone(), true);
    assert_eq!(
        request(&app, "/elo?id=two%20names").await.status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(request(&app, "/elo").await.status(), StatusCode::OK);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
