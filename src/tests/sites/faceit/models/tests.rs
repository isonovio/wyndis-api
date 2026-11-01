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
