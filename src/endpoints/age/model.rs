use std::fmt;

use jiff::{
    Timestamp, ToSpan, Zoned,
    civil::Date,
    tz::{AmbiguousOffset, TimeZone},
};

use super::offset::Offset;

pub(super) struct Age {
    birth: Zoned,
    now: Zoned,
}

impl Age {
    pub(super) fn at(birthday: Date, now: Timestamp, tz: TimeZone) -> Option<Self> {
        let birth = midnight(birthday, tz.clone())?;
        let now = now.to_zoned(tz);
        (birth <= now).then_some(Self { birth, now })
    }

    fn anniversary(&self, year: i16) -> Option<Date> {
        let years = (year - self.birth.year()).years();
        self.birth.date().checked_add(years).ok()
    }

    fn next_birthday(&self) -> Option<Zoned> {
        let mut year = self.now.year();
        if self.anniversary(year)? < self.now.date() {
            year = year.checked_add(1)?;
        };
        midnight(self.anniversary(year)?, self.now.time_zone().clone())
    }

    pub(super) fn message(&self) -> Option<Message> {
        let today = self.now.date();
        let next = self.next_birthday()?;
        let years = next.year() - self.birth.year();
        let countdown_start = next.date().checked_sub(1.month()).ok()?;
        match today == next.date() {
            true => Some(Message::Birthday { years }),
            false => match today >= countdown_start {
                true => Some(Message::Upcoming {
                    years,
                    countdown: Offset::by_days(&self.now, &next)?,
                }),
                false => Some(Message::Elapsed(Offset::by_years(&self.birth, &self.now)?)),
            },
        }
    }
}

pub(super) enum Message {
    Birthday { years: i16 },
    Upcoming { years: i16, countdown: Offset },
    Elapsed(Offset),
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Birthday { years } => write!(f, " 🥳 I am {years} 🎂 Today is my 🎈BIRTHDAY🎉"),
            Self::Upcoming { years, countdown } => write!(
                f,
                "I will be {years} {} old in {countdown}! 🥳",
                if *years == 1 { "year" } else { "years" }
            ),
            Self::Elapsed(offset) => write!(f, "I am {offset} old."),
        }
    }
}

fn midnight(date: Date, timezone: TimeZone) -> Option<Zoned> {
    let midnight = timezone.to_ambiguous_zoned(date.at(0, 0, 0, 0));
    match midnight.offset() {
        AmbiguousOffset::Gap { .. } => None,
        _ => midnight.earlier().ok(),
    }
}

#[cfg(test)]
#[path = "../../tests/endpoints/age/model/tests.rs"]
mod tests;
