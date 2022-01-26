pub mod eras;

use crate::eras::SORTED_ERAS;
use chrono::prelude::*;
use std::convert::TryInto;

#[derive(Debug)]
pub enum Jidai {
    Asuka,
    Nara,
    Heian,
    Kamakura,
    Nanbokuchou,
    Sengoku,
    Muromachi,
    AzuchiMomoyama,
    Edo,
    Modern,
}

#[derive(Debug)]
pub struct Era {
    pub kanji: Option<&'static str>,
    pub romaji: Option<&'static str>,
    pub jidai: Jidai,
    pub started_at: i64,
    pub ended_at: Option<i64>,
}

impl Era {
    pub fn from_datetime(datetime: DateTime<Utc>) -> Option<&'static Era> {
        Era::from_unix_epoch(datetime.timestamp())
    }

    pub fn from_unix_epoch(unix_epoch: i64) -> Option<&'static Era> {
        if unix_epoch < SORTED_ERAS[0].started_at {
            return None;
        }

        // We just do a linear search because even though this data is sorted,
        // with this small of N, the cache locality is more important than, e.g.
        // the upper bound wins from binary search.
        for era in SORTED_ERAS {
            match (era.started_at < unix_epoch, era.ended_at) {
                // The era hasn't happened yet, continue.
                (false, _) => (),
                // We got to the last era without a match. By default, this
                // means the unix_timestamp is referring to the current era.
                (_, None) => return Some(era),
                // The unix_timestamp falls squarely within this era. Found it!
                (true, Some(ended_at)) => {
                    if unix_epoch < ended_at {
                        return Some(era);
                    }
                }
            }
        }

        return None;
    }

    /// Given a datetime, returns the nenkou datestring.
    pub fn to_jp_nenkou_string(date: DateTime<Utc>) -> Option<String> {
        match Era::from_datetime(date) {
            None => None,
            Some(era) => match era.kanji {
                Some(kanji) => Some(format!(
                    "{}{}年{}月{}日",
                    kanji,
                    to_jp_intstring(
                        (1 + (date - Utc.timestamp(era.started_at, 0)).num_days() / 365)
                            .try_into()
                            .unwrap(),
                    ),
                    to_jp_intstring(date.month()),
                    to_jp_intstring(date.day())
                )),
                None => None,
            },
        }
    }
}

/// Makes a string of the uint and converts the ASCII 0-9 to the Japanese ０−９.
fn to_jp_intstring(num: u32) -> String {
    return num
        .to_string()
        .chars()
        // Japanese integers are shifted 65,248 slots away from ASCII integers
        // in Unicode character space.
        .map(|c| char::from_u32(c as u32 + 65248).unwrap())
        .into_iter()
        .collect();
}

pub fn utc_dt(date: &str) -> DateTime<Utc> {
    Utc.from_utc_datetime(
        &DateTime::parse_from_rfc3339(format!("{}T22:10:57Z", date).as_str())
            .unwrap()
            .naive_utc(),
    )
}

/// A rudimentary way to detect Japanese-language strings.
/// Note: O(n) on the length of the string.
/// Note: Short-circuit returns true on any Japanese grapheme.
/// Note: Does not handle mixed-language strings well.
pub fn is_jp(s: &str) -> bool {
    for c in s.chars().into_iter() {
        match c as u32 {
            // Hiragana graphemes
            0x3040..=0x309F => return true,
            // Katakana graphemes
            0x30A0..=0x30FF => return true,
            // Katakana phonetic extension graphemes
            0x31F0..=0x31FF => return true,
            _ => (),
        }
    }

    return false;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_unix_epoch_various_eras() {
        // Testing various era happy paths
        assert_eq!(
            Era::from_unix_epoch(-1556668810).unwrap().romaji,
            Some("taishou")
        );
        assert_eq!(
            Era::from_unix_epoch(-23123213123).unwrap().romaji,
            Some("katei")
        );
    }

    #[test]
    fn test_from_unix_epoch_first_era_boundary_cases() {
        // 1 second before the earliest era we have should be None.
        assert_eq!(Era::from_unix_epoch(-41795654401).is_none(), true);
        // 1 second after the start of the earliest era we have should exist
        assert_eq!(Era::from_unix_epoch(-41795654399).is_none(), false);
        // Taika should be the first era.
        assert_eq!(
            Era::from_unix_epoch(-41795654399).unwrap().romaji,
            Some("taika")
        );
    }

    #[test]
    fn test_from_unix_epoch_last_era_boundary_cases() {
        // The last era should be reiwa (this should never change)
        assert_eq!(
            Era::from_unix_epoch(1636346788).unwrap().romaji,
            Some("reiwa")
        );

        // Asking for a far future date, e.g. the year 2211, should also yield
        // the last era. (this assertion will change if latest era changes)
        assert_eq!(
            Era::from_unix_epoch(7636346788).unwrap().romaji,
            Some("reiwa")
        );
    }

    #[test]
    fn test_to_jp_nenkou_string() {
        // November 2021 should be Reiwa 3
        assert_eq!(
            Era::to_jp_nenkou_string(
                Utc.from_utc_datetime(
                    &DateTime::parse_from_rfc3339("2021-11-12T22:10:57Z")
                        .unwrap()
                        .naive_utc()
                ),
            ),
            Some("令和３年１１月１２日".to_owned())
        );

        // Summer 2019 should be Reiwa 1
        assert_eq!(
            Era::to_jp_nenkou_string(utc_dt("2019-06-13")),
            Some("令和１年６月１３日".to_owned())
        );
    }

    #[test]
    fn test_is_jp() {
        assert!(!is_jp("testing 123 Hello, world!"));
        assert!(is_jp("日本語の文です。"));
    }
}
