use axum::{Router, middleware};

use crate::middlewares::{faceit_nickname::extract_faceit_nickname, rate_limit};
use crate::sites::faceit::Client;

use super::elo;

pub fn init(client: Client, rate_limit: rate_limit::Config) -> Router {
    elo::route::init()
        .route_layer(rate_limit::layer(rate_limit))
        .route_layer(middleware::from_fn(extract_faceit_nickname))
        .with_state(client)
}
