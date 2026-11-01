use anyhow::Result;
use axum::{Router, middleware};

use crate::config::{Config, RateLimits};
use crate::endpoints::{age, faceit};
use crate::middlewares::{person::extract_person, rate_limit};
use crate::sources::nightbot;
use crate::state::State;

pub fn init(config: Config) -> Result<Router> {
    let state = State::new(config.persons)?;
    Ok(compose(state, config.rate_limit))
}

pub fn compose(state: State, rate_limit: RateLimits) -> Router {
    let State { persons, client } = state;
    let endpoints = age::route::init()
        .nest("/faceit", faceit::routes::init(client, rate_limit.faceit))
        .route_layer(middleware::from_fn_with_state(persons, extract_person))
        .route_layer(middleware::from_fn(nightbot::extract_twitch_channel));
    Router::new()
        .nest("/api", endpoints)
        .layer(rate_limit::layer(rate_limit.global))
}
