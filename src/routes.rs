use anyhow::Result;
use axum::Router;

use crate::config::Config;
use crate::env::Env;
use crate::faceit;
use crate::middlewares::rate_limit;

pub fn init(config: Config, env: &Env) -> Result<Router> {
    let state = faceit::State::new(config.faceit, &env.faceit_api_key)?;
    Ok(compose(state, config.rate_limit))
}

pub fn compose(faceit: faceit::State, config: rate_limit::Config) -> Router {
    Router::new()
        .nest("/api/faceit", faceit::routes::init(faceit))
        .layer(rate_limit::layer(config))
}
