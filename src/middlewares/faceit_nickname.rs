use axum::{
    Extension,
    extract::{Query, Request},
    middleware::Next,
    response::Response,
};
use serde::Deserialize;
use tracing::warn;

use crate::error::Error;
use crate::models::Person;
use crate::sites::faceit;

#[derive(Deserialize)]
struct NicknameQuery {
    id: Option<String>,
}

pub async fn extract_faceit_nickname(
    person: Option<Extension<Person>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    let Query(query) = Query::<NicknameQuery>::try_from_uri(request.uri())
        .inspect_err(|error| warn!(%error, "Invalid query parameters"))
        .map_err(|_| Error::bad_request("Invalid query parameters"))?;
    let nickname = match (query.id.as_deref())
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        Some(nickname) => faceit::Nickname::try_from(nickname.to_owned())
            .inspect_err(|error| warn!(%error, "Invalid FACEIT nickname"))
            .map_err(|_| Error::bad_request("Invalid FACEIT nickname"))?,
        None => person
            .map(|Extension(person)| person)
            .ok_or_else(|| warn!("Missing FACEIT nickname"))
            .map_err(|_| Error::bad_request("Missing FACEIT nickname"))?
            .faceit_nickname
            .ok_or_else(|| warn!("Missing FACEIT nickname"))
            .map_err(|_| Error::not_found("Missing FACEIT nickname"))?,
    };
    request.extensions_mut().insert(nickname);
    Ok(next.run(request).await)
}

#[cfg(test)]
#[path = "../tests/middlewares/faceit_nickname/tests.rs"]
mod tests;
