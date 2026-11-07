use axum::{Extension, Router, extract::State, routing::get};
use tracing::warn;

use crate::error::Error;
use crate::sites::faceit::{Nickname, client::Client, models::Player};

use super::model::EloInfo;

pub fn init() -> Router<Client> {
    Router::new().route("/elo", get(elo))
}

async fn elo(
    State(client): State<Client>,
    Extension(nickname): Extension<Nickname>,
) -> Result<String, Error> {
    tokio::time::timeout(client.request_timeout(), fetch(&client, &nickname))
        .await
        .inspect_err(|_| warn!(%nickname, "Elo computation timed out"))
        .map_err(|_| Error::gateway_timeout("FACEIT request timed out"))?
}

async fn fetch(client: &Client, nickname: &Nickname) -> Result<String, Error> {
    let player = Player::fetch(client, nickname).await?;
    Ok(EloInfo::fetch(client, &player).await?.to_string())
}
