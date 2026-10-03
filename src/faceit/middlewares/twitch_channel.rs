use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;

use crate::error::Error;

use crate::faceit::config::is_twitch_channel_name;

#[derive(Clone)]
pub enum TwitchChannel {
    Name(String),
    Missing,
    Invalid,
}

impl TwitchChannel {
    pub fn name(&self) -> Result<&str, Error> {
        match self {
            Self::Name(name) => Ok(name),
            Self::Missing => Err(Error::new(
                StatusCode::BAD_REQUEST,
                "Provide a FACEIT nickname or a Nightbot channel header.",
            )),
            Self::Invalid => Err(Error::new(
                StatusCode::BAD_REQUEST,
                "Invalid Twitch channel header.",
            )),
        }
    }
}

#[derive(Deserialize)]
struct ChannelHeader {
    provider: String,
    name: String,
}

pub async fn extract_channel(mut request: Request, next: Next) -> Response {
    let channel = parse_channel(request.headers());
    request.extensions_mut().insert(channel);
    next.run(request).await
}

fn parse_channel(headers: &HeaderMap) -> TwitchChannel {
    let Some(header) = headers.get("nightbot-channel") else {
        return TwitchChannel::Missing;
    };
    let channel = header
        .to_str()
        .ok()
        .and_then(|value| serde_urlencoded::from_str::<ChannelHeader>(value).ok());
    match channel {
        Some(channel) if channel.provider == "twitch" && is_twitch_channel_name(&channel.name) => {
            TwitchChannel::Name(channel.name.to_ascii_lowercase())
        }
        _ => TwitchChannel::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Extension, Router, body::Body, middleware, routing::get};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn middleware_extracts_and_validates_channel() {
        let app = Router::new()
            .route(
                "/",
                get(|Extension(channel): Extension<TwitchChannel>| async move {
                    channel.name().map(str::to_owned)
                }),
            )
            .route_layer(middleware::from_fn(extract_channel));
        for (header, status, body) in [
            (
                Some("name=Streamer&provider=twitch&providerId=123"),
                StatusCode::OK,
                "streamer",
            ),
            (
                Some("name=streamer&provider=youtube"),
                StatusCode::BAD_REQUEST,
                "Invalid Twitch channel header.",
            ),
            (
                Some("provider=twitch"),
                StatusCode::BAD_REQUEST,
                "Invalid Twitch channel header.",
            ),
            (
                Some("name=bad%20name&provider=twitch"),
                StatusCode::BAD_REQUEST,
                "Invalid Twitch channel header.",
            ),
            (
                None,
                StatusCode::BAD_REQUEST,
                "Provide a FACEIT nickname or a Nightbot channel header.",
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
            .route_layer(middleware::from_fn(extract_channel));
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
