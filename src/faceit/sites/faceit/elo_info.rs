use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::error::Error;

use super::{
    Client,
    ext::{LastMatch, Ranking, Today, day_start},
    models::{FaceitNickname, Match, Player, Region},
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
    pub async fn fetch(client: &Client, nickname: &FaceitNickname) -> Result<Self, Error> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::internal_server_error("Invalid server time."))?
            .as_secs() as i64;
        Self::fetch_at(client, nickname, now).await
    }

    pub async fn fetch_at(
        client: &Client,
        nickname: &FaceitNickname,
        now: i64,
    ) -> Result<Self, Error> {
        let player = Player::fetch(client, nickname).await?;
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
            "LEVEL {} | {} elo | no. {} in {} | Today {} elo {}W {}L | Last Match ",
            self.level, self.elo, rank, self.region, diff, self.today.wins, self.today.losses
        )?;
        match &self.last_match {
            Some(last) => write!(
                f,
                "{}-{} {} {}adr {}kd",
                last.score.0,
                last.score.1,
                if last.won { "W" } else { "L" },
                metric(last.adr),
                metric(last.kd)
            ),
            None => f.write_str("N/A"),
        }
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
mod tests {
    use super::*;

    #[test]
    fn display_matches_chat_format() {
        let info = EloInfo {
            level: 10,
            elo: 2345,
            rank: Some(1234),
            region: "EU".to_owned().try_into().unwrap(),
            today: Today {
                elo_diff: Some(25),
                wins: 2,
                losses: 1,
            },
            last_match: Some(LastMatch {
                score: (13, 8),
                won: true,
                adr: Some(92.5),
                kd: Some(1.5),
            }),
        };
        assert_eq!(
            info.to_string(),
            "LEVEL 10 | 2345 elo | no. 1234 in EU | Today +25 elo 2W 1L | Last Match 13-8 W 92.50adr 1.50kd"
        );
    }

    #[test]
    fn missing_metrics_are_explicit() {
        let info = EloInfo {
            level: 1,
            elo: 100,
            rank: None,
            region: "NA".to_owned().try_into().unwrap(),
            today: Today::default(),
            last_match: None,
        };
        assert_eq!(
            info.to_string(),
            "LEVEL 1 | 100 elo | no. N/A in NA | Today N/A elo 0W 0L | Last Match N/A"
        );
    }
}
