use std::sync::Arc;

use axum::{Router, body::Body, http::StatusCode, middleware, routing::get};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::config::Config;
use crate::middlewares::person::extract_person;
use crate::sites::faceit;
use crate::sources::nightbot;

use super::*;

#[tokio::test]
async fn middleware_resolves_nickname_before_handler() {
    let config: Config = toml::from_str("bind_addr = '127.0.0.1:3000'\n[rate_limit.global]\nrequests_per_minute = 600\nburst = 100\n[rate_limit.faceit]\nrequests_per_minute = 2\nburst = 1\n[[persons]]\nid = 'internal-owner'\ntwitch_channel = 'Streamer'\nfaceit_nickname = 'owner'\n[[persons]]\nid = 'no-faceit'\ntwitch_channel = 'empty'").unwrap();
    let app =
        Router::new()
            .route(
                "/",
                get(
                    |Extension(nickname): Extension<faceit::Nickname>| async move {
                        nickname.to_string()
                    },
                ),
            )
            .route_layer(middleware::from_fn(extract_faceit_nickname))
            .route_layer(middleware::from_fn_with_state(
                Arc::new(config.persons),
                extract_person,
            ))
            .route_layer(middleware::from_fn(nightbot::extract_twitch_channel));
    for (uri, header, status, body) in [
        (
            "/",
            Some("provider=twitch&name=empty"),
            StatusCode::NOT_FOUND,
            "Missing FACEIT nickname",
        ),
        (
            "/?id=donk666",
            Some("provider=twitch&name=empty"),
            StatusCode::OK,
            "donk666",
        ),
        (
            "/?id=donk666",
            Some("provider=twitch&name=unknown"),
            StatusCode::OK,
            "donk666",
        ),
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
            "Unknown person",
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
