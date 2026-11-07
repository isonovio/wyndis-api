use std::sync::Arc;

use axum::{
    Extension,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use tracing::warn;

use crate::error::Error;
use crate::models::Persons;
use crate::sites::twitch::TwitchChannelName;

pub async fn extract_person(
    State(persons): State<Arc<Persons>>,
    Extension(channel): Extension<TwitchChannelName>,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    let person = (persons.from_twitch(&channel).cloned())
        .ok_or_else(|| warn!(%channel, "Unknown person"))
        .map_err(|_| Error::not_found("Unknown person"))?;
    request.extensions_mut().insert(person);
    Ok(next.run(request).await)
}

#[cfg(test)]
#[path = "../tests/middlewares/person/tests.rs"]
mod tests;
