use std::fmt;

use jiff::{
    Span, ToSpan, Unit, Zoned,
    civil::{DateTime, DateTimeDifference},
};

const SECONDS_PER_MINUTE: i64 = 60;
const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

pub(super) struct Offset(Span);

impl Offset {
    pub(super) fn by_years(start: &Zoned, end: &Zoned) -> Option<Self> {
        if end < start {
            return None;
        }
        let end = end.with_time_zone(start.time_zone().clone()).datetime();
        let mut start = start.datetime();

        let years = resolve_years(&mut start, &end)?;
        let months = resolve_months(&mut start, &end)?;

        Some(Self(
            start
                .until(
                    DateTimeDifference::new(end)
                        .largest(Unit::Day)
                        .smallest(Unit::Second),
                )
                .ok()?
                .years(years)
                .months(months),
        ))
    }

    pub(super) fn by_days(start: &Zoned, end: &Zoned) -> Option<Self> {
        if end < start {
            return None;
        }
        let duration = start.duration_until(end);
        let seconds = duration.as_secs() + i64::from(duration.subsec_nanos() > 0);
        Some(Self(
            Span::new()
                .days(seconds / SECONDS_PER_DAY)
                .hours(seconds % SECONDS_PER_DAY / SECONDS_PER_HOUR)
                .minutes(seconds % SECONDS_PER_HOUR / SECONDS_PER_MINUTE)
                .seconds(seconds % SECONDS_PER_MINUTE),
        ))
    }
}

fn resolve_years(start: &mut DateTime, end: &DateTime) -> Option<i16> {
    let mut years = end.year() - start.year();
    if start.checked_add(years.years()).ok()? > *end {
        years -= 1;
    }
    *start = start.checked_add(years.years()).ok()?;
    Some(years)
}

fn resolve_months(start: &mut DateTime, end: &DateTime) -> Option<i16> {
    let mut months = (end.year() - start.year()) * 12 + i16::from(end.month() - start.month());
    if start.checked_add(months.months()).ok()? > *end {
        months -= 1;
    }
    *start = start.checked_add(months.months()).ok()?;
    Some(months)
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut separator = "";
        for (value, unit) in [
            (i64::from(self.0.get_years()), "year"),
            (i64::from(self.0.get_months()), "month"),
            (i64::from(self.0.get_days()), "day"),
            (i64::from(self.0.get_hours()), "hour"),
            (self.0.get_minutes(), "minute"),
            (self.0.get_seconds(), "second"),
        ] {
            match value {
                0 => continue,
                value => {
                    let suffix = if value == 1 { "" } else { "s" };
                    write!(f, "{separator}{value} {unit}{suffix}")?;
                    separator = " ";
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/endpoints/age/offset/tests.rs"]
mod tests;
