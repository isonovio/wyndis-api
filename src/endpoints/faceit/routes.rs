use std::time::Duration;

use axum::{Router, middleware};

use crate::middlewares::{
    cache::{BufferedResponse, CacheLayer},
    faceit_nickname::extract_faceit_nickname,
    rate_limit,
};
use crate::sites::faceit::Client;

use super::{cache::FaceitRequest, elo};

pub fn init(client: Client, rate_limit: rate_limit::Config) -> Router {
    elo::route::init()
        .route_layer(rate_limit::layer(rate_limit))
        .route_layer(CacheLayer::<FaceitRequest, BufferedResponse>::new(
            Duration::from_secs(30),
        ))
        .route_layer(middleware::from_fn(extract_faceit_nickname))
        .with_state(client)
}
