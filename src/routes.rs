use anyhow::Result;
use axum::Router;

use crate::config::Config;
use crate::endpoints::{age, faceit};
use crate::middlewares::rate_limit;
use crate::state::State;

pub fn init(config: Config) -> Result<Router> {
    let state = State::new(config.persons)?;
    Ok(compose(state, config.rate_limit))
}

pub fn compose(state: State, config: rate_limit::Config) -> Router {
    let State { persons, client } = state;
    Router::new()
        .nest("/api", age::route::init(persons.clone()))
        .nest("/api/faceit", faceit::elo::init(persons, client))
        .layer(rate_limit::layer(config))
}
