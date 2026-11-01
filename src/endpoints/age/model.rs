use std::fmt;

use chrono::{DateTime, Datelike, NaiveDate, Utc};

use crate::error::Error;
use crate::models::Person;

pub(super) struct Age {
    today: NaiveDate,
    birthday: NaiveDate,
}

impl Age {
    pub(super) fn at(person: &Person, now: DateTime<Utc>) -> Result<Self, Error> {
        let birthday = person
            .birthday
            .ok_or_else(|| Error::not_found("Missing birthday"))?;
        let today = now
            .with_timezone(&person.timezone.unwrap_or(chrono_tz::UTC))
            .date_naive();
        Self::from_dates(birthday, today)
    }

    fn from_dates(birthday: NaiveDate, today: NaiveDate) -> Result<Self, Error> {
        if birthday > today {
            return Err(Error::bad_request("Birthday is in the future"));
        }
        if today.year() == NaiveDate::MAX.year() {
            return Err(Error::internal_server_error("Invalid anniversary date"));
        }
        Ok(Self { today, birthday })
    }

    fn anniversary(&self, year: i32) -> NaiveDate {
        self.birthday
            .with_year(year)
            .or_else(|| NaiveDate::from_ymd_opt(year, 2, 28))
            .expect("anniversary year is within the validated date range")
    }
}

impl fmt::Display for Age {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this_year = self.anniversary(self.today.year());
        let next_birthday = self.anniversary(self.today.year() + i32::from(this_year < self.today));
        let years = next_birthday.year() - self.birthday.year();
        match (next_birthday - self.today).num_days() {
            0 => write!(f, " 🥳 I am {years} 🎂 Today is my 🎈BIRTHDAY🎉"),
            1 => write!(f, "I will be {years} tomorrow! 🥳"),
            days @ 2..=7 => write!(f, "I will be {years} in {days} days! 🥳"),
            _ => {
                let previous_birthday = self.anniversary(next_birthday.year() - 1);
                let elapsed_days = (self.today - previous_birthday).num_days();
                let year_days = (next_birthday - previous_birthday).num_days();
                let age = (years - 1) as f64 + elapsed_days as f64 / year_days as f64;
                let number = format!("{age:.4}");
                f.write_str(number.trim_end_matches('0').trim_end_matches('.'))
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/endpoints/age/model/tests.rs"]
mod tests;
