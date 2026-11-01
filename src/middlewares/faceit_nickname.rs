use axum::{
    Extension,
    extract::{Query, Request, State as AxumState},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;

use crate::error::Error;

use crate::sites::faceit::FaceitNickname;
use crate::sites::nightbot::TwitchChannel;
use crate::state::Channels;

#[derive(Deserialize)]
struct NicknameQuery {
    id: Option<String>,
}

pub async fn extract_faceit_nickname(
    AxumState(channels): AxumState<Channels>,
    Extension(channel): Extension<TwitchChannel>,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    let Query(query) = Query::<NicknameQuery>::try_from_uri(request.uri())
        .map_err(|_| Error::bad_request("Invalid query parameters"))?;
    let nickname = match query
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(nickname) => FaceitNickname::try_from(nickname.to_owned())
            .map_err(|_| Error::bad_request("Invalid FACEIT nickname"))?,
        None => channels
            .get(channel.name()?)
            .cloned()
            .ok_or(Error::not_found("Missing FACEIT channel mapping"))?,
    };
    request.extensions_mut().insert(nickname);
    Ok(next.run(request).await)
}

#[cfg(test)]
#[path = "../tests/middlewares/faceit_nickname/tests.rs"]
mod tests;
