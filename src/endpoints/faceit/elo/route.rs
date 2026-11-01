use axum::{Extension, Router, extract::State, routing::get};

use crate::error::Error;
use crate::sites::faceit::{Client, Nickname, models::Player};

use super::model::EloInfo;

pub fn init() -> Router<Client> {
    Router::new().route("/elo", get(elo))
}

async fn elo(
    State(client): State<Client>,
    Extension(nickname): Extension<Nickname>,
) -> Result<String, Error> {
    let player = Player::fetch(&client, &nickname).await?;
    Ok(EloInfo::fetch(&client, &player).await?.to_string())
}
