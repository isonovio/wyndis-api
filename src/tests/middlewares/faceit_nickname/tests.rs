use std::{collections::HashMap, sync::Arc};

use axum::{Router, body::Body, http::StatusCode, middleware, routing::get};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::middlewares::twitch_channel;

use super::*;

#[tokio::test]
async fn middleware_resolves_nickname_before_handler() {
    let channels: Channels = Arc::new(HashMap::from([(
        "Streamer".to_owned().try_into().unwrap(),
        "owner".to_owned().try_into().unwrap(),
    )]));
    let app =
        Router::new()
            .route(
                "/",
                get(
                    |Extension(nickname): Extension<FaceitNickname>| async move {
                        nickname.to_string()
                    },
                ),
            )
            .route_layer(middleware::from_fn_with_state(
                channels,
                extract_faceit_nickname,
            ))
            .route_layer(middleware::from_fn(twitch_channel::extract_twitch_channel));
    for (uri, header, status, body) in [
        ("/?id=donk666", None, StatusCode::OK, "donk666"),
        ("/?id=donk666", Some("invalid"), StatusCode::OK, "donk666"),
        (
            "/?id=donk666",
            Some("provider=twitch&name=streamer"),
            StatusCode::OK,
            "donk666",
        ),
        (
            "/",
            Some("provider=twitch&name=STREAMER"),
            StatusCode::OK,
            "owner",
        ),
        (
            "/?id=",
            Some("provider=twitch&name=streamer"),
            StatusCode::OK,
            "owner",
        ),
        (
            "/?id=%20",
            Some("provider=twitch&name=streamer"),
            StatusCode::OK,
            "owner",
        ),
        ("/?id=%20donk666%20", None, StatusCode::OK, "donk666"),
        (
            "/?id=two%20names",
            None,
            StatusCode::BAD_REQUEST,
            "Invalid FACEIT nickname",
        ),
        (
            "/",
            None,
            StatusCode::BAD_REQUEST,
            "Missing Nightbot channel header",
        ),
        (
            "/",
            Some("invalid"),
            StatusCode::BAD_REQUEST,
            "Invalid Nightbot channel header",
        ),
        (
            "/",
            Some("provider=twitch&name=unknown"),
            StatusCode::NOT_FOUND,
            "Missing FACEIT channel mapping",
        ),
        (
            "/?id=a&id=b",
            None,
            StatusCode::BAD_REQUEST,
            "Invalid query parameters",
        ),
    ] {
        let mut request = Request::builder().uri(uri);
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
