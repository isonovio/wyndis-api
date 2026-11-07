use std::sync::Arc;

use axum::{
    Extension,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::error::Error;
use crate::models::{Person, Persons};
use crate::sites::nightbot::TwitchChannel;

pub type ResolvedPerson = Result<Person, Error>;

pub async fn extract_person(
    State(persons): State<Arc<Persons>>,
    Extension(channel): Extension<TwitchChannel>,
    mut request: Request,
    next: Next,
) -> Response {
    let person: ResolvedPerson = channel.name().and_then(|channel| {
        (persons.from_twitch(channel).cloned()).ok_or_else(|| Error::not_found("Unknown person"))
    });
    request.extensions_mut().insert(person);
    next.run(request).await
}

#[cfg(test)]
#[path = "../tests/middlewares/person/tests.rs"]
mod tests;
