use anyhow::Result;
use axum::Router;

use crate::config::Config;
use crate::env::Env;
use crate::faceit;

pub fn init(config: Config, env: &Env) -> Result<Router> {
    let state = faceit::State::new(config.faceit, &env.faceit_api_key)?;
    Ok(compose(state))
}

pub fn compose(faceit: faceit::State) -> Router {
    Router::new().nest("/api/faceit", faceit::routes::init(faceit))
}
