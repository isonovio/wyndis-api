use anyhow::Result;
use axum::Router;

use crate::config::{Config, RateLimits};
use crate::endpoints::{age, faceit};
use crate::state::State;

pub fn init(config: Config) -> Result<Router> {
    let state = State::new(config.persons)?;
    Ok(compose(state, config.rate_limit))
}

pub fn compose(state: State, rate_limit: RateLimits) -> Router {
    let State { persons, client } = state;
    Router::new()
        .nest("/api", age::route::init(persons.clone(), rate_limit.age))
        .nest(
            "/api/faceit",
            faceit::elo::init(persons, client, rate_limit.elo),
        )
}
