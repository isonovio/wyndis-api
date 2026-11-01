use axum::{extract::Request, middleware::Next, response::Response};

use crate::sites::nightbot::parse_channel;

pub async fn extract_twitch_channel(mut request: Request, next: Next) -> Response {
    let channel = parse_channel(request.headers());
    request.extensions_mut().insert(channel);
    next.run(request).await
}

#[cfg(test)]
#[path = "../tests/middlewares/twitch_channel/tests.rs"]
mod tests;
