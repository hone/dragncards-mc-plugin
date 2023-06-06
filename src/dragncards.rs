use crate::marvelcdb::{Card as McdbCard, TypeCode};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub uuid: Uuid,
    pub name: String,
    pub r#type: TypeCode,
    pub code: String,
    pub image_url: String,
    pub card_back: CardBack,
}

impl Card {
    pub fn new(card: McdbCard, pack_index: &HashMap<String, u32>) -> Card {
        let uuid = uuid(&card.code);
        let image_url = image_url(&card, pack_index);
        let card_back = card_back(&card);

        Card {
            uuid,
            name: card.name,
            r#type: card.type_code,
            code: card.code,
            image_url,
            card_back,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardBack {
    DoubleSided,
    Encounter,
    Player,
    Villain,
}

fn uuid(code: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, code.as_bytes())
}

fn image_url(card: &McdbCard, pack_index: &HashMap<String, u32>) -> String {
    let mut side = "";
    match card.code.chars().last().unwrap() {
        'a' => side = "A",
        'b' => side = "B",
        'c' => side = "C",
        _ => (),
    }
    format!(
        "https://cerebrodatastorage.blob.core.windows.net/cerebro-cards/official/{:0>2}{:0>3}{side}.jpg",
        pack_index.get(card.pack_code.as_str()).unwrap(),
        card.position
    )
}

fn card_back(card: &McdbCard) -> CardBack {
    match card.type_code {
        TypeCode::Hero | TypeCode::AlterEgo => CardBack::DoubleSided,
        TypeCode::Ally | TypeCode::Event | TypeCode::Support | TypeCode::Upgrade => {
            CardBack::Player
        }
        TypeCode::Villain => CardBack::Villain,
        _ => CardBack::Encounter,
    }
}
