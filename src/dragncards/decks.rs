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
    pub label: String,
    pub cards: Vec<Card>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub load_group_id: String,
    pub quantity: u32,
    pub database_id: Uuid,
    #[serde(rename = "_name")]
    pub _name: String,
}
