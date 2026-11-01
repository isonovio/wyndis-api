use axum::{Extension, Router, routing::get};
use jiff::{Timestamp, tz::TimeZone};

use crate::error::Error;
use crate::middlewares::person::ResolvedPerson;

use super::model::Age;

pub fn init() -> Router {
    Router::new().route("/age", get(age))
}

async fn age(Extension(person): Extension<ResolvedPerson>) -> Result<String, Error> {
    let person = person?;
    let birthday = person
        .birthday
        .ok_or_else(|| Error::not_found("Missing birthday"))?;
    let tz = person.timezone.unwrap_or(TimeZone::UTC);
    let now = Timestamp::now();
    if birthday > now.to_zoned(tz.clone()).date() {
        return Err(Error::bad_request("Birthday is in the future"));
    }

    let message = Age::at(birthday, now, tz)
        .and_then(|age| age.message())
        .ok_or(Error::internal_server_error("Unable to calculate age"))?;
    Ok(message.to_string())
}
