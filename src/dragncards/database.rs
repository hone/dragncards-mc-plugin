use crate::{
    cerebro::{Card as CerebroCard, CardType, Classification, Pack, Printing, ScalingNumber},
    marvelcdb,
};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

const WAKANDA_FOREVER_ID_BASE: &'static str = "01043";
const ANDROID_EFFICIENCY_ID_BASE: &'static str = "01144";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub database_id: Uuid,
    pub cerebro_id: String,
    pub marvelcdb_id: String,
    pub name: String,
    pub subname: Option<String>,
    pub r#type: CardType,
    pub classification: Classification,
    pub image_url: String,
    pub card_back: CardBack,
    pub traits: Option<String>,
    pub hand_size: Option<u32>,
    pub hit_points_fixed: Option<i64>,
    pub hit_points_scaling: Option<i64>,
    pub stage: Option<String>,
    pub starting_threat_fixed: Option<i64>,
    pub starting_threat_scaling: Option<i64>,
    pub toughness: bool,
    pub permanent: bool,
    pub nemesis_minion: bool,
    pub victory: Option<i64>,
}

impl Card {
    pub fn new(card: CerebroCard, packs: &HashMap<Uuid, Pack>) -> Vec<Card> {
        let card_back = card_back(&card);

        card.printings
            .iter()
            .map(|printing| {
                let database_id = uuid(&printing.artificial_id);
                let pack = packs.get(&printing.pack_id).unwrap();
                let image_url = image_url(&card, &printing);
                let permanent = card
                    .rules
                    .as_ref()
                    .map(|rules| rules.contains("Permanent."))
                    .unwrap_or(false);
                let nemesis_minion = card.r#type == CardType::Minion
                    && (card
                        .rules
                        .as_ref()
                        .map(|rules| rules.contains("nemesis minion"))
                        .unwrap_or(false)
                        || printing
                            .set_number
                            .as_ref()
                            .map(|set_number| {
                                let range = &set_number.0;
                                range.contains(&1) || range.contains(&2)
                            })
                            .unwrap_or(false));

                let mut new_card = Card {
                    database_id,
                    cerebro_id: card.id.clone(),
                    marvelcdb_id: marvelcdb::card_id(&pack.number, &printing.pack_number.0),
                    name: card.name.clone(),
                    subname: card.subname.clone(),
                    r#type: card.r#type.clone(),
                    classification: card.classification.clone(),
                    image_url,
                    card_back: card_back.clone(),
                    traits: card.traits.as_ref().map(|traits| traits.join(",")),
                    hand_size: card
                        .hand
                        .as_ref()
                        .map(|hand_size| hand_size.parse::<u32>().unwrap()),
                    hit_points_fixed: None,
                    hit_points_scaling: None,
                    starting_threat_fixed: None,
                    starting_threat_scaling: None,
                    stage: card.stage.clone(),
                    toughness: card
                        .rules
                        .clone()
                        .map(|rules| rules.contains("Toughness."))
                        .unwrap_or(false),
                    nemesis_minion,
                    permanent,
                    victory: card.victory().map(|v| v as i64),
                };

                if let Some(health) = card.health.as_ref() {
                    match health {
                        ScalingNumber::Fixed(i) => new_card.hit_points_fixed = Some(*i as i64),
                        ScalingNumber::Scaling(i) => new_card.hit_points_scaling = Some(*i as i64),
                        ScalingNumber::Infinity => new_card.hit_points_fixed = Some(-1),
                    }
                }

                if let Some(starting_threat) = card.starting_threat.as_ref() {
                    match starting_threat {
                        ScalingNumber::Fixed(i) => new_card.starting_threat_fixed = Some(*i as i64),
                        ScalingNumber::Scaling(i) => {
                            new_card.starting_threat_scaling = Some(*i as i64)
                        }
                        ScalingNumber::Infinity => new_card.starting_threat_fixed = Some(-1),
                    }
                }

                if let Some(hinder) = card.hinder() {
                    let existing = new_card.starting_threat_scaling.unwrap_or(0);
                    new_card.starting_threat_scaling = Some(existing + hinder as i64);
                }

                new_card
            })
            .collect()
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardBack {
    MultiSided,
    Encounter,
    Player,
    Villain,
}

pub fn uuid(code: &str) -> Uuid {
    let id = if let Ok(_) = code.parse::<u32>() {
        code
    } else if code.contains(WAKANDA_FOREVER_ID_BASE) || code.contains(ANDROID_EFFICIENCY_ID_BASE) {
        code
    } else {
        let mut chars = code.chars();
        chars.next_back();
        chars.as_str()
    };

    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes())
}

fn image_url(card: &CerebroCard, printing: &Printing) -> String {
    let official = if card.official {
        "official"
    } else {
        "unofficial"
    };
    let id = if printing.unique_art {
        &printing.artificial_id
    } else {
        &card.id
    };

    format!("https://cerebrodatastorage.blob.core.windows.net/cerebro-cards/{official}/{id}.jpg")
}

fn card_back(card: &CerebroCard) -> CardBack {
    // Wakanda Forever uses A/B/C/D in id, but are not multi-sided cards
    if !card.id.contains(WAKANDA_FOREVER_ID_BASE)
        && !card.id.contains(ANDROID_EFFICIENCY_ID_BASE)
        && card.id.parse::<u32>().is_err()
    {
        return CardBack::MultiSided;
    }

    match card.r#type {
        CardType::Hero | CardType::AlterEgo | CardType::MainScheme => CardBack::MultiSided,
        CardType::Ally
        | CardType::Event
        | CardType::PlayerSideScheme
        | CardType::Resource
        | CardType::Support
        | CardType::Upgrade => CardBack::Player,
        CardType::Villain => CardBack::Villain,
        _ => CardBack::Encounter,
    }
}
