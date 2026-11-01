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
        "LEVEL 10 | 2345 elo | no. 1234 in EU | Today +25 elo 2W-1L | Last Match 13-8 W 92.50adr 1.50kd"
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
        "LEVEL 1 | 100 elo | no. N/A in NA | Today N/A elo 0W-0L"
    );
}
