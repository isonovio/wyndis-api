use anyhow::Result;
use axum::Router;

use crate::config::Config;
use crate::endpoints::faceit;
use crate::middlewares::rate_limit;
use crate::state::State;

pub fn init(config: Config) -> Result<Router> {
    let state = State::new(config.persons)?;
    Ok(compose(state, config.rate_limit))
}

pub fn compose(faceit: State, config: rate_limit::Config) -> Router {
    Router::new()
        .nest("/api/faceit", faceit::elo::init(faceit))
        .layer(rate_limit::layer(config))
}
