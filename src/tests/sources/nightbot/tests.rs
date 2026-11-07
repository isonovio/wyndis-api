use super::*;
use crate::sites::twitch::TwitchChannelName;
use axum::http::StatusCode;
use axum::{Extension, Router, body::Body, middleware, routing::get};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn middleware_extracts_and_validates_channel() {
    let app =
        Router::new()
            .route(
                "/",
                get(
                    |Extension(channel): Extension<TwitchChannelName>| async move {
                        channel.to_string()
                    },
                ),
            )
            .route_layer(middleware::from_fn(extract_twitch_channel));
    for (header, status, body) in [
        (
            Some("name=Streamer&provider=twitch&providerId=123"),
            StatusCode::OK,
            "streamer",
        ),
        (
            Some("name=streamer&provider=youtube"),
            StatusCode::BAD_REQUEST,
            "Invalid Nightbot channel header",
        ),
        (
            Some("provider=twitch"),
            StatusCode::BAD_REQUEST,
            "Invalid Nightbot channel header",
        ),
        (
            Some("name=bad%20name&provider=twitch"),
            StatusCode::BAD_REQUEST,
            "Invalid Nightbot channel header",
        ),
        (
            None,
            StatusCode::BAD_REQUEST,
            "Missing Nightbot channel header",
        ),
    ] {
        let mut request = Request::builder().uri("/");
        if let Some(header) = header {
            request = request.header("nightbot-channel", header);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            body
        );
    }
}

#[tokio::test]
async fn invalid_channels_short_circuit_even_handlers_that_do_not_need_them() {
    let app = Router::new()
        .route("/", get(|| async { "explicit nickname" }))
        .route_layer(middleware::from_fn(extract_twitch_channel));
    for header in [None, Some("invalid")] {
        let mut request = Request::builder().uri("/");
        if let Some(header) = header {
            request = request.header("nightbot-channel", header);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}

#[tokio::test]
async fn silence_errors_converts_failures_and_preserves_successes() {
    for status in [
        StatusCode::OK,
        StatusCode::BAD_REQUEST,
        StatusCode::NOT_FOUND,
        StatusCode::METHOD_NOT_ALLOWED,
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::INTERNAL_SERVER_ERROR,
        StatusCode::BAD_GATEWAY,
        StatusCode::GATEWAY_TIMEOUT,
    ] {
        let app = Router::new()
            .fallback(move || async move { (status, [("retry-after", "10")], "response") })
            .layer(middleware::from_fn(silence_errors));
        for path in [
            "/api/nightbot",
            "/api/nightbot/elo",
            "/api/nightbot/missing",
            "/api/faceit/elo",
            "/api/nightbot-other",
        ] {
            let response = app
                .clone()
                .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            let silent = status != StatusCode::OK;
            assert_eq!(
                response.status(),
                if silent { StatusCode::OK } else { status }
            );
            assert_eq!(response.headers().contains_key("retry-after"), !silent);
            assert_eq!(
                response.into_body().collect().await.unwrap().to_bytes(),
                if silent { " " } else { "response" }
            );
        }
    }
}
