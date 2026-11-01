use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use chrono::Utc;
use http_body_util::BodyExt;
use tower::ServiceExt;
use wyndis_api::{config::Config, routes, state::State};

#[tokio::test]
async fn age_uses_person_middleware_and_returns_plain_text() {
    let now = Utc::now();
    let birthday = now.format("2000-%m-%d");
    let config: Config = toml::from_str(&format!(
        "bind_addr = '127.0.0.1:3000'\n[rate_limit.age]\nrequests_per_minute = 1000\nburst = 100\n[[persons]]\nid = 'owner'\ntwitch_channel = 'streamer'\nbirthday = {birthday}\n[[persons]]\nid = 'missing'\ntwitch_channel = 'missing'\n[[persons]]\nid = 'future'\ntwitch_channel = 'future'\nbirthday = 9999-01-01"
    )).unwrap();
    let app = routes::compose(State::new(config.persons).unwrap(), config.rate_limit);
    for (header, status, expected) in [
        (
            Some("provider=twitch&name=STREAMER"),
            StatusCode::OK,
            format!(
                " 🥳 I am {} 🎂 Today is my 🎈BIRTHDAY🎉",
                chrono::Datelike::year(&now) - 2000
            ),
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
