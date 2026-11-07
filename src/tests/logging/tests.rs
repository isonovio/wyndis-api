use std::{
    io::{self, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{Router, body::Body, extract::Request, http::StatusCode, middleware, routing::get};
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;
use tracing::warn;

use crate::{
    error::Error,
    middlewares::{
        cache::{BufferedResponse, CacheLayer, RequestKey},
        rate_limit,
    },
};

#[derive(Clone, Default)]
struct Output(Arc<Mutex<Vec<u8>>>);

impl Write for Output {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Output {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + use<> {
        let output = self.clone();
        tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || output.clone())
            .finish()
    }
}

async fn request(app: Router, path: &str) -> StatusCode {
    app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

#[tokio::test]
async fn logs_direct_rejections_without_duplicate_propagation() {
    use crate::middlewares::faceit_nickname::extract_faceit_nickname;

    let output = Output::default();
    async {
        let app = Router::new()
            .route("/elo", get(|| async { "ok" }))
            .route_layer(middleware::from_fn(extract_faceit_nickname))
            .layer(rate_limit::layer(rate_limit::Config {
                requests_per_minute: 1.try_into().unwrap(),
                burst: 3.try_into().unwrap(),
            }));
        assert_eq!(
            request(app.clone(), "/elo?id=valid-player").await,
            StatusCode::OK
        );
        assert!(output.text().is_empty());
        assert_eq!(request(app.clone(), "/elo").await, StatusCode::BAD_REQUEST);
        assert_eq!(
            request(app.clone(), "/elo?id=invalid%20name").await,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(app, "/elo?id=valid-player").await,
            StatusCode::TOO_MANY_REQUESTS
        );
    }
    .with_subscriber(output.subscriber())
    .await;
    let text = output.text();
    assert_eq!(text.lines().count(), 3);
    assert!(text.contains("Missing FACEIT nickname"));
    assert!(text.contains("Invalid FACEIT nickname"));
    assert!(text.contains("Rate limit rejected request"));
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct Key;

impl RequestKey for Key {
    fn from_request(_: &Request) -> Option<Self> {
        Some(Self)
    }
}

#[tokio::test]
async fn shared_cache_failure_logs_once_for_the_underlying_failure() {
    let output = Output::default();
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = calls.clone();
    let app = Router::new()
        .route(
            "/fail",
            get(move || {
                counter.fetch_add(1, Ordering::SeqCst);
                async {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    warn!("Upstream failed");
                    Error::bad_gateway("Upstream failed")
                }
            }),
        )
        .layer(CacheLayer::<Key, BufferedResponse>::new(
            Duration::from_secs(30),
        ));
    async {
        let (first, second) = tokio::join!(request(app.clone(), "/fail"), request(app, "/fail"));
        assert_eq!(first, StatusCode::BAD_GATEWAY);
        assert_eq!(second, StatusCode::BAD_GATEWAY);
    }
    .with_subscriber(output.subscriber())
    .await;
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(output.text().lines().count(), 1);
}
