use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use jiff::{Timestamp, tz::TimeZone};
use tower::ServiceExt;
use wyndis_api::{config::Config, routes, state::State};

#[tokio::test]
async fn age_uses_person_middleware_and_returns_plain_text() {
    let now = Timestamp::now().to_zoned(TimeZone::UTC);
    let birthday = now.strftime("2000-%m-%d");
    let config: Config = toml::from_str(&format!(
        "bind_addr = '127.0.0.1:3000'\n[rate_limit.global]\nrequests_per_minute = 1000\nburst = 100\n[rate_limit.faceit]\nrequests_per_minute = 2\nburst = 1\n[[persons]]\nid = 'owner'\ntwitch_channel = 'streamer'\nbirthday = {birthday}\n[[persons]]\nid = 'missing'\ntwitch_channel = 'missing'\n[[persons]]\nid = 'future'\ntwitch_channel = 'future'\nbirthday = 9999-01-01\n[[persons]]\nid = 'invalid-midnight'\ntwitch_channel = 'invalid_midnight'\nbirthday = 2018-11-04\ntimezone = 'America/Sao_Paulo'"
    )).unwrap();
    let app = routes::compose(State::new(config.persons).unwrap(), config.rate_limit);
    for (header, status, expected) in [
        (
            Some("provider=twitch&name=invalid_midnight"),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to calculate age".into(),
        ),
        (
            Some("provider=twitch&name=STREAMER"),
            StatusCode::OK,
            format!(" 🥳 I am {} 🎂 Today is my 🎈BIRTHDAY🎉", now.year() - 2000),
        ),
        (
            None,
            StatusCode::BAD_REQUEST,
            "Missing Nightbot channel header".into(),
        ),
        (
            Some("invalid"),
            StatusCode::BAD_REQUEST,
            "Invalid Nightbot channel header".into(),
        ),
        (
            Some("provider=youtube&name=streamer"),
            StatusCode::BAD_REQUEST,
            "Invalid Nightbot channel header".into(),
        ),
        (
            Some("provider=twitch&name=unknown"),
            StatusCode::NOT_FOUND,
            "Unknown person".into(),
        ),
        (
            Some("provider=twitch&name=missing"),
            StatusCode::NOT_FOUND,
            "Missing birthday".into(),
        ),
        (
            Some("provider=twitch&name=future"),
            StatusCode::BAD_REQUEST,
            "Birthday is in the future".into(),
        ),
    ] {
        let mut request = Request::builder().uri("/api/age");
        if let Some(header) = header {
            request = request.header("nightbot-channel", header);
        }
        let response = app
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/plain")
        );
        assert_eq!(
            response.into_body().collect().await.unwrap().to_bytes(),
            expected
        );
    }
}
