use std::{
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{Request as HttpRequest, StatusCode},
    response::IntoResponse,
};
use tower::{Layer, ServiceExt, service_fn};

use super::*;

#[derive(Clone, Eq, PartialEq, Hash)]
struct Key(String);

impl RequestKey for Key {
    fn from_request(request: &Request) -> Option<Self> {
        Some(Self(request.uri().path().to_owned()))
    }
}

#[tokio::test]
async fn cache_hits_do_not_even_call_the_inner_service() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let inner = service_fn(move |_: Request| {
        count.fetch_add(1, Ordering::SeqCst);
        async { Ok::<_, Infallible>("cached".into_response()) }
    });
    let service = CacheLayer::<Key, BufferedResponse>::new(Duration::from_secs(30)).layer(inner);
    for path in ["/one", "/one", "/two"] {
        assert_eq!(
            service
                .clone()
                .oneshot(
                    HttpRequest::builder()
                        .uri(path)
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn errors_keep_their_status_headers_and_body_and_are_retried() {
    let calls = Arc::new(AtomicUsize::new(0));
    let count = calls.clone();
    let inner = service_fn(move |_: Request| {
        count.fetch_add(1, Ordering::SeqCst);
        async {
            Ok::<_, Infallible>(
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    [("retry-after", "10")],
                    "Rate limit exceeded",
                )
                    .into_response(),
            )
        }
    });
    let service = CacheLayer::<Key, BufferedResponse>::new(Duration::from_secs(30)).layer(inner);
    for _ in 0..2 {
        let response = service
            .clone()
            .oneshot(Request::new(Body::empty()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()["retry-after"], "10");
        assert_eq!(
            to_bytes(response.into_body(), 1024).await.unwrap(),
            "Rate limit exceeded"
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
