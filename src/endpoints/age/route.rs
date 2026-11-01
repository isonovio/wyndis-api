use axum::{Extension, Router, routing::get};
use chrono::Utc;

use crate::error::Error;
use crate::middlewares::person::ResolvedPerson;

use super::model::Age;

pub fn init() -> Router {
    Router::new().route("/age", get(age))
}

async fn age(Extension(person): Extension<ResolvedPerson>) -> Result<String, Error> {
    Ok(Age::at(&person?, Utc::now())?.to_string())
}
