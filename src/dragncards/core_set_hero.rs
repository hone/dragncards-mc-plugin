use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Doc {
    pub heroes: HashMap<String, Vec<Card>>,
}

impl Doc {
    pub fn from_fixture() -> Doc {
        serde_json::from_str(include_str!("../../fixtures/core_set_hero_decks.json")).unwrap()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub uuid: Uuid,
    pub quantity: u32,
    pub load_group_id: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_heroes() {
        let result: Result<Doc, _> =
            serde_json::from_str(include_str!("../../fixtures/core_set_hero_decks.json"));
        assert!(result.is_ok());
    }
}
