use std::fmt;

use jiff::Timestamp;
use tracing::{error, warn};

use crate::error::Error;

use crate::sites::faceit::{
    client::Client,
    ext::{LastMatch, Ranking, Today, day_start},
    models::{Match, Player, Region},
};

#[derive(Debug)]
pub struct EloInfo {
    pub level: u8,
    pub elo: u32,
    pub rank: Option<u32>,
    pub region: Region,
    pub today: Today,
    pub last_match: Option<LastMatch>,
}

impl EloInfo {
    pub async fn fetch(client: &Client, player: &Player) -> Result<Self, Error> {
        Self::fetch_at(client, player, Timestamp::now()).await
    }

    pub async fn fetch_at(client: &Client, player: &Player, now: Timestamp) -> Result<Self, Error> {
        let id = player.id;
        let game = (player.games.get("cs2"))
            .ok_or_else(missing_elo)
            .inspect_err(|error| warn!(%id, %error))?;
        let elo = (game.faceit_elo)
            .ok_or_else(missing_elo)
            .inspect_err(|error| warn!(%id, %error))?;
        let level = (game.skill_level)
            .ok_or_else(missing_elo)
            .inspect_err(|error| warn!(%id, %error))?;
        let timezone = (game.region.timezone())
            .ok_or_else(|| Error::internal_server_error("Unable to resolve FACEIT timezone"))
            .inspect_err(|error| error!(%id, %error))?;
        let start = day_start(now, timezone)
            .ok_or_else(|| Error::bad_request("Missing regional midnight"))
            .inspect_err(|error| warn!(%id, %error))?;
        let (rank, matches) = tokio::try_join!(
            Ranking::fetch(client, id, &game.region),
            Match::fetch(client, id, start, now),
        )?;
        let today = Today::from_matches(&matches, start, now)
            .inspect_err(|error| warn!(%id, %error, "Invalid daily match data"))?;
        let last_match = (matches.first().map(LastMatch::from_match).transpose())
            .inspect_err(|error| warn!(%id, %error, "Invalid last match data"))?;
        Ok(Self {
            level,
            elo,
            rank: rank.position,
            region: game.region.clone(),
            today,
            last_match,
        })
    }
}

impl fmt::Display for EloInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank = (self.rank.map(|rank| rank.to_string())).unwrap_or_else(|| "N/A".into());
        let diff = (self.today.elo_diff)
            .map(|diff| format!("{diff:+}"))
            .unwrap_or_else(|| "N/A".into());
        write!(
            f,
            "LEVEL {} | {} elo | no. {} in {} | Today {} elo {}W-{}L",
            self.level, self.elo, rank, self.region, diff, self.today.wins, self.today.losses
        )?;
        if let Some(last) = &self.last_match {
            write!(
                f,
                " | Last Match {}-{} {} {}adr {}kd",
                last.score.0,
                last.score.1,
                if last.won { "W" } else { "L" },
                metric(last.adr),
                metric(last.kd)
            )?
        }
        Ok(())
    }
}

fn metric(value: Option<f64>) -> String {
    value
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "N/A".into())
}

fn missing_elo() -> Error {
    Error::not_found("This player has no CS2 Elo on FACEIT.")
}

#[cfg(test)]
#[path = "../../../tests/endpoints/faceit/elo/model/tests.rs"]
mod tests;
