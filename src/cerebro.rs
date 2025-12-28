use crate::rules::{Acceleration, CardRules, CardType, Classification, ScalingNumber};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, ops::RangeInclusive};
use uuid::Uuid;

const PACKS_API: &str = "https://cerebro-beta-bot.herokuapp.com/packs";
const CARDS_API: &str = "https://cerebro-beta-bot.herokuapp.com/cards";
const SETS_API: &str = "https://cerebro-beta-bot.herokuapp.com/sets";

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
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
#[allow(dead_code)]
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

impl CardRules for Card {
    fn rules_text(&self) -> Option<&str> {
        self.rules.as_deref()
    }
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
        use lazy_static::lazy_static;
        use regex::Regex;
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
pub enum SetNumber {
    Unknown,
    Range(RangeInclusive<u32>),
}

impl SetNumber {
    pub fn length(&self) -> u32 {
        match self {
            SetNumber::Unknown => 1,
            SetNumber::Range(r) => r.end() - r.start() + 1,
        }
    }

    pub fn as_range(&self) -> Option<&RangeInclusive<u32>> {
        match self {
            SetNumber::Unknown => None,
            SetNumber::Range(r) => Some(r),
        }
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
        use lazy_static::lazy_static;
        use regex::Regex;
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

            Ok(SetNumber::Range(start..=end))
        } else if ["??", "???"].contains(&value) {
            Ok(SetNumber::Unknown)
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
    #[serde(rename = "Leader Set")]
    Leader,
    #[serde(rename = "Modular Set")]
    Modular,
    #[serde(rename = "Nemesis Set")]
    Nemesis,
    #[serde(rename = "Supplementary Set")]
    Supplementary,
    #[serde(rename = "Villain Set")]
    Villain,
}

impl fmt::Display for SetType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SetType::Campaign => write!(f, "Campaign"),
            SetType::Hero => write!(f, "Hero"),
            SetType::Leader => write!(f, "Leader"),
            SetType::Modular => write!(f, "Modular"),
            SetType::Nemesis => write!(f, "Nemesis"),
            SetType::Supplementary => write!(f, "Supplementary"),
            SetType::Villain => write!(f, "Scenario"),
        }
    }
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
    use crate::rules::Icon;

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
