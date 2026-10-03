use std::time::Duration;

use axum::http::StatusCode;
use reqwest::{Client as HttpClient, Url, header};
use serde::de::DeserializeOwned;

use crate::error::Error;

#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    base_url: Url,
}

impl Client {
    pub fn new(api_key: &str) -> anyhow::Result<Self> {
        Self::with_base_url(api_key, Url::parse("https://open.faceit.com/data/v4/")?)
    }

    pub fn with_base_url(api_key: &str, base_url: Url) -> anyhow::Result<Self> {
        let mut authorization = header::HeaderValue::from_str(&format!("Bearer {api_key}"))?;
        authorization.set_sensitive(true);
        let mut headers = header::HeaderMap::new();
        headers.insert(header::AUTHORIZATION, authorization);
        let http = HttpClient::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(3))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self { http, base_url })
    }

    pub async fn fetch<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<T, Error> {
        let url = self.base_url.join(path).map_err(|_| unavailable())?;
        if url.origin() != self.base_url.origin() {
            return Err(unavailable());
        }
        let response = self
            .http
            .get(url)
            .query(query)
            .send()
            .await
            .map_err(|error| {
                tracing::warn!(timeout = error.is_timeout(), "FACEIT request failed");
                match error.is_timeout() {
                    true => Error::gateway_timeout("FACEIT request timed out"),
                    false => unavailable(),
                }
            })?;
        match response.status() {
            StatusCode::OK => response
                .json()
                .await
                .map_err(|_| Error::bad_gateway("Invalid FACEIT response")),
            StatusCode::NOT_FOUND => Err(Error::not_found("FACEIT resource not found")),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(Error::service_unavailable("FACEIT rate limit exceeded"))
            }
            status => {
                tracing::warn!(%status, "FACEIT returned an error");
                Err(unavailable())
            }
        }
    }
}

fn unavailable() -> Error {
    Error::bad_gateway("FACEIT unavailable")
}
