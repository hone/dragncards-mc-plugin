use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct Doc {
    pub leaders: HashMap<String, Leader>,
}

impl Doc {
    pub fn from_fixture() -> Doc {
        serde_json::from_str(include_str!("../../fixtures/civil_war_leader_decks.json")).unwrap()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_civil_war_leaders() {
        let result: Result<Doc, _> =
            serde_json::from_str(include_str!("../../fixtures/civil_war_leader_decks.json"));
        assert!(result.is_ok());
    }
}
