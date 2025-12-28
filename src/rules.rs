use lazy_static::lazy_static;
use regex::Regex;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{collections::HashMap, fmt};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    #[serde(rename = "Evidence - Means")]
    EvidenceMeans,
    #[serde(rename = "Evidence - Motive")]
    EvidenceMotive,
    #[serde(rename = "Evidence - Opportunity")]
    EvidenceOpportunity,
    Hero,
    Leader,
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

pub trait CardRules {
    fn rules_text(&self) -> Option<&str>;

    fn health_raw(&self) -> Option<&str> {
        None
    }
    fn starting_threat_raw(&self) -> Option<&str> {
        None
    }
    fn acceleration_raw(&self) -> Option<&str> {
        None
    }

    fn icons(&self) -> Option<HashMap<Icon, usize>> {
        if let Some(rules) = self.rules_text() {
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

    fn hinder(&self) -> Option<u32> {
        if let Some(rules) = self.rules_text() {
            lazy_static! {
                static ref HINDER_RE: Regex = Regex::new(r"Hinder (\d+)\{i\}.").unwrap();
            }
            if let Some(captures) = HINDER_RE.captures(rules) {
                return Some(captures[1].parse::<u32>().unwrap());
            }
        }

        None
    }

    fn victory(&self) -> Option<i64> {
        if let Some(rules) = self.rules_text() {
            lazy_static! {
                static ref VICTORY_RE: Regex = Regex::new(r"Victory (-?\d+).").unwrap();
            }
            if let Some(captures) = VICTORY_RE.captures(rules) {
                return Some(captures[1].parse::<i64>().unwrap());
            }
        }
        None
    }

    fn is_permanent(&self) -> bool {
        self.rules_text()
            .map(|r| r.contains("Permanent."))
            .unwrap_or(false)
    }

    fn is_tough(&self) -> bool {
        self.rules_text()
            .map(|r| r.contains("Toughness."))
            .unwrap_or(false)
    }

    fn has_nemesis_minion_rule(&self) -> bool {
        self.rules_text()
            .map(|r| r.contains("nemesis minion"))
            .unwrap_or(false)
    }

    fn health_parsed(&self) -> (Option<i64>, Option<i64>) {
        parse_scaling_number_str(self.health_raw())
    }

    fn starting_threat_parsed(&self) -> (Option<i64>, Option<i64>) {
        let (fixed, scaling) = parse_scaling_number_str(self.starting_threat_raw());
        if let Some(hinder) = self.hinder() {
            (fixed, Some(scaling.unwrap_or(0) + hinder as i64))
        } else {
            (fixed, scaling)
        }
    }

    fn acceleration_parsed(&self) -> (Option<i64>, Option<i64>) {
        parse_acceleration_str(self.acceleration_raw())
    }
}

pub fn parse_scaling_number_str(input: Option<&str>) -> (Option<i64>, Option<i64>) {
    match input {
        Some(s) => {
            if ["∞", "—", "–", "-"].contains(&s) {
                (Some(-1), None)
            } else {
                lazy_static! {
                    static ref SCALING_RE: Regex =
                        Regex::new(r"(?<number>\d+)(?<scaling>\{i\})?").unwrap();
                }
                if let Some(captures) = SCALING_RE.captures(s) {
                    let num = captures["number"].parse::<i64>().unwrap();
                    if captures.name("scaling").is_some() {
                        (None, Some(num))
                    } else {
                        (Some(num), None)
                    }
                } else {
                    (None, None)
                }
            }
        }
        None => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCard {
        rules: Option<String>,
        health: Option<String>,
        threat: Option<String>,
        accel: Option<String>,
    }

    impl CardRules for MockCard {
        fn rules_text(&self) -> Option<&str> { self.rules.as_deref() }
        fn health_raw(&self) -> Option<&str> { self.health.as_deref() }
        fn starting_threat_raw(&self) -> Option<&str> { self.threat.as_deref() }
        fn acceleration_raw(&self) -> Option<&str> { self.accel.as_deref() }
    }

    #[test]
    fn test_victory_parsing() {
        let card = MockCard { 
            rules: Some("Victory 1.".to_string()), 
            health: None, threat: None, accel: None 
        };
        assert_eq!(card.victory(), Some(1));

        let card_neg = MockCard { 
            rules: Some("Victory -2.".to_string()), 
            health: None, threat: None, accel: None 
        };
        assert_eq!(card_neg.victory(), Some(-2));
    }

    #[test]
    fn test_hinder_parsing() {
        let card = MockCard { 
            rules: Some("Hinder 3{i}.".to_string()), 
            health: None, threat: None, accel: None 
        };
        assert_eq!(card.hinder(), Some(3));
    }

    #[test]
    fn test_icon_parsing() {
        let card = MockCard {
            rules: Some("Rules with {a}{a} and {c} and {h}.".to_string()),
            health: None, threat: None, accel: None
        };
        let icons = card.icons().unwrap();
        assert_eq!(icons.get(&Icon::Acceleration), Some(&2));
        assert_eq!(icons.get(&Icon::Crisis), Some(&1));
        assert_eq!(icons.get(&Icon::Hazard), Some(&1));
        assert!(icons.get(&Icon::Amplify).is_none());
    }

    #[test]
    fn test_scaling_number_parsing() {
        assert_eq!(parse_scaling_number_str(Some("10")), (Some(10), None));
        assert_eq!(parse_scaling_number_str(Some("5{i}")), (None, Some(5)));
        assert_eq!(parse_scaling_number_str(Some("∞")), (Some(-1), None));
    }

    #[test]
    fn test_acceleration_parsing() {
        assert_eq!(parse_acceleration_str(Some("+1")), (Some(1), None));
        assert_eq!(parse_acceleration_str(Some("+2{i}")), (None, Some(2)));
        assert_eq!(parse_acceleration_str(Some("0 {s}")), (Some(0), None));
    }
}
pub fn parse_acceleration_str(input: Option<&str>) -> (Option<i64>, Option<i64>) {
    match input {
        Some(s) => {
            lazy_static! {
                static ref ACCEL_RE: Regex =
                    Regex::new(r"[+](?<digit>\d+|X)(?<scaling>\{i\})?(?<star> \{s\})?").unwrap();
            }
            if let Some(captures) = ACCEL_RE.captures(s) {
                if let Ok(number) = captures["digit"].parse::<i64>() {
                    let is_scaling = captures.name("scaling").is_some();
                    if is_scaling {
                        (None, Some(number))
                    } else {
                        (Some(number), None)
                    }
                } else {
                    (None, None)
                }
            } else {
                if s == "0 {s}" {
                    (Some(0), None)
                } else {
                    (None, None)
                }
            }
        }
        None => (None, None),
    }
}
