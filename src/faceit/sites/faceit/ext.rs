use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use uuid::Uuid;

use crate::error::Error;

use super::{
    Client,
    models::{History, MatchStats, Region},
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
    pub async fn fetch(client: &Client, id: Uuid, timezone: Tz, now: i64) -> Result<Self, Error> {
        let start = day_start(now, timezone)?;
        let mut today = Today::default();
        for offset in (0..=1000).step_by(100) {
            let history = History::fetch(client, id, start, now, offset, 100).await?;
            today.add_history(&history, id, start, now)?;
            if history.items.len() < 100 {
                return Ok(today);
            }
        }
        Err(Error::bad_gateway(
            "Daily history pagination limit exceeded",
        ))
    }

    fn add_history(
        &mut self,
        history: &History,
        id: Uuid,
        start: i64,
        now: i64,
    ) -> Result<(), Error> {
        for game in &history.items {
            if !game.status.eq_ignore_ascii_case("finished")
                || game.finished_at < start
                || game.finished_at > now
            {
                continue;
            }
            let (_, won, _) = game.result(id)?;
            match won {
                true => self.wins += 1,
                false => self.losses += 1,
            }
        }
        Ok(())
    }
}

impl LastMatch {
    pub async fn fetch(client: &Client, id: Uuid, now: i64) -> Result<Option<Self>, Error> {
        let history = History::fetch(client, id, 0, now, 0, 1).await?;
        let Some(game) = history.items.first() else {
            return Ok(None);
        };
        let (score, won, _) = game.result(id)?;
        let stats = MatchStats::fetch(client, &game.match_id).await?;
        let player = stats.as_ref().and_then(|stats| stats.player(id));
        Ok(Some(Self {
            score,
            won,
            adr: player.and_then(|player| player.metric("ADR")),
            kd: player.and_then(|player| player.metric("K/D Ratio")),
        }))
    }
}

fn day_start(now: i64, timezone: Tz) -> Result<i64, Error> {
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

    #[test]
    fn daily_history_counts_finished_matches_in_the_local_day() {
        let id = Uuid::from_u128(1);
        let now = DateTime::parse_from_rfc3339("2026-07-10T12:00:00Z")
            .unwrap()
            .timestamp();
        let start = day_start(now, chrono_tz::Europe::Berlin).unwrap();
        let items = [
            ("finished", start, "faction1"),
            ("finished", start + 1, "faction2"),
            ("FINISHED", now, "faction1"),
            ("finished", start - 1, "faction1"),
            ("finished", now + 1, "faction1"),
            ("cancelled", start + 1, "faction1"),
            ("ongoing", start + 1, "faction1"),
        ]
        .map(|(status, finished_at, winner)| {
            serde_json::json!({
                "match_id": "match",
                "finished_at": finished_at,
                "status": status,
                "results": {"score": {"faction1": 13, "faction2": 8}, "winner": winner},
                "teams": {
                    "faction1": {"players": [{"player_id": id}]},
                    "faction2": {"players": [{"player_id": Uuid::from_u128(2)}]}
                }
            })
        });
        let history: History = serde_json::from_value(serde_json::json!({"items": items})).unwrap();
        let mut today = Today::default();
        today.add_history(&history, id, start, now).unwrap();
        assert_eq!((today.wins, today.losses), (2, 1));
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
