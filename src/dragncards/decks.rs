use indexmap::IndexMap;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreBuiltDeckDoc {
    pub pre_built_decks: IndexMap<String, PreBuiltDeck>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckMenuDoc {
    pub deck_menu: DeckMenu,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreBuiltDeck {
    pub label: String,
    pub cards: Vec<Card>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_load_action_list: Option<ActionList>,
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
pub enum ActionList {
    List(Vec<Value>),
    Id(String),
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeckMenu {
    pub sub_menus: Vec<SubMenu>,
}

#[derive(Clone, Serialize)]
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

#[allow(dead_code)]
impl SubMenu {
    pub fn deck_lists(&self) -> Option<&Vec<DeckList>> {
        match self {
            Self::DeckLists {
                label: _,
                deck_lists,
            } => Some(deck_lists),
            _ => None,
        }
    }

    pub fn deck_lists_as_mut(&mut self) -> Option<&mut Vec<DeckList>> {
        match self {
            Self::DeckLists {
                label: _,
                deck_lists,
            } => Some(deck_lists),
            _ => None,
        }
    }

    pub fn sub_menus(&self) -> Option<&Vec<SubMenu>> {
        match self {
            Self::SubMenu {
                label: _,
                sub_menus,
            } => Some(sub_menus),
            _ => None,
        }
    }

    pub fn sub_menus_as_mut(&mut self) -> Option<&mut Vec<SubMenu>> {
        match self {
            Self::SubMenu {
                label: _,
                sub_menus,
            } => Some(sub_menus),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize)]
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
