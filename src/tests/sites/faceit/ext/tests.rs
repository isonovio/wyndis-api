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
    let now = Timestamp::from_second(1791060120).unwrap();
    let start = day_start(now, TimeZone::get("Europe/Berlin").unwrap()).unwrap();
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
        Today::from_matches(&games, Timestamp::UNIX_EPOCH, Timestamp::MAX)
            .unwrap()
            .elo_diff,
        None
    );
    games[0].elo = None;
    assert_eq!(
        Today::from_matches(&games, Timestamp::UNIX_EPOCH, Timestamp::MAX)
            .unwrap()
            .elo_diff,
        Some(0)
    );
    let empty = Today::from_matches(
        &games,
        Timestamp::from_second(1791060034).unwrap(),
        Timestamp::from_second(1791074010).unwrap(),
    )
    .unwrap();
    assert_eq!((empty.elo_diff, empty.wins, empty.losses), (Some(0), 0, 0));
    games[0].elo_delta = Some("bad".into());
    assert!(Today::from_matches(&games, Timestamp::UNIX_EPOCH, Timestamp::MAX).is_err());
}

#[test]
fn day_boundary_handles_central_european_summer_time() {
    let winter = "2026-01-10T12:00:00Z".parse::<Timestamp>().unwrap();
    let summer = "2026-07-10T12:00:00Z".parse::<Timestamp>().unwrap();
    assert_eq!(
        day_start(winter, TimeZone::get("Europe/Berlin").unwrap()).unwrap(),
        "2026-01-09T23:00:00Z".parse::<Timestamp>().unwrap()
    );
    assert_eq!(
        day_start(summer, TimeZone::get("Europe/Berlin").unwrap()).unwrap(),
        "2026-07-09T22:00:00Z".parse::<Timestamp>().unwrap()
    );
}

#[test]
fn day_boundary_respects_regional_timezone() {
    let now = "2026-07-10T00:30:00Z".parse::<Timestamp>().unwrap();
    assert_eq!(
        day_start(now, TimeZone::get("America/New_York").unwrap()).unwrap(),
        "2026-07-09T04:00:00Z".parse::<Timestamp>().unwrap()
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
        assert_eq!(region.timezone().unwrap().iana_name(), Some(expected));
    }
}

#[test]
fn regional_midnight_rejects_gaps_and_chooses_the_earliest_fold() {
    assert!(
        day_start(
            "2018-11-04T12:00:00Z".parse().unwrap(),
            TimeZone::get("America/Sao_Paulo").unwrap(),
        )
        .is_none()
    );
    assert_eq!(
        day_start(
            "2020-11-01T12:00:00Z".parse().unwrap(),
            TimeZone::get("America/Havana").unwrap(),
        )
        .unwrap(),
        "2020-11-01T04:00:00Z".parse::<Timestamp>().unwrap()
    );
}

#[test]
fn regional_midnight_tracks_dst_transition_days() {
    for (now, expected) in [
        ("2026-03-08T16:00:00Z", "2026-03-08T05:00:00Z"),
        ("2026-11-01T16:00:00Z", "2026-11-01T04:00:00Z"),
    ] {
        assert_eq!(
            day_start(
                now.parse().unwrap(),
                TimeZone::get("America/New_York").unwrap()
            )
            .unwrap(),
            expected.parse::<Timestamp>().unwrap()
        );
    }
}

#[test]
fn daily_window_includes_its_boundaries_without_discarding_milliseconds() {
    let start = "2026-10-04T00:00:00Z".parse().unwrap();
    let now = "2026-10-04T12:00:00.500Z".parse().unwrap();
    let mut games: Vec<_> = history().into_iter().chain(history()).take(5).collect();
    for (game, date) in games.iter_mut().zip([
        "2026-10-03T23:59:59.999Z",
        "2026-10-04T00:00:00Z",
        "2026-10-04T12:00:00.500Z",
        "2026-10-04T12:00:00.501Z",
        "2026-10-04T06:00:00Z",
    ]) {
        game.date = date.parse().unwrap();
        game.result = "1".into();
        game.elo_delta = Some("1".into());
    }
    games[4].status = "PENDING".into();
    let today = Today::from_matches(&games, start, now).unwrap();
    assert_eq!((today.elo_diff, today.wins, today.losses), (Some(2), 2, 0));
}
