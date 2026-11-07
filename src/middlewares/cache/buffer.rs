use axum::{
    body::{Body, Bytes, to_bytes},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

use super::kv::CachedValue;

#[derive(Clone)]
pub struct BufferedResponse {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl BufferedResponse {
    pub async fn read(response: Response) -> Self {
        let (parts, body) = response.into_parts();
        match to_bytes(body, 64 * 1024).await {
            Ok(body) => Self {
                status: parts.status,
                headers: parts.headers,
                body,
            },
            Err(_) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                headers: HeaderMap::new(),
                body: Bytes::from_static(b"Unable to buffer response"),
            },
        }
    }
}

impl CachedValue for BufferedResponse {
    async fn from_response(response: Response) -> Result<Self, Self> {
        let response = Self::read(response).await;
        match response.status == StatusCode::OK {
            true => Ok(response),
            false => Err(response),
        }
    }
}

impl IntoResponse for BufferedResponse {
    fn into_response(self) -> Response {
        let mut response = Response::new(Body::from(self.body));
        *response.status_mut() = self.status;
        *response.headers_mut() = self.headers;
        response
    }
}
