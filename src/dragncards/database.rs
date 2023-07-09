use crate::{
    cerebro::{Card as CerebroCard, CardType, Classification, Pack, Printing, ScalingNumber},
    marvelcdb,
};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize)]
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
                    stage: card.stage.clone(),
                };

                if let Some(health) = card.health.as_ref() {
                    match health {
                        ScalingNumber::Fixed(i) => new_card.hit_points_fixed = Some(*i as i64),
                        ScalingNumber::Scaling(i) => new_card.hit_points_scaling = Some(*i as i64),
                        ScalingNumber::Infinity => new_card.hit_points_fixed = Some(-1),
                    }
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
    if card.id.parse::<u32>().is_err() {
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
