use crate::marvelcdb::{Card as McdbCard, TypeCode};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub uuid: Uuid,
    pub name: String,
    pub type_code: TypeCode,
    pub code: String,
    pub image_url: String,
    pub card_back: CardBack,
}

impl From<McdbCard> for Card {
    fn from(card: McdbCard) -> Card {
        let uuid = uuid(&card.code);
        let image_url = image_url(&card.code);
        let card_back = card_back(&card);

        Card {
            uuid,
            name: card.name,
            type_code: card.type_code,
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

fn image_url(code: &str) -> String {
    format!("https://cerebrodatastorage.blob.core.windows.net/cerebro-cards/official/{code}.jpg",)
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
