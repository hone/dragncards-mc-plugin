use crate::{
    dragncards::database::{uuid, Card as DragnCard, CardBack},
    rules::{Acceleration, CardRules, CardType, Classification, Icon, ScalingNumber},
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub subname: Option<String>,
    pub r#type: CardType,
    pub classification: Classification,
    pub traits: Option<String>,
    pub rules: Option<String>,
    pub cost: Option<String>,
    pub resource: Option<String>,
    pub hand: Option<String>,
    pub health: Option<ScalingNumber>,
    pub thwart: Option<String>,
    pub scheme: Option<String>,
    pub attack: Option<String>,
    pub defense: Option<String>,
    pub recover: Option<String>,
    pub boost: Option<String>,
    pub special: Option<String>,
    pub starting_threat: Option<ScalingNumber>,
    pub acceleration: Option<Acceleration>,
    pub stage: Option<String>,
    pub set: Option<String>,
    pub image_url: String,
}

impl CardRules for Card {
    fn r#type(&self) -> CardType {
        self.r#type.clone()
    }

    fn rules_text(&self) -> Option<&str> {
        self.rules.as_deref()
    }

    fn health(&self) -> Option<ScalingNumber> {
        self.health.clone()
    }

    fn starting_threat(&self) -> Option<ScalingNumber> {
        self.starting_threat.clone()
    }

    fn acceleration(&self) -> Option<Acceleration> {
        self.acceleration.clone()
    }

    fn stage(&self) -> Option<&str> {
        self.stage.as_deref()
    }
}

impl From<Card> for DragnCard {
    fn from(card: Card) -> Self {
        let database_id = uuid(&card.id);
        let cerebro_id = card.id.clone();
        let marvelcdb_id = card.id.clone();

        let card_back = match card.r#type {
            CardType::Hero | CardType::AlterEgo | CardType::MainScheme => CardBack::MultiSided,
            CardType::Ally
            | CardType::Event
            | CardType::PlayerSideScheme
            | CardType::Resource
            | CardType::Support
            | CardType::Upgrade => CardBack::Player,
            CardType::Villain => CardBack::Villain,
            _ => CardBack::Encounter,
        };

        let icons = card.icons();
        let (hit_points_fixed, hit_points_scaling) = card.health_parsed();
        let (starting_threat_fixed, starting_threat_scaling) = card.starting_threat_parsed();
        let (acceleration_fixed, acceleration_scaling) = card.acceleration_parsed();
        let toughness = card.is_tough();
        let permanent = card.is_permanent();
        let starting = card.is_starting();
        let uses = card.uses();
        let nemesis_minion = card.r#type == CardType::Minion && card.has_nemesis_minion_rule();
        let victory = card.victory();

        DragnCard {
            database_id,
            cerebro_id,
            marvelcdb_id,
            name: card.name,
            subname: card.subname,
            r#type: card.r#type,
            classification: card.classification,
            image_url: card.image_url,
            card_back,
            traits: card.traits,
            hand_size: card.hand.and_then(|h| h.parse::<u32>().ok()),
            hit_points_fixed,
            hit_points_scaling,
            set: card.set,
            stage: card.stage,
            starting_threat_fixed,
            starting_threat_scaling,
            acceleration_fixed,
            acceleration_scaling,
            acceleration: icons
                .as_ref()
                .and_then(|i| i.get(&Icon::Acceleration).copied()),
            amplify: icons.as_ref().and_then(|i| i.get(&Icon::Amplify).copied()),
            crisis: icons.as_ref().and_then(|i| i.get(&Icon::Crisis).copied()),
            hazard: icons.as_ref().and_then(|i| i.get(&Icon::Hazard).copied()),
            toughness,
            permanent,
            starting,
            uses,
            nemesis_minion,
            victory,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::ReaderBuilder;

    #[test]
    fn it_deserializes_local_tsv() {
        let data = include_str!("../../../examples/local.tsv");
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(data.as_bytes());

        let result: Vec<Result<Card, _>> = reader.deserialize().collect();

        for (i, item) in result.iter().enumerate() {
            if let Err(e) = item {
                panic!("Failed to deserialize row {}: {:?}", i, e);
            }
        }

        let cards: Vec<Card> = result.into_iter().map(|r| r.unwrap()).collect();
        assert!(cards.len() > 0);
        assert_eq!(cards[0].name, "Spider-Man");
        assert_eq!(cards[0].id, "01001a");
        assert_eq!(cards[0].health, Some(ScalingNumber::Fixed(10)));
    }

    #[test]
    fn it_transforms_to_dragncard() {
        let local_card = Card {
            id: "01001a".to_string(),
            name: "Spider-Man".to_string(),
            subname: Some("Peter Parker".to_string()),
            r#type: CardType::Hero,
            classification: Classification::Hero,
            traits: Some("Avenger".to_string()),
            rules: Some("Spider-Sense — Interrupt: When the villain initiates an attack against you, draw 1 card.".to_string()),
            cost: None,
            resource: Some("{w}".to_string()),
            hand: Some("5".to_string()),
            health: Some(ScalingNumber::Fixed(10)),
            thwart: Some("1".to_string()),
            scheme: None,
            attack: Some("2".to_string()),
            defense: Some("3".to_string()),
            recover: None,
            boost: None,
            special: None,
            starting_threat: None,
            acceleration: None,
            stage: None,
            set: Some("Spider-Man".to_string()),
            image_url: "https://example.com/image.png".to_string(),
        };

        let dragn_card: DragnCard = local_card.into();
        assert_eq!(dragn_card.name, "Spider-Man");
        assert_eq!(dragn_card.hit_points_fixed, Some(10));
        assert_eq!(dragn_card.hand_size, Some(5));
        assert_eq!(dragn_card.card_back, CardBack::MultiSided);
    }

    #[test]
    fn it_handles_scaling_threat_and_hinder() {
        let local_card = Card {
            id: "01097a".to_string(),
            name: "The Break-In!".to_string(),
            subname: None,
            r#type: CardType::MainScheme,
            classification: Classification::Encounter,
            traits: None,
            rules: Some(
                "When Revealed: Place 1 threat on this scheme per player. Hinder 2{i}.".to_string(),
            ),
            cost: None,
            resource: None,
            hand: None,
            health: None,
            thwart: None,
            scheme: None,
            attack: None,
            defense: None,
            recover: None,
            boost: None,
            special: None,
            starting_threat: Some(ScalingNumber::Scaling(7)),
            acceleration: None,
            stage: Some("1A".to_string()),
            set: Some("Rhino".to_string()),
            image_url: "https://example.com/image.png".to_string(),
        };

        let dragn_card: DragnCard = local_card.into();
        // 7{i} + Hinder 2{i} = 9{i}
        assert_eq!(dragn_card.starting_threat_scaling, Some(9));
    }
}
