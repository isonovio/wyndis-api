use axum::{
    Extension,
    extract::{Query, Request, State as AxumState},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use serde::Deserialize;

use crate::error::Error;
use crate::faceit::{State, config::is_faceit_nickname};

use super::twitch_channel::TwitchChannel;

#[derive(Clone)]
pub struct FaceitNickname(pub String);

#[derive(Deserialize)]
struct NicknameQuery {
    id: Option<String>,
}

pub async fn extract_nickname(
    AxumState(state): AxumState<State>,
    Extension(channel): Extension<TwitchChannel>,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    let Query(query) = Query::<NicknameQuery>::try_from_uri(request.uri())
        .map_err(|_| Error::new(StatusCode::BAD_REQUEST, "Invalid query parameters."))?;
    let nickname = match query
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(nickname) => nickname,
        None => match state.channels.get(channel.name()?) {
            Some(nickname) => nickname,
            None => {
                return Err(Error::new(
                    StatusCode::NOT_FOUND,
                    "No FACEIT player is configured for this channel.",
                ));
            }
        },
    };
    if !is_faceit_nickname(&nickname) {
        return Err(Error::new(
            StatusCode::BAD_REQUEST,
            "Provide a single FACEIT nickname.",
        ));
    }
    request
        .extensions_mut()
        .insert(FaceitNickname(nickname.to_owned()));
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::faceit::{Client, middlewares::twitch_channel};
    use axum::{Router, body::Body, middleware, routing::get};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn middleware_resolves_nickname_before_handler() {
        let state = State::with_client(
            Client::new("test-key").unwrap(),
            HashMap::from([("Streamer".into(), "owner".into())]),
        );
        let app = Router::new()
            .route(
                "/",
                get(
                    |Extension(FaceitNickname(nickname)): Extension<FaceitNickname>| async move {
                        nickname
                    },
                ),
            )
            .route_layer(middleware::from_fn_with_state(state, extract_nickname))
            .route_layer(middleware::from_fn(twitch_channel::extract_channel));
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
                "Provide a single FACEIT nickname.",
            ),
            (
                "/",
                None,
                StatusCode::BAD_REQUEST,
                "Provide a FACEIT nickname or a Nightbot channel header.",
            ),
            (
                "/",
                Some("invalid"),
                StatusCode::BAD_REQUEST,
                "Invalid Twitch channel header.",
            ),
            (
                "/",
                Some("provider=twitch&name=unknown"),
                StatusCode::NOT_FOUND,
                "No FACEIT player is configured for this channel.",
            ),
            (
                "/?id=a&id=b",
                None,
                StatusCode::BAD_REQUEST,
                "Invalid query parameters.",
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
}
