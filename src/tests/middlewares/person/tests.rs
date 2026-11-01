use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    middleware,
    routing::get,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use crate::config::Config;
use crate::middlewares::twitch_channel::extract_twitch_channel;

use super::*;

#[tokio::test]
async fn resolves_person_without_requiring_faceit_data() {
    let config: Config = toml::from_str("bind_addr = '127.0.0.1:3000'\n[[persons]]\nid = 'internal-owner'\ntwitch_channel = 'Streamer'").unwrap();
    let app = Router::new()
        .route(
            "/",
            get(|Extension(person): Extension<ResolvedPerson>| async move {
                person.map(|person| person.id.clone())
            }),
        )
        .route_layer(middleware::from_fn_with_state(
            Arc::new(config.persons),
            extract_person,
        ))
        .route_layer(middleware::from_fn(extract_twitch_channel));
    for (header, status, body) in [
        (
            Some("provider=twitch&name=STREAMER"),
            StatusCode::OK,
            "internal-owner",
        ),
        (
            Some("provider=twitch&name=unknown"),
            StatusCode::NOT_FOUND,
            "Unknown person",
        ),
        (
            Some("invalid"),
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
