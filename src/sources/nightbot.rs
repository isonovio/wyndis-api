use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};

use tracing::warn;

use crate::error::Error;
use crate::sites::nightbot::parse_channel;

pub async fn extract_twitch_channel(mut request: Request, next: Next) -> Result<Response, Error> {
    let channel = parse_channel(request.headers())
        .inspect_err(|error| warn!(%error, "Nightbot channel rejected"))?;
    request.extensions_mut().insert(channel);
    Ok(next.run(request).await)
}

pub async fn silence_errors(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    match response.status().is_client_error() || response.status().is_server_error() {
        true => " ".into_response(),
        false => response,
    }
}

#[cfg(test)]
#[path = "../tests/sources/nightbot/tests.rs"]
mod tests;
