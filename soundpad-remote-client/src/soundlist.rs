use crate::parse_or::ParseOrDefault;
use serde::{
    de::{Error, Unexpected},
    Deserialize,
};
use std::{path::PathBuf, str::FromStr, time::Duration};
use time::Date;

#[derive(Deserialize, Debug, Clone)]
pub struct SoundList {
    #[serde(rename = "$value")]
    pub sounds: Vec<Sound>,
}

impl FromStr for SoundList {
    type Err = serde_xml_rs::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_xml_rs::from_str(s)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sound {
    pub index: u64,
    pub title: String,
    pub url: PathBuf,
    #[serde(with = "duration")]
    pub duration: Duration,
    #[serde(with = "empty_string_as_none")]
    pub artist: Option<String>,
    #[serde(with = "iso8601")]
    pub added_on: Date,
    #[serde(with = "iso8601")]
    pub last_played_on: Date,
    pub play_count: u64,
}

#[cfg(test)]
//  Todo: Test failure cases
//  Todo: Move tests onto the submodules / into doc comments
mod tests {
    use super::*;

    const MYLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Soundlist>
  <Sound index="1" url="O:\SteamLibrary\steamapps\common\Soundpad\sounds\ba dum tss.mp3" artist="" title="ba dum tss" duration="0:02" addedOn="2022-11-27" lastPlayedOn="2022-11-27" playCount="15"/>
  <Sound index="2" url="O:\SteamLibrary\steamapps\common\Soundpad\sounds\firework.mp3" artist="" title="firework" duration="0:02" addedOn="2022-11-27" lastPlayedOn="2022-11-27" playCount="1"/>
  <Sound index="3" url="O:\SteamLibrary\steamapps\common\Soundpad\sounds\cue.mp3" artist="" title="cue" duration="0:03" addedOn="2022-11-27" lastPlayedOn="2022-11-27" playCount="9"/>
  <Sound index="4" url="O:\SteamLibrary\steamapps\common\Soundpad\sounds\scream.mp3" artist="" title="scream" duration="0:03" addedOn="2022-11-27" lastPlayedOn="2022-11-27" playCount="1"/>
</Soundlist>"#;

    fn get_sounds() -> Vec<Sound> {
        let deserialized: SoundList = serde_xml_rs::from_str(MYLIST).unwrap();
        deserialized.sounds
    }

    #[test]
    fn title_field() {
        let sounds = get_sounds();
        assert_eq!("cue", sounds[2].title);
    }

    #[test]
    fn date_field() {
        let sounds = get_sounds();
        assert_eq!(
            Date::from_calendar_date(2022, time::Month::November, 27).unwrap(),
            sounds[3].last_played_on
        )
    }

    #[test]
    fn duration_field() {
        let sounds = get_sounds();
        assert_eq!(Duration::from_secs(3), sounds[3].duration)
    }
}

mod duration {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut parts = s.split(':');
        let seconds: u64 = parts.next_back().parse_or_default();
        let minutes: u64 = parts.next_back().parse_or_default();
        let hours: u64 = parts.next_back().parse_or_default();

        Ok(Duration::from_secs(
            hours * 60 * 60 + minutes * 60 + seconds,
        ))
    }
}

mod iso8601 {
    use time::Month;

    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut date_parts = s.split('-');
        let year = date_parts.next().parse_or_default();
        let month: u8 = date_parts.next().parse_or_default();
        let month = match month {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => {
                return Err(D::Error::invalid_value(
                    Unexpected::Unsigned(month as u64),
                    &"a month between 1 and 12",
                ))
            }
        };
        let day = date_parts.next().parse_or_default();

        let Ok(date) = Date::from_calendar_date(year, month, day) else {
            return Err(D::Error::invalid_value(
                Unexpected::Str(&s),
                &"a valid date",
            ));
        };

        Ok(date)
    }
}

mod empty_string_as_none {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    }
}
