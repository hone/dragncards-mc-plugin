use indexmap::IndexMap;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreBuiltDeckDoc {
    pub pre_built_decks: IndexMap<String, PreBuiltDeck>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckMenuDoc {
    pub deck_menu: DeckMenu,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreBuiltDeck {
    pub label: String,
    pub cards: Vec<Card>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckMenu {
    pub sub_menus: Vec<SubMenu>,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SubMenu {
    #[serde(rename_all = "camelCase")]
    DeckLists {
        label: String,
        deck_lists: Vec<DeckList>,
    },
    #[serde(rename_all = "camelCase")]
    SubMenu {
        label: String,
        sub_menus: Vec<SubMenu>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckList {
    pub label: String,
    pub deck_list_id: String,
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
