use crate::cerebro::{Card as CerebroCard, CardType, Classification};
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub uuid: Uuid,
    pub cerebro_id: String,
    pub name: String,
    pub subname: Option<String>,
    pub r#type: CardType,
    pub classification: Classification,
    pub image_url: String,
    pub card_back: CardBack,
    pub traits: Option<String>,
    pub hand_size: Option<u32>,
    pub hit_points: Option<String>,
}

impl From<CerebroCard> for Card {
    fn from(card: CerebroCard) -> Card {
        let uuid = uuid(&card.id);
        let card_back = card_back(&card);
        let image_url = image_url(&card);

        Card {
            uuid,
            cerebro_id: card.id,
            name: card.name,
            subname: card.subname,
            r#type: card.r#type,
            classification: card.classification,
            image_url,
            card_back,
            traits: card.traits.map(|traits| traits.join(",")),
            hand_size: card.hand.map(|hand_size| hand_size.parse::<u32>().unwrap()),
            hit_points: card.health,
        }
    }
}

#[derive(Serialize)]
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

fn image_url(card: &CerebroCard) -> String {
    let official = if card.official {
        "official"
    } else {
        "unofficial"
    };

    format!(
        "https://cerebrodatastorage.blob.core.windows.net/cerebro-cards/{official}/{}.jpg",
        card.id
    )
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
