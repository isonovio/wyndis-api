use anyhow::Result;
use axum::{Router, http::StatusCode, middleware};

use crate::config::{Config, GlobalConfig};
use crate::endpoints::{age, faceit};
use crate::middlewares::{person::extract_person, rate_limit};
use crate::sources::nightbot;
use crate::state::State;

pub fn init(config: Config) -> Result<Router> {
    let state = State::new(config.persons, config.faceit.client)?;
    Ok(compose(state, config.global, config.faceit))
}

pub fn compose(state: State, global: GlobalConfig, faceit: faceit::config::Config) -> Router {
    let State { persons, client } = state;
    let normal = Router::new()
        .nest("/faceit", faceit::routes::init(client.clone(), faceit))
        .layer(rate_limit::layer(global.rate_limit));
    let nightbot = Router::new()
        .merge(age::route::init())
        .merge(faceit::routes::init(client, faceit))
        .route_layer(middleware::from_fn_with_state(persons, extract_person))
        .route_layer(middleware::from_fn(nightbot::extract_twitch_channel))
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(rate_limit::layer(global.rate_limit))
        .layer(middleware::from_fn(nightbot::silence_errors));
    Router::new()
        .nest("/api", normal)
        .nest("/api/nightbot", nightbot)
}
