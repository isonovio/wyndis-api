use std::{collections::HashMap, fmt};

use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::error::Error;

use super::Client;

#[derive(Debug, Deserialize)]
pub struct Player {
    pub player_id: Uuid,
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
pub struct Ranking {
    pub position: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct History {
    pub items: Vec<Match>,
}

#[derive(Debug, Deserialize)]
pub struct Match {
    pub match_id: String,
    pub finished_at: i64,
    pub status: String,
    pub results: MatchResults,
    pub teams: HashMap<String, HistoryTeam>,
}

#[derive(Debug, Deserialize)]
pub struct MatchResults {
    pub score: HashMap<String, u32>,
    pub winner: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryTeam {
    pub players: Vec<HistoryPlayer>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryPlayer {
    pub player_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct MatchStats {
    pub rounds: Vec<RoundStats>,
}

#[derive(Debug, Deserialize)]
pub struct RoundStats {
    pub teams: Vec<StatsTeam>,
}

#[derive(Debug, Deserialize)]
pub struct StatsTeam {
    pub players: Vec<StatsPlayer>,
}

#[derive(Debug, Deserialize)]
pub struct StatsPlayer {
    pub player_id: Uuid,
    pub player_stats: HashMap<String, String>,
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
        client
            .fetch("players", &[("nickname", nickname.to_string())])
            .await
    }
}

impl Ranking {
    pub async fn fetch(client: &Client, id: Uuid, region: &Region) -> Result<Self, Error> {
        let result: Result<Self, _> = client
            .fetch(
                &format!("rankings/games/cs2/regions/{region}/players/{id}"),
                &[],
            )
            .await;
        match result {
            Ok(mut ranking) => {
                ranking.position = ranking.position.filter(|position| *position > 0);
                Ok(ranking)
            }
            Err(error) if error.status == StatusCode::NOT_FOUND => Ok(Self { position: None }),
            Err(error) => Err(error),
        }
    }
}

impl History {
    pub async fn fetch(
        client: &Client,
        id: Uuid,
        from: i64,
        to: i64,
        offset: u32,
        limit: u32,
    ) -> Result<Self, Error> {
        client
            .fetch(
                &format!("players/{id}/history"),
                &[
                    ("game", "cs2".into()),
                    ("from", from.to_string()),
                    ("to", to.to_string()),
                    ("offset", offset.to_string()),
                    ("limit", limit.to_string()),
                ],
            )
            .await
    }
}

impl MatchStats {
    pub async fn fetch(client: &Client, match_id: &str) -> Result<Option<Self>, Error> {
        match client
            .fetch(&format!("matches/{match_id}/stats"), &[])
            .await
        {
            Ok(stats) => Ok(Some(stats)),
            Err(error) if error.status == StatusCode::NOT_FOUND => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn player(&self, id: Uuid) -> Option<&StatsPlayer> {
        self.rounds.last().and_then(|round| {
            round
                .teams
                .iter()
                .flat_map(|team| &team.players)
                .find(|player| player.player_id == id)
        })
    }
}

impl StatsPlayer {
    pub fn metric(&self, key: &str) -> Option<f64> {
        self.player_stats
            .get(key)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite() && *value >= 0.0)
    }
}

impl Match {
    pub fn result(&self, id: Uuid) -> Result<((u32, u32), bool, &str), Error> {
        let team = self
            .teams
            .iter()
            .find(|(_, team)| team.players.iter().any(|player| player.player_id == id))
            .map(|(team, _)| team)
            .ok_or_else(invalid_data)?;
        let opponent = self
            .teams
            .keys()
            .find(|key| *key != team)
            .ok_or_else(invalid_data)?;
        if !self.teams.contains_key(&self.results.winner) {
            return Err(invalid_data());
        }
        let own = *self.results.score.get(team).ok_or_else(invalid_data)?;
        let other = *self.results.score.get(opponent).ok_or_else(invalid_data)?;
        Ok(((own, other), self.results.winner == *team, team))
    }
}

pub(super) fn invalid_data() -> Error {
    Error::bad_gateway("Incomplete match data")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scores_and_winner_are_from_players_perspective() {
        let game: Match = serde_json::from_value(serde_json::json!({
            "match_id": "match", "finished_at": 10, "status": "FINISHED",
            "results": {"score": {"faction1": 13, "faction2": 8}, "winner": "faction1"},
            "teams": {"faction1": {"players": [{"player_id": "00000000-0000-0000-0000-000000000001"}]},
                "faction2": {"players": [{"player_id": "00000000-0000-0000-0000-000000000002"}]}}
        })).unwrap();
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
        assert_eq!(game.result(id).unwrap(), ((8, 13), false, "faction2"));
    }
}
