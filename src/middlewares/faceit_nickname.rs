use axum::{
    Extension,
    extract::{Query, Request},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;

use crate::error::Error;
use crate::sites::faceit;

use super::person::ResolvedPerson;

#[derive(Deserialize)]
struct NicknameQuery {
    id: Option<String>,
}

pub async fn extract_faceit_nickname(
    Extension(person): Extension<ResolvedPerson>,
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
        Some(nickname) => faceit::Nickname::try_from(nickname.to_owned())
            .map_err(|_| Error::bad_request("Invalid FACEIT nickname"))?,
        None => person?
            .faceit_nickname
            .ok_or_else(|| Error::not_found("Missing FACEIT nickname"))?,
    };
    request.extensions_mut().insert(nickname);
    Ok(next.run(request).await)
}

#[cfg(test)]
#[path = "../tests/middlewares/faceit_nickname/tests.rs"]
mod tests;
