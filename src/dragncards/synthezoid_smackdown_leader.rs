use crate::dragncards::leader::Leader;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct Doc {
    pub leaders: HashMap<String, Leader>,
}

impl Doc {
    pub fn from_fixture() -> Doc {
        serde_json::from_str(include_str!(
            "../../fixtures/synthezoid_smackdown_leader_decks.json"
        ))
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_civil_war_leaders() {
        let result: Result<Doc, _> = serde_json::from_str(include_str!(
            "../../fixtures/synthezoid_smackdown_leader_decks.json"
        ));
        assert!(result.is_ok());
    }
}
