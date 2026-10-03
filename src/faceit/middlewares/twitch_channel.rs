use axum::{extract::Request, middleware::Next, response::Response};

use crate::faceit::sites::nightbot::parse_channel;

pub async fn extract_twitch_channel(mut request: Request, next: Next) -> Response {
    let channel = parse_channel(request.headers());
    request.extensions_mut().insert(channel);
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::faceit::sites::nightbot::TwitchChannel;
    use axum::http::StatusCode;
    use axum::{Extension, Router, body::Body, middleware, routing::get};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn middleware_extracts_and_validates_channel() {
        let app = Router::new()
            .route(
                "/",
                get(|Extension(channel): Extension<TwitchChannel>| async move {
                    channel.name().map(ToString::to_string)
                }),
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
    async fn channel_is_optional_for_handlers_that_do_not_need_it() {
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
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}
