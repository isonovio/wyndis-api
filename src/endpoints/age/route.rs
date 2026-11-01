use std::sync::Arc;

use axum::{Extension, Router, middleware, routing::get};
use chrono::Utc;

use crate::error::Error;
use crate::middlewares::person::{ResolvedPerson, extract_person};
use crate::middlewares::twitch_channel::extract_twitch_channel;
use crate::models::Persons;

use super::model::Age;

pub fn init(persons: Arc<Persons>) -> Router {
    Router::new()
        .route("/age", get(age))
        .route_layer(middleware::from_fn_with_state(persons, extract_person))
        .route_layer(middleware::from_fn(extract_twitch_channel))
}

async fn age(Extension(person): Extension<ResolvedPerson>) -> Result<String, Error> {
    Ok(Age::at(&person?, Utc::now())?.to_string())
}
