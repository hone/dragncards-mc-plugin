use indexmap::IndexMap;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Doc {
    pub pre_built_decks: IndexMap<String, PreBuiltDeck>,
}

#[derive(Serialize)]
pub struct PreBuiltDeck {
    pub name: String,
    pub cards: Vec<Card>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub load_group_id: String,
    pub quantity: u32,
    pub uuid: Uuid,
    pub _name: String,
}
