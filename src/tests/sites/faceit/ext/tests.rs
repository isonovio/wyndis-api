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
