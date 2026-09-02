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

    fn id(&self) -> Option<&str> {
        None
    }

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

    fn global_hand_size_modifier(&self) -> Option<i32> {
        if let Some(rules) = self.rules_text() {
            lazy_static! {
                static ref GLOBAL_HAND_SIZE_RE: Regex = Regex::new(
                    r"(?i)(?:each|every)\s+(?:identity|player)\s+gets\s+([+-]\d+)\s+hand\s+size"
                )
                .unwrap();
            }
            if let Some(captures) = GLOBAL_HAND_SIZE_RE.captures(rules) {
                return captures[1].parse::<i32>().ok();
            }
        }
        None
    }

    fn hero_hand_size_modifier(&self) -> Option<i32> {
        if self.global_hand_size_modifier().is_some() {
            return None;
        }

        if let Some(rules) = self.rules_text() {
            if rules.contains("for each") || rules.contains("until the end of the phase") {
                return None;
            }

            lazy_static! {
                static ref HERO_HAND_SIZE_RE: Regex = Regex::new(
                    r"(?i)(?:you\s+get|your\s+hero\s+gets)\s+([+-]\d+)\s+hand\s+size\s+while\s+(?:you\s+are\s+)?in\s+hero\s+form|while\s+(?:you\s+are\s+)?in\s+hero\s+form,\s+(?:you|your\s+hero)\s+gets?\s+([+-]\d+)\s+hand\s+size"
                )
                .unwrap();
                static ref GENERAL_HAND_SIZE_INC_RE: Regex = Regex::new(
                    r"(?i)(?:you\s+get|your\s+identity\s+gets).*?([+-]\d+)\s+hand\s+size"
                )
                .unwrap();
                static ref GENERAL_HAND_SIZE_DEC_RE: Regex =
                    Regex::new(r"(?i)(?:your\s+)?hand\s+size\s+is\s+reduced\s+by\s+(\d+)").unwrap();
            }

            if let Some(captures) = HERO_HAND_SIZE_RE.captures(rules) {
                let val = captures
                    .get(1)
                    .or_else(|| captures.get(2))
                    .unwrap()
                    .as_str();
                return val.parse::<i32>().ok();
            }

            if self.r#type() != CardType::Hero && self.r#type() != CardType::AlterEgo {
                if let Some(captures) = GENERAL_HAND_SIZE_INC_RE.captures(rules) {
                    return captures[1].parse::<i32>().ok();
                }
                if let Some(captures) = GENERAL_HAND_SIZE_DEC_RE.captures(rules) {
                    if let Ok(val) = captures[1].parse::<i32>() {
                        return Some(-val);
                    }
                }
            }
        }

        None
    }

    fn alter_ego_hand_size_modifier(&self) -> Option<i32> {
        if self.global_hand_size_modifier().is_some() {
            return None;
        }

        // Vision Intangible Mass Form
        if self.id() == Some("26002A") || self.id() == Some("57046B") {
            return Some(1);
        }

        if let Some(rules) = self.rules_text() {
            if rules.contains("for each") || rules.contains("until the end of the phase") {
                return None;
            }

            lazy_static! {
                static ref ALTER_EGO_HAND_SIZE_RE: Regex = Regex::new(
                    r"(?i)(?:you\s+get|your\s+alter-ego\s+gets)\s+([+-]\d+)\s+hand\s+size\s+while\s+(?:you\s+are\s+)?in\s+alter-ego\s+form|while\s+(?:you\s+are\s+)?in\s+alter-ego\s+form,\s+(?:you|your\s+alter-ego)\s+gets?\s+([+-]\d+)\s+hand\s+size"
                )
                .unwrap();
                static ref HERO_HAND_SIZE_RE: Regex = Regex::new(
                    r"(?i)(?:you\s+get|your\s+hero\s+gets)\s+([+-]\d+)\s+hand\s+size\s+while\s+(?:you\s+are\s+)?in\s+hero\s+form|while\s+(?:you\s+are\s+)?in\s+hero\s+form,\s+(?:you|your\s+hero)\s+gets?\s+([+-]\d+)\s+hand\s+size"
                )
                .unwrap();
                static ref GENERAL_HAND_SIZE_INC_RE: Regex = Regex::new(
                    r"(?i)(?:you\s+get|your\s+identity\s+gets).*?([+-]\d+)\s+hand\s+size"
                )
                .unwrap();
                static ref GENERAL_HAND_SIZE_DEC_RE: Regex =
                    Regex::new(r"(?i)(?:your\s+)?hand\s+size\s+is\s+reduced\s+by\s+(\d+)").unwrap();
            }

            if let Some(captures) = ALTER_EGO_HAND_SIZE_RE.captures(rules) {
                let val = captures
                    .get(1)
                    .or_else(|| captures.get(2))
                    .unwrap()
                    .as_str();
                return val.parse::<i32>().ok();
            }

            if HERO_HAND_SIZE_RE.is_match(rules) {
                return None;
            }

            if self.r#type() != CardType::Hero && self.r#type() != CardType::AlterEgo {
                if let Some(captures) = GENERAL_HAND_SIZE_INC_RE.captures(rules) {
                    return captures[1].parse::<i32>().ok();
                }
                if let Some(captures) = GENERAL_HAND_SIZE_DEC_RE.captures(rules) {
                    if let Ok(val) = captures[1].parse::<i32>() {
                        return Some(-val);
                    }
                }
            }
        }

        None
    }

    fn identity_hit_points_modifier(&self) -> Option<i32> {
        if let Some(rules) = self.rules_text() {
            let lower_rules = rules.to_lowercase();
            // Exclude temporary/conditional modifiers or ally/minion/character attachments
            if lower_rules.contains("for each")
                || lower_rules.contains("until the end of")
                || lower_rules.contains("attached minion")
                || lower_rules.contains("attached ally")
                || lower_rules.contains("attached character")
                || lower_rules.contains("attached enemy")
                || lower_rules.contains("each ally")
                || lower_rules.contains("allies get")
                || lower_rules.contains("ally gets")
                || lower_rules.contains("each minion")
                || lower_rules.contains("minions get")
            {
                return None;
            }

            if self.r#type() != CardType::Upgrade && self.r#type() != CardType::Support {
                return None;
            }

            lazy_static! {
                static ref IDENTITY_HP_RE: Regex = Regex::new(
                    r"(?i)(?:you\s+get|your\s+identity\s+gets|[a-z\-0-9\s]+\s+gets|\band)\s+([+-]\d+)\s+hit\s+points?"
                )
                .unwrap();
            }

            if let Some(captures) = IDENTITY_HP_RE.captures(rules) {
                return captures[1].parse::<i32>().ok();
            }
        }
        None
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
        id: Option<String>,
        rules: Option<String>,
        card_type: Option<CardType>,
    }

    impl CardRules for MockCard {
        fn id(&self) -> Option<&str> {
            self.id.as_deref()
        }

        fn r#type(&self) -> CardType {
            self.card_type.clone().unwrap_or(CardType::Upgrade)
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
            id: None,
            rules: Some("Victory 1.".to_string()),
            card_type: None,
        };
        assert_eq!(card.victory(), Some(1));

        let card_neg = MockCard {
            id: None,
            rules: Some("Victory -2.".to_string()),
            card_type: None,
        };
        assert_eq!(card_neg.victory(), Some(-2));
    }

    #[test]
    fn test_hinder_parsing() {
        let card = MockCard {
            id: None,
            rules: Some("Hinder 3{i}.".to_string()),
            card_type: None,
        };
        assert_eq!(card.hinder(), Some(3)); // usize
    }

    #[test]
    fn test_icon_parsing() {
        let card = MockCard {
            id: None,
            rules: Some("Rules with {a}{a} and {c} and {h}.".to_string()),
            card_type: None,
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
            id: None,
            rules: Some("Permanent. Toughness. Starting.".to_string()),
            card_type: None,
        };
        assert!(card.is_permanent());
        assert!(card.is_tough());
        assert!(card.is_starting());
    }

    #[test]
    fn test_uses_parsing() {
        let card = MockCard {
            id: None,
            rules: Some("Attach to Venom. Uses (2 rage counters).".to_string()),
            card_type: None,
        };
        assert_eq!(card.uses(), Some(2));

        let card_tac = MockCard {
            id: None,
            rules: Some("Uses (3 charge counters).".to_string()),
            card_type: None,
        };
        assert_eq!(card_tac.uses(), Some(3));

        let card_none = MockCard {
            id: None,
            rules: Some("Attack for 3 damage.".to_string()),
            card_type: None,
        };
        assert_eq!(card_none.uses(), None);
    }

    #[test]
    fn test_global_hand_size_modifier() {
        let card_live = MockCard {
            id: None,
            rules: Some("Each identity gets +2 hand size.".to_string()),
            card_type: Some(CardType::PlayerSideScheme),
        };
        assert_eq!(card_live.global_hand_size_modifier(), Some(2));
        assert_eq!(card_live.hero_hand_size_modifier(), None);
        assert_eq!(card_live.alter_ego_hand_size_modifier(), None);

        let card_mojo = MockCard {
            id: None,
            rules: Some("Each player gets +1 hand size.".to_string()),
            card_type: Some(CardType::Environment),
        };
        assert_eq!(card_mojo.global_hand_size_modifier(), Some(1));

        let card_freeze = MockCard {
            id: None,
            rules: Some("Each player gets -1 hand size.".to_string()),
            card_type: Some(CardType::Environment),
        };
        assert_eq!(card_freeze.global_hand_size_modifier(), Some(-1));

        let card_asgard = MockCard {
            id: None,
            rules: Some("You get +1 hand size.".to_string()),
            card_type: Some(CardType::Support),
        };
        assert_eq!(card_asgard.global_hand_size_modifier(), None);
    }

    #[test]
    fn test_hero_and_alter_ego_hand_size_modifiers() {
        // Hero-only: The Sorcerer Supreme
        let card_sorcerer = MockCard {
            id: None,
            rules: Some(
                "Play only if you have the Mystic trait. You get +1 hand size while in hero form."
                    .to_string(),
            ),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(card_sorcerer.hero_hand_size_modifier(), Some(1));
        assert_eq!(card_sorcerer.alter_ego_hand_size_modifier(), None);

        // General: Asgard
        let card_asgard = MockCard {
            id: None,
            rules: Some("You get +1 hand size.".to_string()),
            card_type: Some(CardType::Support),
        };
        assert_eq!(card_asgard.hero_hand_size_modifier(), Some(1));
        assert_eq!(card_asgard.alter_ego_hand_size_modifier(), Some(1));

        // General: Symbiote Suit
        let card_symbiote = MockCard {
            id: None,
            rules: Some("Your identity gets +1 to each of its basic powers, +1 hand size, and +10 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(card_symbiote.hero_hand_size_modifier(), Some(1));
        assert_eq!(card_symbiote.alter_ego_hand_size_modifier(), Some(1));

        // General Reduction: Martial Law
        let card_martial = MockCard {
            id: None,
            rules: Some("Your hand size is reduced by 1.".to_string()),
            card_type: Some(CardType::Obligation),
        };
        assert_eq!(card_martial.hero_hand_size_modifier(), Some(-1));
        assert_eq!(card_martial.alter_ego_hand_size_modifier(), Some(-1));

        // Vision Intangible Mass Form
        let card_intangible = MockCard {
            id: Some("26002A".to_string()),
            rules: Some("Vision cannot attack or defend.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(card_intangible.hero_hand_size_modifier(), None);
        assert_eq!(card_intangible.alter_ego_hand_size_modifier(), Some(1));

        // Vision Dense Mass Form
        let card_dense = MockCard {
            id: Some("26002B".to_string()),
            rules: Some("While in hero form, Vision gets +2 ATK and +2 DEF.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(card_dense.hero_hand_size_modifier(), None);
        assert_eq!(card_dense.alter_ego_hand_size_modifier(), None);

        // Dynamic cards must return None (handled by dynamic gameRules)
        let card_iron_man = MockCard {
            id: None,
            rules: Some("You get +1 hand size for each Tech upgrade you control (to a maximum hand size of 7).".to_string()),
            card_type: Some(CardType::Hero),
        };
        assert_eq!(card_iron_man.hero_hand_size_modifier(), None);
        assert_eq!(card_iron_man.alter_ego_hand_size_modifier(), None);

        let card_star_lord = MockCard {
            id: None,
            rules: Some("While you are in hero form, you get +1 hand size for each facedown encounter card in front of you (to a maximum of +3 hand size).".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(card_star_lord.hero_hand_size_modifier(), None);
        assert_eq!(card_star_lord.alter_ego_hand_size_modifier(), None);
    }

    #[test]
    fn test_identity_hit_points_modifier() {
        // Rocket Boots
        let rocket_boots = MockCard {
            id: None,
            rules: Some("You get +1 hit point.\nHero Action: Exhaust Rocket Boots...".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(rocket_boots.identity_hit_points_modifier(), Some(1));

        // Mark V Armor
        let mark_v = MockCard {
            id: None,
            rules: Some("You get +6 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(mark_v.identity_hit_points_modifier(), Some(6));

        // Endurance
        let endurance = MockCard {
            id: None,
            rules: Some("Play under any player's control. Max 1 per player.\nYou get +3 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(endurance.identity_hit_points_modifier(), Some(3));

        // Thor's Helmet
        let thors_helmet = MockCard {
            id: None,
            rules: Some("You get +5 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(thors_helmet.identity_hit_points_modifier(), Some(5));

        // Hercules upgrades (named character)
        let nemean = MockCard {
            id: None,
            rules: Some("Permanent.\nHercules gets +2 hit points and gains steady.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(nemean.identity_hit_points_modifier(), Some(2));

        let sword_peleus = MockCard {
            id: None,
            rules: Some("Permanent. Restricted.\nHercules gets +1 hit point and his basic attacks gain piercing.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(sword_peleus.identity_hit_points_modifier(), Some(1));

        // Symbiote Suit
        let symbiote = MockCard {
            id: None,
            rules: Some("Max 1 per deck.\nYour identity gets +1 to each of its basic powers, +1 hand size, and +10 hit points.\n{h}".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(symbiote.identity_hit_points_modifier(), Some(10));

        // Front Line Specialist
        let front_line = MockCard {
            id: None,
            rules: Some("Linked (Specialized Training).\nYour identity gets +4 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(front_line.identity_hit_points_modifier(), Some(4));

        // Impact-Dampening Suit (Side A and B)
        let impact_a = MockCard {
            id: None,
            rules: Some("Setup. Permanent.\nYour identity gets +2 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(impact_a.identity_hit_points_modifier(), Some(2));

        let impact_b = MockCard {
            id: None,
            rules: Some("Permanent.\nYour identity gets +3 hit points.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(impact_b.identity_hit_points_modifier(), Some(3));

        // Exclusions: Ally buffs must return None
        let team_training = MockCard {
            id: None,
            rules: Some("Play under any player's control. Max 1 per player.\nEach ally you control gets +1 hit point.".to_string()),
            card_type: Some(CardType::Support),
        };
        assert_eq!(team_training.identity_hit_points_modifier(), None);

        let sidekick = MockCard {
            id: None,
            rules: Some("Attach to an identity-specific ally you control.\nAttached ally gets +2 hit points and is your \"sidekick.\"".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(sidekick.identity_hit_points_modifier(), None);

        let shield_deputy = MockCard {
            id: None,
            rules: Some("Attach to a friendly character.\nAttached character gets +1 hit point and gains the S.H.I.E.L.D. trait.".to_string()),
            card_type: Some(CardType::Upgrade),
        };
        assert_eq!(shield_deputy.identity_hit_points_modifier(), None);

        let defeat_hydra = MockCard {
            id: None,
            rules: Some("Attached minion gets +6 hit points and gains the Elite trait.".to_string()),
            card_type: Some(CardType::Attachment),
        };
        assert_eq!(defeat_hydra.identity_hit_points_modifier(), None);
    }
}
