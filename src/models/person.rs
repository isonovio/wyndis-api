use std::collections::HashMap;

use chrono::NaiveDate;
use chrono_tz::Tz;
use serde::Deserialize;

use crate::sites::faceit;
use crate::sites::twitch::TwitchChannelName;

use super::utils::{naive_date, slug};

#[derive(Clone, Debug, Deserialize)]
#[serde(try_from = "PersonConfig")]
pub struct Person {
    pub id: String,
    pub twitch_channel: Option<String>,
    pub faceit_nickname: Option<faceit::Nickname>,
    pub birthday: Option<NaiveDate>,
    pub timezone: Option<Tz>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PersonConfig {
    id: String,
    twitch_channel: Option<TwitchChannelName>,
    faceit_nickname: Option<faceit::Nickname>,
    birthday: Option<toml::value::Datetime>,
    timezone: Option<Tz>,
}

impl TryFrom<PersonConfig> for Person {
    type Error = String;

    fn try_from(config: PersonConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            id: slug(config.id).ok_or("Invalid person ID".to_owned())?,
            twitch_channel: config.twitch_channel.map(|channel| channel.to_string()),
            faceit_nickname: config.faceit_nickname,
            birthday: config
                .birthday
                .map(|d| naive_date(d).ok_or("Invalid birthday".to_owned()))
                .transpose()?,
            timezone: config.timezone,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(try_from = "Vec<Person>")]
pub struct Persons {
    by_id: HashMap<String, Person>,
    by_twitch: HashMap<TwitchChannelName, String>,
}

impl Persons {
    pub fn from_twitch(&self, channel: &TwitchChannelName) -> Option<&Person> {
        self.by_twitch
            .get(channel)
            .and_then(|id| self.by_id.get(id))
    }
}

impl TryFrom<Vec<Person>> for Persons {
    type Error = String;

    fn try_from(persons: Vec<Person>) -> Result<Self, Self::Error> {
        let mut result = Self::default();
        for person in persons {
            if result.by_id.contains_key(&person.id) {
                return Err(format!("Duplicate person ID: {}", person.id));
            }
            if let Some(channel) = &person.twitch_channel {
                let channel =
                    TwitchChannelName::try_from(channel.clone()).map_err(str::to_owned)?;
                if result.by_twitch.contains_key(&channel) {
                    return Err(format!("Duplicate Twitch channel: {channel}"));
                }
                result.by_twitch.insert(channel, person.id.clone());
            }
            result.by_id.insert(person.id.clone(), person);
        }
        Ok(result)
    }
}

#[cfg(test)]
#[path = "../tests/models/person/tests.rs"]
mod tests;
