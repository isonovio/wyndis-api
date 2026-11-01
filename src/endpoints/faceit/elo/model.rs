use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::error::Error;

use crate::sites::faceit::{
    Client,
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
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::internal_server_error("Invalid server time."))?
            .as_secs() as i64;
        Self::fetch_at(client, player, now).await
    }

    pub async fn fetch_at(client: &Client, player: &Player, now: i64) -> Result<Self, Error> {
        let id = player.id;
        let game = player.games.get("cs2").ok_or_else(missing_elo)?;
        let elo = game.faceit_elo.ok_or_else(missing_elo)?;
        let level = game.skill_level.ok_or_else(missing_elo)?;
        let start = day_start(now, game.region.timezone())?;
        let (rank, matches) = tokio::try_join!(
            Ranking::fetch(client, id, &game.region),
            Match::fetch(client, id, start, now),
        )?;
        Ok(Self {
            level,
            elo,
            rank: rank.position,
            region: game.region.clone(),
            today: Today::from_matches(&matches, start, now)?,
            last_match: matches.first().map(LastMatch::from_match).transpose()?,
        })
    }
}

impl fmt::Display for EloInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank = self
            .rank
            .map(|rank| rank.to_string())
            .unwrap_or_else(|| "N/A".into());
        let diff = self
            .today
            .elo_diff
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
