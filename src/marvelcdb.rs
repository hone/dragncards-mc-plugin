use serde::{Deserialize, Serialize};

const CARDS_API: &str = "https://marvelcdb.com/api/public/cards/?encounter=1";

#[derive(Deserialize)]
pub struct Card {
    pub name: String,
    pub type_code: TypeCode,
    pub code: String,
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
    Ok(reqwest::get(CARDS_API).await?.json::<Vec<Card>>().await?)
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
