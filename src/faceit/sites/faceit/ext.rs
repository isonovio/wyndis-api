use axum::http::StatusCode;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::error::Error;

use super::{
    Client,
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
    pub fn timezone(&self) -> Tz {
        match self.to_string().as_str() {
            "NA" => chrono_tz::America::New_York,
            "SA" => chrono_tz::America::Sao_Paulo,
            "SEA" => chrono_tz::Asia::Singapore,
            "OCE" => chrono_tz::Australia::Sydney,
            _ => chrono_tz::Europe::Berlin,
        }
    }
}

impl Today {
    pub fn from_matches(matches: &[Match], start: i64, now: i64) -> Result<Self, Error> {
        let mut today = Self {
            elo_diff: Some(0),
            ..Self::default()
        };
        for game in matches.iter().filter(|game| {
            let finished = game.date.div_euclid(1000);
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

pub(super) fn day_start(now: i64, timezone: Tz) -> Result<i64, Error> {
    let local = DateTime::<Utc>::from_timestamp(now, 0)
        .ok_or_else(|| Error::bad_gateway("Invalid timestamp"))?
        .with_timezone(&timezone);
    let midnight = local
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| Error::bad_gateway("Invalid midnight"))?;
    timezone
        .from_local_datetime(&midnight)
        .earliest()
        .map(|date| date.timestamp())
        .ok_or_else(|| Error::bad_request("Missing regional midnight"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> Vec<Match> {
        serde_json::from_value(serde_json::json!([
            {"matchId":"last", "matchRound":"1", "date":1791060033000_i64, "status":"APPLIED",
             "elo":"3291", "elo_delta":"-1", "i10":"0", "c5":"11", "i18":"11 / 13", "c10":"83.3", "c2":"0.95"},
            {"matchId":"middle", "matchRound":"1", "date":1791057410000_i64, "status":"APPLIED",
             "elo":"3292", "elo_delta":"1", "i10":"1", "c5":"13", "i18":"8 / 13"},
            {"matchId":"first", "matchRound":"1", "date":1791055071000_i64, "status":"APPLIED",
             "elo":"3291", "elo_delta":"-1", "i10":"0", "c5":"11", "i18":"11 / 13"}
        ])).unwrap()
    }

    #[test]
    fn history_produces_daily_elo_and_last_match() {
        let now = 1791060120;
        let start = day_start(now, chrono_tz::Europe::Berlin).unwrap();
        let games = history();
        let today = Today::from_matches(&games, start, now).unwrap();
        assert_eq!((today.elo_diff, today.wins, today.losses), (Some(-1), 1, 2));
        let last = LastMatch::from_match(&games[0]).unwrap();
        assert_eq!(
            (last.score, last.won, last.adr, last.kd),
            ((11, 13), false, Some(83.3), Some(0.95))
        );
    }

    #[test]
    fn missing_adjustments_and_empty_days_are_distinct() {
        let mut games = history();
        games[0].elo_delta = None;
        assert_eq!(
            Today::from_matches(&games, 0, i64::MAX).unwrap().elo_diff,
            None
        );
        games[0].elo = None;
        assert_eq!(
            Today::from_matches(&games, 0, i64::MAX).unwrap().elo_diff,
            Some(0)
        );
        let empty = Today::from_matches(&games, 1791060034, 1791074010).unwrap();
        assert_eq!((empty.elo_diff, empty.wins, empty.losses), (Some(0), 0, 0));
        games[0].elo_delta = Some("bad".into());
        assert!(Today::from_matches(&games, 0, i64::MAX).is_err());
    }

    #[test]
    fn day_boundary_handles_central_european_summer_time() {
        let winter = DateTime::parse_from_rfc3339("2026-01-10T12:00:00Z")
            .unwrap()
            .timestamp();
        let summer = DateTime::parse_from_rfc3339("2026-07-10T12:00:00Z")
            .unwrap()
            .timestamp();
        assert_eq!(
            day_start(winter, chrono_tz::Europe::Berlin).unwrap(),
            DateTime::parse_from_rfc3339("2026-01-09T23:00:00Z")
                .unwrap()
                .timestamp()
        );
        assert_eq!(
            day_start(summer, chrono_tz::Europe::Berlin).unwrap(),
            DateTime::parse_from_rfc3339("2026-07-09T22:00:00Z")
                .unwrap()
                .timestamp()
        );
    }

    #[test]
    fn day_boundary_respects_regional_timezone() {
        let now = DateTime::parse_from_rfc3339("2026-07-10T00:30:00Z")
            .unwrap()
            .timestamp();
        assert_eq!(
            day_start(now, chrono_tz::America::New_York).unwrap(),
            DateTime::parse_from_rfc3339("2026-07-09T04:00:00Z")
                .unwrap()
                .timestamp()
        );
    }

    #[test]
    fn regions_have_simple_timezone_defaults() {
        for (region, expected) in [
            ("EU", "Europe/Berlin"),
            ("na", "America/New_York"),
            ("SA", "America/Sao_Paulo"),
            ("SEA", "Asia/Singapore"),
            ("OCE", "Australia/Sydney"),
            ("unknown", "Europe/Berlin"),
        ] {
            let region = Region::try_from(region.to_owned()).unwrap();
            assert_eq!(region.timezone().name(), expected);
        }
    }
}
