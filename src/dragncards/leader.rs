use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Leader {
    pub main_schemes: Vec<Card>,
    pub sets: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub uuid: Uuid,
    pub name: String,
    pub load_group_id: String,
}
