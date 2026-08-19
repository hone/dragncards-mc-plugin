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
    Fixed(usize),
    Scaling(usize),
    Infinity,
}

impl ScalingNumber {
    pub fn as_tuple(&self) -> (Option<i64>, Option<usize>) {
        match self {
            ScalingNumber::Fixed(i) => (Some(*i as i64), None),
            ScalingNumber::Scaling(i) => (None, Some(*i)),
            ScalingNumber::Infinity => (Some(-1 as i64), None),
        }
    }
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
                .parse::<usize>()
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
    Fixed(usize),
    Scaling(usize),
    FixedX,
    ScalingX,
    ZeroStar, // This isn't a FixedStar b/c there's no leading '+'
    FixedStar(usize),
    ScalingStar(usize),
    None,
}

impl Acceleration {
    pub fn as_tuple(&self) -> (Option<usize>, Option<usize>) {
        match self {
            Acceleration::Fixed(i) => (Some(*i), None),
            Acceleration::Scaling(i) => (None, Some(*i)),
            Acceleration::FixedStar(i) => (Some(*i), None),
            Acceleration::ScalingStar(i) => (None, Some(*i)),
            Acceleration::ZeroStar => (Some(0), None),
            _ => (None, None),
        }
    }
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
            if let Ok(number) = captures["digit"].parse::<usize>() {
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
    fn r#type(&self) -> CardType;
    fn rules_text(&self) -> Option<&str>;
    fn health(&self) -> Option<ScalingNumber>;
    fn starting_threat(&self) -> Option<ScalingNumber>;
    fn acceleration(&self) -> Option<Acceleration>;
    fn stage(&self) -> Option<&str>;

    fn icons(&self) -> Option<HashMap<Icon, usize>> {
        if let Some(rules) = self.rules_text() {
            let mut icons = HashMap::new();
            let acceleration_icons = rules.matches("{a}").count();
            if acceleration_icons > 0 {
                icons.insert(Icon::Acceleration, acceleration_icons);
            }
            let amplify_icons = rules.matches("{y}").count();
            if amplify_icons > 0 {
                icons.insert(Icon::Amplify, amplify_icons);
            }
            let crisis_icons = rules.matches("{c}").count();
            if crisis_icons > 0 {
                icons.insert(Icon::Crisis, crisis_icons);
            }
            let hazard_icons = rules.matches("{h}").count();
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

    fn hinder(&self) -> Option<usize> {
        if let Some(rules) = self.rules_text() {
            lazy_static! {
                static ref HINDER_RE: Regex = Regex::new(r"Hinder (\d+)\{i\}.").unwrap();
            }
            if let Some(captures) = HINDER_RE.captures(rules) {
                return Some(captures[1].parse::<usize>().unwrap());
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

    fn uses(&self) -> Option<usize> {
        if let Some(rules) = self.rules_text() {
            lazy_static! {
                static ref USES_RE: Regex =
                    Regex::new(r"Uses\s*\((?<number>\d+)\s*(?:[a-zA-Z\s]+)?counters?\)").unwrap();
            }
            if let Some(captures) = USES_RE.captures(rules) {
                return captures["number"].parse::<usize>().ok();
            }
        }
        None
    }

    fn is_permanent(&self) -> bool {
        self.rules_text()
            .map(|r| r.contains("Permanent."))
            .unwrap_or(false)
    }

    fn is_starting(&self) -> bool {
        self.rules_text()
            .map(|r| r.contains("Starting."))
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

    fn health_parsed(&self) -> (Option<i64>, Option<usize>) {
        self.health().map(|s| s.as_tuple()).unwrap_or((None, None))
    }

    fn starting_threat_parsed(&self) -> (Option<i64>, Option<usize>) {
        let (fixed, scaling) = self
            .starting_threat()
            .map(|s| s.as_tuple())
            .unwrap_or((None, None));
        if let Some(hinder) = self.hinder() {
            (fixed, Some(scaling.unwrap_or(0) + hinder))
        } else {
            (fixed, scaling)
        }
    }

    fn acceleration_parsed(&self) -> (Option<usize>, Option<usize>) {
        self.acceleration()
            .map(|a| a.as_tuple())
            .unwrap_or((None, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockCard {
        rules: Option<String>,
    }

    impl CardRules for MockCard {
        fn r#type(&self) -> CardType {
            CardType::Hero
        }

        fn rules_text(&self) -> Option<&str> {
            self.rules.as_deref()
        }

        fn health(&self) -> Option<ScalingNumber> {
            None
        }

        fn starting_threat(&self) -> Option<ScalingNumber> {
            None
        }

        fn acceleration(&self) -> Option<Acceleration> {
            None
        }

        fn stage(&self) -> Option<&str> {
            None
        }
    }

    #[test]
    fn test_victory_parsing() {
        let card = MockCard {
            rules: Some("Victory 1.".to_string()),
        };
        assert_eq!(card.victory(), Some(1));

        let card_neg = MockCard {
            rules: Some("Victory -2.".to_string()),
        };
        assert_eq!(card_neg.victory(), Some(-2));
    }

    #[test]
    fn test_hinder_parsing() {
        let card = MockCard {
            rules: Some("Hinder 3{i}.".to_string()),
        };
        assert_eq!(card.hinder(), Some(3)); // usize
    }

    #[test]
    fn test_icon_parsing() {
        let card = MockCard {
            rules: Some("Rules with {a}{a} and {c} and {h}.".to_string()),
        };
        let icons = card.icons().unwrap();
        assert_eq!(icons.get(&Icon::Acceleration), Some(&2)); // usize
        assert_eq!(icons.get(&Icon::Crisis), Some(&1));
        assert_eq!(icons.get(&Icon::Hazard), Some(&1));
        assert!(icons.get(&Icon::Amplify).is_none());
    }

    #[test]
    fn test_permanent_and_tough() {
        let card = MockCard {
            rules: Some("Permanent. Toughness. Starting.".to_string()),
        };
        assert!(card.is_permanent());
        assert!(card.is_tough());
        assert!(card.is_starting());
    }

    #[test]
    fn test_uses_parsing() {
        let card = MockCard {
            rules: Some("Attach to Venom. Uses (2 rage counters).".to_string()),
        };
        assert_eq!(card.uses(), Some(2));

        let card_tac = MockCard {
            rules: Some("Uses (3 charge counters).".to_string()),
        };
        assert_eq!(card_tac.uses(), Some(3));

        let card_none = MockCard {
            rules: Some("Attack for 3 damage.".to_string()),
        };
        assert_eq!(card_none.uses(), None);
    }
}
