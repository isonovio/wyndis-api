use std::sync::Arc;

use axum::{Extension, Router, extract::State as AxumState, middleware, routing::get};

use crate::error::Error;

use crate::middlewares::faceit_nickname::extract_faceit_nickname;
use crate::middlewares::person::extract_person;
use crate::middlewares::rate_limit;
use crate::middlewares::twitch_channel::extract_twitch_channel;
use crate::models::Persons;
use crate::sites::faceit;

use super::elo_info::EloInfo;

pub fn init(
    persons: Arc<Persons>,
    client: faceit::Client,
    rate_limit: rate_limit::Config,
) -> Router {
    Router::new()
        .route("/elo", get(elo))
        .route_layer(middleware::from_fn(extract_faceit_nickname))
        .route_layer(middleware::from_fn_with_state(persons, extract_person))
        .route_layer(middleware::from_fn(extract_twitch_channel))
        .route_layer(rate_limit::layer(rate_limit))
        .with_state(client)
}

async fn elo(
    AxumState(client): AxumState<faceit::Client>,
    Extension(nickname): Extension<faceit::Nickname>,
) -> Result<String, Error> {
    Ok(EloInfo::fetch(&client, &nickname).await?.to_string())
}
