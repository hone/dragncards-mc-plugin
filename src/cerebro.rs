use lazy_static::lazy_static;
use regex::Regex;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{collections::HashMap, fmt};
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
    pub acceleration: Option<Acceleration>,
}

impl Card {
    pub fn icons(&self) -> Option<HashMap<Icon, usize>> {
        if let Some(rules) = self.rules.as_ref() {
            let mut icons = HashMap::new();
            let acceleration_icons = rules.matches("{a}").collect::<Vec<_>>().len();
            if acceleration_icons > 0 {
                icons.insert(Icon::Acceleration, acceleration_icons);
            }
            let amplify_icons = rules.matches("{y}").collect::<Vec<_>>().len();
            if amplify_icons > 0 {
                icons.insert(Icon::Amplify, amplify_icons);
            }
            let crisis_icons = rules.matches("{c}").collect::<Vec<_>>().len();
            if crisis_icons > 0 {
                icons.insert(Icon::Crisis, crisis_icons);
            }
            let hazard_icons = rules.matches("{h}").collect::<Vec<_>>().len();
            if hazard_icons > 0 {
                icons.insert(Icon::Hazard, hazard_icons);
            }

            if icons.len() > 0 {
                Some(icons)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn hinder(&self) -> Option<u32> {
        if let Some(rules) = self.rules.as_ref() {
            lazy_static! {
                static ref HINDER_RE: Regex = Regex::new(r"Hinder (\d+)\{i\}.").unwrap();
            }
            if let Some(captures) = HINDER_RE.captures(rules) {
                return Some(captures[1].parse::<u32>().unwrap());
            }
        }

        None
    }

    pub fn victory(&self) -> Option<i32> {
        if let Some(rules) = self.rules.as_ref() {
            lazy_static! {
                static ref VICTORY_RE: Regex = Regex::new(r"Victory (-?\d+).").unwrap();
            }
            if let Some(captures) = VICTORY_RE.captures(rules) {
                return Some(captures[1].parse::<i32>().unwrap());
            }
        }
        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Acceleration,
    Amplify,
    Crisis,
    Hazard,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScalingNumber {
    Fixed(u32),
    Scaling(u32),
    Infinity,
}

struct ScalingNumberVisitor;

impl<'de> Visitor<'de> for ScalingNumberVisitor {
    type Value = ScalingNumber;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer, integer{i} for player scaling, —, or ∞")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        lazy_static! {
            static ref SCALING_NUMBER_RE: Regex =
                Regex::new(r"(?<number>\d+)(?<scaling>\{i\})?").unwrap();
        }

        if let Some(captures) = SCALING_NUMBER_RE.captures(value) {
            let number = captures["number"]
                .parse::<u32>()
                .map_err(|_| E::custom(format!("Need an integer: {value}")))?;
            if captures.name("scaling").is_some() {
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

#[derive(Clone, Debug, PartialEq)]
pub enum Acceleration {
    Fixed(u32),
    Scaling(u32),
    FixedX,
    ScalingX,
    ZeroStar, // This isn't a FixedStar b/c there's no leading '+'
    FixedStar(u32),
    ScalingStar(u32),
    None,
}

struct AccelerationVisitor;

impl<'de> Visitor<'de> for AccelerationVisitor {
    type Value = Acceleration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .write_str("an integer, X, integer{i} for player scaling, or +X{i} for player scaling")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        lazy_static! {
            static ref ACCELERATION_RE: Regex =
                Regex::new(r"[+](?<digit>\d+|X)(?<scaling>\{i\})?(?<star> \{s\})?").unwrap();
        }

        if let Some(captures) = ACCELERATION_RE.captures(value) {
            if let Ok(number) = captures["digit"].parse::<u32>() {
                if captures.name("scaling").is_some() && captures.name("star").is_none() {
                    Ok(Acceleration::Scaling(number))
                } else if captures.name("scaling").is_some() && captures.name("star").is_some() {
                    Ok(Acceleration::ScalingStar(number))
                } else if captures.name("scaling").is_none() && captures.name("star").is_some() {
                    Ok(Acceleration::FixedStar(number))
                } else {
                    Ok(Acceleration::Fixed(number))
                }
            } else {
                if captures.name("scaling").is_some() {
                    Ok(Acceleration::ScalingX)
                } else {
                    Ok(Acceleration::FixedX)
                }
            }
        } else if ["∞", "—", "–", "-"].contains(&value) {
            Ok(Acceleration::None)
        } else if value == "0 {s}" {
            Ok(Acceleration::ZeroStar)
        } else {
            Err(E::custom(format!(
                "Not an integer, X, integer{{i}}, or +X{{i}} format: '{value}'"
            )))
        }
    }
}

impl<'de> Deserialize<'de> for Acceleration {
    fn deserialize<D>(deserializer: D) -> Result<Acceleration, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(AccelerationVisitor)
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
    pub recommends: Option<Vec<Uuid>>,
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

    fn card_by_id(id: &str) -> Card {
        let cards: Vec<Card> =
            serde_json::from_str(include_str!("../fixtures/cerebro/cards.json")).unwrap();
        cards
            .iter()
            .filter(|card| card.id == id)
            .next()
            .unwrap()
            .clone()
    }

    #[test]
    fn it_parses_cards_fixture() {
        let result: Result<Vec<Card>, _> =
            serde_json::from_str(include_str!("../fixtures/cerebro/cards.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn it_parses_hinder() {
        let card = card_by_id("40146");
        assert_eq!(card.hinder(), Some(1));
        let card_no_hinder = card_by_id("01026");
        assert!(card_no_hinder.hinder().is_none())
    }

    #[test]
    fn it_parses_acceleration() {
        let card_fixed = card_by_id("11010B");
        assert_eq!(card_fixed.acceleration, Some(Acceleration::Fixed(1)));
        let card_scaling = card_by_id("01097B");
        assert_eq!(card_scaling.acceleration, Some(Acceleration::Scaling(1)));

        let card_scaling_x = card_by_id("16092B");
        assert_eq!(card_scaling_x.acceleration, Some(Acceleration::ScalingX));
        let card_fixed_x = card_by_id("02018B");
        assert_eq!(card_fixed_x.acceleration, Some(Acceleration::FixedX));

        let card_zero_star = card_by_id("40166B");
        assert_eq!(card_zero_star.acceleration, Some(Acceleration::ZeroStar));
        let card_fixed_star = card_by_id("07001B");
        assert_eq!(
            card_fixed_star.acceleration,
            Some(Acceleration::FixedStar(1))
        );
        let card_scaling_star = card_by_id("24006B");
        assert_eq!(
            card_scaling_star.acceleration,
            Some(Acceleration::ScalingStar(1))
        );

        let card_none = card_by_id("40139B");
        assert_eq!(card_none.acceleration, Some(Acceleration::None));
    }

    #[test]
    fn it_parses_victory() {
        let card = card_by_id("16178B");
        assert_eq!(card.victory(), Some(1));
        let card_negative_victory = card_by_id("27181");
        assert_eq!(card_negative_victory.victory(), Some(-1));
        let card_no_victory = card_by_id("01026");
        assert!(card_no_victory.victory().is_none());
    }

    #[test]
    fn it_parses_icons() {
        let card = card_by_id("27088B");
        assert_eq!(card.icons(), None);

        let accelerate = card_by_id("16112");
        assert_eq!(
            accelerate
                .icons()
                .map(|icons| icons.get(&Icon::Acceleration).map(|quantity| *quantity))
                .flatten(),
            Some(2 as usize)
        );

        let amplify = card_by_id("16069");
        assert_eq!(
            amplify
                .icons()
                .map(|icons| icons.get(&Icon::Amplify).map(|quantity| *quantity))
                .flatten(),
            Some(1 as usize)
        );

        let crisis = card_by_id("16066");
        assert_eq!(
            crisis
                .icons()
                .map(|icons| icons.get(&Icon::Crisis).map(|quantity| *quantity))
                .flatten(),
            Some(1 as usize)
        );

        let hazard = card_by_id("16068");
        assert_eq!(
            hazard
                .icons()
                .map(|icons| icons.get(&Icon::Hazard).map(|quantity| *quantity))
                .flatten(),
            Some(1 as usize)
        );

        let multi = card_by_id("27155");
        if let Some(icons) = multi.icons() {
            assert_eq!(icons.get(&Icon::Acceleration), Some(1 as usize).as_ref());
            assert_eq!(icons.get(&Icon::Crisis), Some(1 as usize).as_ref());
            assert_eq!(icons.get(&Icon::Hazard), Some(1 as usize).as_ref());
        } else {
            assert!(multi.icons().is_some());
        }
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
