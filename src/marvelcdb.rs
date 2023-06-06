use serde::{Deserialize, Serialize};

const CARDS_API: &str = "https://marvelcdb.com/api/public/cards/?encounter=1";
const PACKS_API: &str = "https://marvelcdb.com/api/public/packs";

#[derive(Deserialize)]
pub struct Card {
    pub name: String,
    pub type_code: TypeCode,
    pub pack_code: String,
    pub code: String,
    pub position: u32,
}

#[derive(Deserialize)]
pub struct Pack {
    pub name: String,
    pub code: String,
    pub position: u32,
    pub known: u32,
    pub total: u32,
    pub url: String,
    pub id: u32,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeCode {
    Hero,
    Ally,
    AlterEgo,
    Attachment,
    Environment,
    Event,
    MainScheme,
    Minion,
    Obligation,
    Resource,
    SideScheme,
    Support,
    Treachery,
    Upgrade,
    Villain,
}

pub async fn get_cards() -> Result<Vec<Card>, reqwest::Error> {
    reqwest::get(CARDS_API).await?.json().await
}

pub async fn get_packs() -> Result<Vec<Pack>, reqwest::Error> {
    reqwest::get(PACKS_API).await?.json().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_cards_fixture() {
        let result: Result<Vec<Card>, _> =
            serde_json::from_str(include_str!("../fixtures/marvelcdb.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn it_parses_api() {
        let result = tokio_test::block_on(get_cards());
        assert!(result.is_ok());
    }
}
