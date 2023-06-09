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

pub fn uuid(code: &str) -> Uuid {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, code.as_bytes())
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
    // deal with identity and double sided cards
    if card.id.parse::<u32>().is_err() {
        return CardBack::DoubleSided;
    }

    match card.r#type {
        // this SHOULD be covered by the above
        CardType::Hero | CardType::AlterEgo => CardBack::DoubleSided,
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
