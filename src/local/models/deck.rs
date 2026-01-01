use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Deck {
    pub name: String,
    pub r#type: DeckType,
    pub cards: Option<Vec<DeckCard>>,
    pub set_code: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DeckType {
    Hero,
    Modular,
    Scenario,
    Campaign,
    Nemesis,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DeckCard {
    pub id: String,
    pub quantity: u32,
    pub load_group_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocalDecks {
    pub decks: Vec<Deck>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_deserializes_local_decks_toml() {
        let toml_str = r#"
            [[decks]]
            name = "Spider-Man Starter Deck"
            type = "hero"
            cards = [
                { id = "01001a", quantity = 1 },
                { id = "01001b", quantity = 1 },
                { id = "custom_card", quantity = 1, load_group_id = "sharedOutOfPlay" }
            ]

            [[decks]]
            name = "Rhino Encounter Set"
            type = "scenario"
            set_code = "Rhino"
        "#;

        let local_decks: LocalDecks = toml::from_str(toml_str).unwrap();
        assert_eq!(local_decks.decks.len(), 2);
        assert_eq!(local_decks.decks[0].name, "Spider-Man Starter Deck");
        assert_eq!(local_decks.decks[0].r#type, DeckType::Hero);
        assert_eq!(local_decks.decks[0].cards.as_ref().unwrap().len(), 3);
        assert_eq!(
            local_decks.decks[0].cards.as_ref().unwrap()[2].load_group_id,
            Some("sharedOutOfPlay".to_string())
        );
        assert_eq!(local_decks.decks[1].name, "Rhino Encounter Set");
        assert_eq!(local_decks.decks[1].r#type, DeckType::Scenario);
        assert_eq!(local_decks.decks[1].set_code.as_ref().unwrap(), "Rhino");
    }
}
