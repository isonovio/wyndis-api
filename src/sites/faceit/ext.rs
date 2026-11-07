use axum::http::StatusCode;
use jiff::{
    Timestamp,
    tz::{AmbiguousOffset, TimeZone},
};
use uuid::Uuid;

use crate::error::Error;

use super::{
    client::Client,
    models::{Match, Region, invalid_data},
};

#[derive(Debug, Default)]
pub struct Today {
    pub elo_diff: Option<i32>,
    pub wins: u32,
    pub losses: u32,
}

#[derive(Debug)]
pub struct LastMatch {
    pub score: (u32, u32),
    pub won: bool,
    pub adr: Option<f64>,
    pub kd: Option<f64>,
}

pub struct Ranking {
    pub position: Option<u32>,
}

impl Ranking {
    pub async fn fetch(client: &Client, id: Uuid, region: &Region) -> Result<Self, Error> {
        match client
            .fetch::<u32>(&format!("ranking/v1/globalranking/cs2/{region}/{id}"), &[])
            .await
        {
            Ok(position) => Ok(Self {
                position: Some(position).filter(|position| *position > 0),
            }),
            Err(error) if error.status == StatusCode::NOT_FOUND => Ok(Self { position: None }),
            Err(error) => Err(error),
        }
    }
}

impl Region {
    pub fn timezone(&self) -> Option<TimeZone> {
        let name = match self.to_string().as_str() {
            "NA" => "America/New_York",
            "SA" => "America/Sao_Paulo",
            "SEA" => "Asia/Singapore",
            "OCE" => "Australia/Sydney",
            _ => "Europe/Berlin",
        };
        TimeZone::get(name).ok()
    }
}

impl Today {
    pub fn from_matches(
        matches: &[Match],
        start: Timestamp,
        now: Timestamp,
    ) -> Result<Self, Error> {
        let mut today = Self {
            elo_diff: Some(0),
            ..Self::default()
        };
        for game in matches.iter().filter(|game| {
            let finished = game.date;
            game.status == "APPLIED" && finished >= start && finished <= now
        }) {
            match game.won()? {
                true => today.wins += 1,
                false => today.losses += 1,
            }
            match &game.elo_delta {
                Some(delta) => {
                    let delta: i32 = delta.parse().map_err(|_| invalid_data())?;
                    if let Some(total) = today.elo_diff {
                        today.elo_diff = Some(total.checked_add(delta).ok_or_else(invalid_data)?);
                    }
                }
                None if game.elo.is_some() => today.elo_diff = None,
                None => {}
            }
        }
        Ok(today)
    }
}

impl LastMatch {
    pub fn from_match(game: &Match) -> Result<Self, Error> {
        Ok(Self {
            score: game.score()?,
            won: game.won()?,
            adr: metric(game.adr.as_deref()),
            kd: metric(game.kd.as_deref()),
        })
    }
}

fn metric(value: Option<&str>) -> Option<f64> {
    value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
}

pub(crate) fn day_start(now: Timestamp, timezone: TimeZone) -> Option<Timestamp> {
    let date = now.to_zoned(timezone.clone()).date();
    let midnight = timezone.to_ambiguous_timestamp(date.at(0, 0, 0, 0));
    match midnight.offset() {
        AmbiguousOffset::Gap { .. } => None,
        _ => midnight.earlier().ok(),
    }
}

#[cfg(test)]
#[path = "../../tests/sites/faceit/ext/tests.rs"]
mod tests;
