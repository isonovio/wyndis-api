use std::collections::{HashMap, HashSet};
use std::fmt;

use serde::Deserialize;
use uuid::Uuid;

use crate::error::Error;

use super::Client;

#[derive(Debug, Deserialize)]
pub struct Player {
    pub id: Uuid,
    #[serde(default)]
    pub games: HashMap<String, Game>,
}

#[derive(Debug, Deserialize)]
pub struct Game {
    pub faceit_elo: Option<u32>,
    pub skill_level: Option<u8>,
    pub region: Region,
}

#[derive(Debug, Deserialize)]
pub struct Match {
    #[serde(rename = "matchId")]
    pub id: String,
    #[serde(rename = "matchRound")]
    pub round: String,
    pub date: i64,
    pub status: String,
    pub elo: Option<String>,
    pub elo_delta: Option<String>,
    #[serde(rename = "i10")]
    pub result: String,
    #[serde(rename = "c5")]
    pub team_score: String,
    #[serde(rename = "i18")]
    pub score: String,
    #[serde(rename = "c10")]
    pub adr: Option<String>,
    #[serde(rename = "c2")]
    pub kd: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub struct FaceitNickname(String);

impl TryFrom<String> for FaceitNickname {
    type Error = &'static str;

    fn try_from(nickname: String) -> Result<Self, Self::Error> {
        if nickname.is_empty()
            || nickname.len() > 64
            || nickname
                .chars()
                .any(|c| c.is_whitespace() || c.is_control())
        {
            return Err("expected a single FACEIT nickname");
        }
        Ok(Self(nickname))
    }
}

impl fmt::Display for FaceitNickname {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(try_from = "String")]
pub struct Region(String);

impl TryFrom<String> for Region {
    type Error = &'static str;

    fn try_from(region: String) -> Result<Self, Self::Error> {
        if region.is_empty() || !region.bytes().all(|c| c.is_ascii_alphanumeric()) {
            return Err("expected a FACEIT region");
        }
        Ok(Self(region.to_ascii_uppercase()))
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Player {
    pub async fn fetch(client: &Client, nickname: &FaceitNickname) -> Result<Self, Error> {
        let mut path = url::Url::parse("https://api.faceit.com/users/v1/nicknames/")
            .map_err(|_| invalid_data())?;
        path.path_segments_mut()
            .map_err(|_| invalid_data())?
            .push(&nickname.to_string());
        client.fetch(path.path().trim_start_matches('/'), &[]).await
    }
}

impl Match {
    pub async fn fetch(
        client: &Client,
        id: Uuid,
        start: i64,
        now: i64,
    ) -> Result<Vec<Self>, Error> {
        let mut matches = Vec::new();
        let mut seen = HashSet::new();
        for page in 0..=33 {
            let history: Vec<Self> = client
                .fetch(
                    &format!("stats/v1/stats/time/users/{id}/games/cs2"),
                    &[("page", page.to_string()), ("size", "30".into())],
                )
                .await?;
            let done = history.len() < 30
                || history
                    .iter()
                    .any(|game| game.date.div_euclid(1000) < start);
            let mut added = false;
            for game in history {
                if game.status == "APPLIED"
                    && game.date.div_euclid(1000) <= now
                    && seen.insert((game.id.clone(), game.round.clone()))
                {
                    matches.push(game);
                    added = true;
                }
            }
            if done {
                matches.sort_by_key(|game| std::cmp::Reverse(game.date));
                return Ok(matches);
            }
            if !added {
                break;
            }
        }
        Err(Error::bad_gateway(
            "Daily history pagination limit exceeded",
        ))
    }

    pub fn won(&self) -> Result<bool, Error> {
        match self.result.as_str() {
            "1" => Ok(true),
            "0" => Ok(false),
            _ => Err(invalid_data()),
        }
    }

    pub fn score(&self) -> Result<(u32, u32), Error> {
        let (first, second) = self.score.split_once('/').ok_or_else(invalid_data)?;
        let first: u32 = first.trim().parse().map_err(|_| invalid_data())?;
        let second: u32 = second.trim().parse().map_err(|_| invalid_data())?;
        let own: u32 = self.team_score.parse().map_err(|_| invalid_data())?;
        match own {
            value if value == first => Ok((own, second)),
            value if value == second => Ok((own, first)),
            _ => Err(invalid_data()),
        }
    }
}

pub(super) fn invalid_data() -> Error {
    Error::bad_gateway("Incomplete match data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_is_oriented_using_the_players_team_score() {
        let mut game: Match = serde_json::from_value(serde_json::json!({
            "matchId": "match", "matchRound": "1", "date": 1791057410000_i64,
            "status": "APPLIED", "i10": "1", "c5": "13", "i18": "8 / 13"
        }))
        .unwrap();
        assert_eq!(game.score().unwrap(), (13, 8));
        assert!(game.won().unwrap());
        game.team_score = "8".into();
        game.result = "0".into();
        assert_eq!(game.score().unwrap(), (8, 13));
        assert!(!game.won().unwrap());
        game.team_score = "7".into();
        assert!(game.score().is_err());
    }
}
