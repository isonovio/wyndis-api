use axum::{Extension, Router, routing::get};
use jiff::{Timestamp, tz::TimeZone};
use tracing::{error, warn};

use crate::error::Error;
use crate::models::Person;

use super::model::Age;

pub fn init() -> Router {
    Router::new().route("/age", get(age))
}

async fn age(Extension(person): Extension<Person>) -> Result<String, Error> {
    let birthday = (person.birthday)
        .ok_or_else(|| warn!(person = %person.id, "Missing birthday"))
        .map_err(|_| Error::not_found("Missing birthday"))?;
    let tz = person.timezone.unwrap_or(TimeZone::UTC);
    let now = Timestamp::now();
    if birthday > now.to_zoned(tz.clone()).date() {
        warn!(person = %person.id, "Birthday is in the future");
        return Err(Error::bad_request("Birthday is in the future"));
    }

    let message = Age::at(birthday, now, tz)
        .and_then(|age| age.message())
        .ok_or_else(|| error!(person = %person.id, "Unable to calculate age"))
        .map_err(|_| Error::internal_server_error("Unable to calculate age"))?;
    Ok(message.to_string())
}
