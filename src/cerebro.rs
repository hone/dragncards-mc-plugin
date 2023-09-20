use lazy_static::lazy_static;
use regex::Regex;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::fmt;
use uuid::Uuid;

const PACKS_API: &str = "https://cerebro-beta-bot.herokuapp.com/packs";
const CARDS_API: &str = "https://cerebro-beta-bot.herokuapp.com/cards";
const SETS_API: &str = "https://cerebro-beta-bot.herokuapp.com/sets";

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pack {
    pub deleted: bool,
    pub id: Uuid,
    pub official: bool,
    pub author_id: Option<String>,
    pub name: String,
    pub r#type: PackType,
    pub emoji: Option<String>,
    pub incomplete: bool,
    pub number: String,
}

#[derive(Deserialize, PartialEq)]
pub enum PackType {
    #[serde(rename = "Campaign Expansion")]
    CampaignExpansion,
    #[serde(rename = "Core Set")]
    CoreSet,
    #[serde(rename = "Hero Pack")]
    HeroPack,
    #[serde(rename = "Scenario Pack")]
    ScenarioPack,
    #[serde(rename = "Supplements")]
    Supplements,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Card {
    pub id: String,
    pub deleted: bool,
    pub official: bool,
    pub classification: Classification,
    pub incomplete: bool,
    pub name: String,
    pub subname: Option<String>,
    pub rules: Option<String>,
    pub r#type: CardType,
    pub printings: Vec<Printing>,
    pub stage: Option<String>,
    pub traits: Option<Vec<String>>,
    pub hand: Option<String>,
    pub health: Option<ScalingNumber>,
    pub starting_threat: Option<ScalingNumber>,
}

impl Card {
    pub fn hinder(&self) -> Option<u32> {
        if let Some(rules) = self.rules.as_ref() {
            lazy_static! {
                static ref HINDER_RE: Regex = Regex::new(r"Hinder (\d+)\{i\}").unwrap();
            }
            if let Some(captures) = HINDER_RE.captures(rules) {
                return Some(captures[1].parse::<u32>().unwrap());
            }
        }

        None
    }
}

#[derive(Clone, Debug)]
pub enum ScalingNumber {
    Fixed(u32),
    Scaling(u32),
    Infinity,
}

struct ScalingNumberVisitor;

impl<'de> Visitor<'de> for ScalingNumberVisitor {
    type Value = ScalingNumber;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or integer{i} for player scaling")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        lazy_static! {
            static ref SCALING_NUMBER_RE: Regex = Regex::new(r"(\d+)(\{i\})?").unwrap();
        }

        if let Some(captures) = SCALING_NUMBER_RE.captures(value) {
            let number = captures[1]
                .parse::<u32>()
                .map_err(|_| E::custom(format!("Need an integer: {value}")))?;
            if captures.get(2).is_some() {
                Ok(ScalingNumber::Scaling(number))
            } else {
                Ok(ScalingNumber::Fixed(number))
            }
        } else {
            if ["∞", "—", "–", "-"].contains(&value) {
                Ok(ScalingNumber::Infinity)
            } else {
                Err(E::custom(format!(
                    "Not an integer, integer{{i}}, —, or ∞ format: '{value}'"
                )))
            }
        }
    }
}

impl<'de> Deserialize<'de> for ScalingNumber {
    fn deserialize<D>(deserializer: D) -> Result<ScalingNumber, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ScalingNumberVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Classification {
    Aggression,
    Basic,
    Determination,
    Encounter,
    Hero,
    Justice,
    Leadership,
    #[serde(rename = "'Pool")]
    Pool,
    Protection,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum CardType {
    Ally,
    #[serde(rename = "Alter-Ego")]
    AlterEgo,
    Attachment,
    Deterrence,
    Environment,
    Event,
    Hero,
    #[serde(rename = "Main Scheme")]
    MainScheme,
    Minion,
    Obligation,
    #[serde(rename = "Player Side Scheme")]
    PlayerSideScheme,
    Resource,
    #[serde(rename = "Side Scheme")]
    SideScheme,
    Sign,
    Support,
    Treachery,
    Upgrade,
    Villain,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Printing {
    pub artificial_id: String,
    pub pack_id: Uuid,
    pub pack_number: PackNumber,
    pub set_id: Option<Uuid>,
    pub set_number: Option<SetNumber>,
    pub unique_art: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackNumber(pub String);

struct PackNumberVisitor;

impl<'de> Visitor<'de> for PackNumberVisitor {
    type Value = PackNumber;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or an integer followed by A, B, C, or D")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        lazy_static! {
            static ref PACK_NUMBER_RE: Regex = Regex::new(r"(\d+[A-D]?)").unwrap();
        }

        if let Some(captures) = PACK_NUMBER_RE.captures(value) {
            Ok(PackNumber(format!("{:0>2}", &captures[0])))
        } else {
            Err(E::custom(format!(
                "not an integer or integer followed by A, B, C, or D: {value}"
            )))
        }
    }
}

impl<'de> Deserialize<'de> for PackNumber {
    fn deserialize<D>(deserializer: D) -> Result<PackNumber, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PackNumberVisitor)
    }
}

#[derive(Clone, Debug)]
pub struct SetNumber(pub std::ops::RangeInclusive<u32>);

impl SetNumber {
    pub fn length(&self) -> u32 {
        self.0.end() - self.0.start() + 1
    }
}

struct SetNumberVisitor;

impl<'de> Visitor<'de> for SetNumberVisitor {
    type Value = SetNumber;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer or range separated by a dash")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        lazy_static! {
            static ref SET_NUMBER_RE: Regex = Regex::new(r"(\d+)(-(\d+))?").unwrap();
        }
        if let Some(captures) = SET_NUMBER_RE.captures(value) {
            let start = captures[1]
                .parse::<u32>()
                .map_err(|_| E::custom(format!("Need an initial integer: {value}")))?;
            let end = captures
                .get(3)
                .map(|m| {
                    m.as_str()
                        .parse::<u32>()
                        .map_err(|_| E::custom(format!("Not in range format: {value}")))
                })
                .unwrap_or(Ok(start))?;

            Ok(SetNumber(start..=end))
        } else {
            Err(E::custom(format!("Not in range format: {value}")))
        }
    }
}

impl<'de> Deserialize<'de> for SetNumber {
    fn deserialize<D>(deserializer: D) -> Result<SetNumber, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SetNumberVisitor)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Set {
    pub id: Uuid,
    pub official: bool,
    pub name: String,
    pub r#type: SetType,
    pub modulars: Option<u32>,
    pub pack_id: Uuid,
    pub requires: Option<Vec<Uuid>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Hash)]
pub enum SetType {
    #[serde(rename = "Campaign Set")]
    Campaign,
    #[serde(rename = "Hero Set")]
    Hero,
    #[serde(rename = "Modular Set")]
    Modular,
    #[serde(rename = "Nemesis Set")]
    Nemesis,
    #[serde(rename = "Supplementary Set")]
    Supplementary,
    #[serde(rename = "Villain Set")]
    Villain,
}

pub async fn get_packs(offline: Option<bool>) -> Result<Vec<Pack>, reqwest::Error> {
    let mut packs: Vec<Pack> = if offline.unwrap_or(false) {
        serde_json::from_str(include_str!("../fixtures/cerebro/packs.json")).unwrap()
    } else {
        reqwest::get(PACKS_API).await?.json().await?
    };

    packs.sort_by(|a, b| a.number.cmp(&b.number));

    Ok(packs)
}

pub async fn get_cards(offline: Option<bool>) -> Result<Vec<Card>, reqwest::Error> {
    let mut cards: Vec<Card> = if offline.unwrap_or(false) {
        serde_json::from_str(include_str!("../fixtures/cerebro/cards.json")).unwrap()
    } else {
        reqwest::get(CARDS_API).await?.json().await?
    };

    cards.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(cards)
}

pub async fn get_sets(offline: Option<bool>) -> Result<Vec<Set>, reqwest::Error> {
    let sets: Vec<Set> = if offline.unwrap_or(false) {
        serde_json::from_str(include_str!("../fixtures/cerebro/sets.json")).unwrap()
    } else {
        reqwest::get(SETS_API).await?.json().await?
    };

    Ok(sets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_cards_fixture() {
        let result: Result<Vec<Card>, _> =
            serde_json::from_str(include_str!("../fixtures/cerebro/cards.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn it_parses_packs_fixture() {
        let result: Result<Vec<Pack>, _> =
            serde_json::from_str(include_str!("../fixtures/cerebro/packs.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn it_parses_sets_fixture() {
        let result: Result<Vec<Set>, _> =
            serde_json::from_str(include_str!("../fixtures/cerebro/sets.json"));
        assert!(result.is_ok());
    }
}
