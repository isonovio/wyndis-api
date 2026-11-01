use axum::{Extension, Router, extract::State as AxumState, middleware, routing::get};

use crate::error::Error;

use crate::middlewares::faceit_nickname::extract_faceit_nickname;
use crate::middlewares::twitch_channel::extract_twitch_channel;
use crate::sites::faceit::{Client, FaceitNickname};
use crate::state::State;

use super::elo_info::EloInfo;

pub fn init(state: State) -> Router {
    Router::new()
        .route("/elo", get(elo))
        .route_layer(middleware::from_fn_with_state(
            state.channels,
            extract_faceit_nickname,
        ))
        .route_layer(middleware::from_fn(extract_twitch_channel))
        .with_state(state.client)
}

async fn elo(
    AxumState(client): AxumState<Client>,
    Extension(nickname): Extension<FaceitNickname>,
) -> Result<String, Error> {
    Ok(EloInfo::fetch(&client, &nickname).await?.to_string())
}
