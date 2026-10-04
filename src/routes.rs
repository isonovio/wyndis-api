use anyhow::Result;
use axum::Router;

use crate::config::Config;
use crate::faceit;
use crate::middlewares::rate_limit;

pub fn init(config: Config) -> Result<Router> {
    let state = faceit::State::new(config.faceit)?;
    Ok(compose(state, config.rate_limit))
}

pub fn compose(faceit: faceit::State, config: rate_limit::Config) -> Router {
    Router::new()
        .nest("/api/faceit", faceit::routes::init(faceit))
        .layer(rate_limit::layer(config))
}
