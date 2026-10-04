use std::collections::HashMap;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;
use wyndis_api::{
    faceit::{State, sites::faceit::Client},
    middlewares::rate_limit,
    routes::compose,
};

const OWNER: &str = "channel_player";
const OTHER: &str = "donk666";

fn app(state: State) -> Router {
    compose(
        state,
        rate_limit::Config {
            requests_per_minute: std::num::NonZeroU32::new(1000).unwrap(),
            burst: std::num::NonZeroU32::new(100).unwrap(),
        },
    )
}

struct MockServer(tokio::task::JoinHandle<()>);

impl Drop for MockServer {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn fixture(
    status: StatusCode,
    mut body: serde_json::Value,
    expected_nickname: &str,
) -> (Router, MockServer) {
    let expected_nickname = expected_nickname.to_owned();
    body["id"] = json!("00000000-0000-0000-0000-000000000001");
    let upstream = Router::new()
        .route(
            "/users/v1/nicknames/{nickname}",
            get(
                move |axum::extract::Path(nickname): axum::extract::Path<String>| async move {
                    assert_eq!(nickname, expected_nickname);
                    (status, axum::Json(json!({"payload": body})))
                },
            ),
        )
        .route(
            "/ranking/v1/globalranking/cs2/EU/{id}",
            get(|| async { axum::Json(json!({"payload": 1234})) }),
        )
        .route(
            "/stats/v1/stats/time/users/{id}/games/cs2",
            get(|| async { axum::Json(json!([])) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, upstream).await.unwrap();
    });
    let client = Client::with_base_url(url.parse().unwrap()).unwrap();
    let channels = HashMap::from([(
        "Streamer".to_owned().try_into().unwrap(),
        OWNER.to_owned().try_into().unwrap(),
    )]);
    (app(State::with_client(client, channels)), MockServer(task))
}

async fn request(app: Router, uri: &str, channel: Option<&str>) -> (StatusCode, String) {
    let mut builder = Request::builder().uri(uri);
    if let Some(channel) = channel {
        builder = builder.header("nightbot-channel", channel);
    }
    let response = app
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(
        response.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/plain")
    );
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

fn player() -> serde_json::Value {
    json!({"nickname": "Player", "games": {"cs2": {"faceit_elo": 2345, "skill_level": 10, "region": "EU"}}})
}

#[tokio::test]
async fn explicit_nickname_overrides_channel_mapping() {
    let (app, _mock) = fixture(StatusCode::OK, player(), OTHER).await;
    let (status, body) = request(
        app,
        &format!("/api/faceit/elo?id={OTHER}"),
        Some("provider=twitch&name=streamer&providerId=123"),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        "LEVEL 10 | 2345 elo | no. 1234 in EU | Today +0 elo 0W 0L | Last Match N/A"
    );
}

#[tokio::test]
async fn omitted_or_blank_nickname_uses_channel_owner() {
    let (app, _mock) = fixture(StatusCode::OK, player(), OWNER).await;
    for uri in [
        "/api/faceit/elo",
        "/api/faceit/elo?id=",
        "/api/faceit/elo?id=%20",
    ] {
        let (status, _) = request(
            app.clone(),
            uri,
            Some("name=STREAMER&displayName=Streamer&provider=twitch&providerId=123"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }
}

#[tokio::test]
async fn explicit_nickname_does_not_require_nightbot_headers() {
    let (app, _mock) = fixture(StatusCode::OK, player(), OTHER).await;
    assert_eq!(
        request(app, &format!("/api/faceit/elo?id={OTHER}"), None)
            .await
            .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn invalid_input_and_unknown_channel_are_handled() {
    // Rejected requests never reach FACEIT, so this test needs no listening socket.
    let client = Client::new().unwrap();
    let channels = HashMap::from([(
        "Streamer".to_owned().try_into().unwrap(),
        OWNER.to_owned().try_into().unwrap(),
    )]);
    let app = app(State::with_client(client, channels));
    for (uri, header, expected) in [
        (
            "/api/faceit/elo?id=two%20names",
            None,
            StatusCode::BAD_REQUEST,
        ),
        ("/api/faceit/elo", None, StatusCode::BAD_REQUEST),
        (
            "/api/faceit/elo",
            Some("provider=youtube&name=streamer"),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/faceit/elo",
            Some("provider=twitch"),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/faceit/elo",
            Some("provider=twitch&name=invalid%20name"),
            StatusCode::BAD_REQUEST,
        ),
        (
            "/api/faceit/elo",
            Some("provider=twitch&name=unknown_channel"),
            StatusCode::NOT_FOUND,
        ),
        ("/api/faceit/elo?id=a&id=b", None, StatusCode::BAD_REQUEST),
    ] {
        assert_eq!(request(app.clone(), uri, header).await.0, expected);
    }
}

#[tokio::test]
async fn missing_cs2_elo_is_not_reported_as_zero() {
    for body in [
        json!({"nickname": "Player", "games": {}}),
        json!({"nickname": "Player", "games": {"cs2": {"skill_level": 1, "region": "EU"}}}),
    ] {
        let (app, _mock) = fixture(StatusCode::OK, body, OWNER).await;
        let (status, body) = request(app, &format!("/api/faceit/elo?id={OWNER}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body, "This player has no CS2 Elo on FACEIT.");
    }
}

#[tokio::test]
async fn upstream_errors_are_safe_plain_text() {
    for (upstream, expected) in [
        (StatusCode::NOT_FOUND, StatusCode::NOT_FOUND),
        (
            StatusCode::TOO_MANY_REQUESTS,
            StatusCode::SERVICE_UNAVAILABLE,
        ),
        (StatusCode::UNAUTHORIZED, StatusCode::BAD_GATEWAY),
        (StatusCode::INTERNAL_SERVER_ERROR, StatusCode::BAD_GATEWAY),
        (StatusCode::OK, StatusCode::BAD_GATEWAY), // Invalid player response.
    ] {
        let (app, _mock) = fixture(
            upstream,
            json!({"secret": "upstream details", "games": null}),
            OWNER,
        )
        .await;
        let (status, body) = request(app, &format!("/api/faceit/elo?id={OWNER}"), None).await;
        assert_eq!(status, expected);
        assert!(!body.contains("secret"));
    }
}
