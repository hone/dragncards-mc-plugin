use atoi::atoi;
use serde::{Deserialize, Serialize};

const CARDS_API: &str = "https://marvelcdb.com/api/public/cards/?encounter=1";
#[allow(dead_code)]
const PACKS_API: &str = "https://marvelcdb.com/api/public/packs";

#[derive(Deserialize)]
pub struct Card {
    pub name: String,
    pub type_code: TypeCode,
    pub pack_code: String,
    pub code: String,
    pub position: u32,
    pub quantity: u32,
    pub deck_limit: Option<u32>,
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
    #[serde(rename = "Evidence - Means", alias = "evidence_means")]
    EvidenceMeans,
    #[serde(rename = "Evidence - Motive", alias = "evidence_motive")]
    EvidenceMotive,
    #[serde(rename = "Evidence - Opportunity", alias = "evidence_opportunity")]
    EvidenceOpportunity,
    MainScheme,
    Minion,
    Obligation,
    PlayerSideScheme,
    Resource,
    SideScheme,
    Support,
    Treachery,
    Upgrade,
    Villain,
}

pub async fn get_cards(offline: Option<bool>) -> Result<Vec<Card>, reqwest::Error> {
    if offline.unwrap_or(false) {
        Ok(serde_json::from_str(include_str!("../fixtures/marvelcdb.json")).unwrap())
    } else {
        reqwest::get(CARDS_API).await?.json().await
    }
}

#[allow(dead_code)]
pub async fn get_packs() -> Result<Vec<Pack>, reqwest::Error> {
    reqwest::get(PACKS_API).await?.json().await
}

pub fn card_id(pack_number: &str, pack_position: &str) -> String {
    if pack_position.parse::<u32>().is_ok() {
        format!("{:0>2}{:0>3}", pack_number, pack_position)
    // cerebro uses A/B for all double sided cards, but marvelcdb does not
    } else if pack_number == "26" && (pack_position == "2A" || pack_position == "2B") {
        format!(
            "{:0>2}{:0>3}",
            pack_number,
            atoi::<u32>(pack_position.as_bytes()).unwrap()
        )
    } else {
        format!("{:0>2}{:0>4}", pack_number, pack_position.to_lowercase())
    }
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
    #[ignore]
    fn it_parses_api() {
        let result = tokio_test::block_on(get_cards(None));
        assert!(result.is_ok());
    }
}
