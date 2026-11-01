use jiff::civil::Date;

pub fn slug(s: String) -> Option<String> {
    let byte = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
    let is_slug = |s: &str| !s.is_empty() && s.bytes().all(byte);
    is_slug(&s).then_some(s)
}

pub fn civil_date(date: toml::value::Datetime) -> Option<Date> {
    date.to_string().parse().ok()
}
