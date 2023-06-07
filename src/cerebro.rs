use serde::{Deserialize, Serialize};
use uuid::Uuid;

const PACKS_API: &str = "https://cerebro-beta-bot.herokuapp.com/packs";
const CARDS_API: &str = "https://cerebro-beta-bot.herokuapp.com/cards";

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Pack {
    pub deleted: bool,
    pub id: Uuid,
    pub official: bool,
    pub author_id: Option<String>,
    pub name: String,
    pub r#type: PackType,
    pub emoji: Option<String>,
    pub incomplete: bool,
    pub number: String,
}

#[derive(Deserialize)]
pub enum PackType {
    #[serde(rename = "Campaign Expansion")]
    CampaignExpansion,
    #[serde(rename = "Core Set")]
    CoreSet,
    #[serde(rename = "Hero Pack")]
    HeroPack,
    #[serde(rename = "Scenario Pack")]
    ScenarioPack,
    #[serde(rename = "Supplements")]
    Supplements,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Card {
    pub id: String,
    pub deleted: bool,
    pub official: bool,
    pub classification: Classicification,
    pub name: String,
    pub subname: Option<String>,
    pub rules: Option<String>,
    pub r#type: CardType,
    pub printings: Vec<Printing>,
}

#[derive(Serialize, Deserialize)]
pub enum Classicification {
    Aggression,
    Basic,
    Determination,
    Encounter,
    Hero,
    Justice,
    Leadership,
    Protection,
}

#[derive(Deserialize, Serialize)]
pub enum CardType {
    Ally,
    #[serde(rename = "Alter-Ego")]
    AlterEgo,
    Attachment,
    Deterrence,
    Environment,
    Event,
    Hero,
    #[serde(rename = "Main Scheme")]
    MainScheme,
    Minion,
    Obligation,
    #[serde(rename = "Player Side Scheme")]
    PlayerSideScheme,
    Resource,
    #[serde(rename = "Side Scheme")]
    SideScheme,
    Sign,
    Support,
    Treachery,
    Upgrade,
    Villain,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Printing {
    pub artificial_id: String,
    pub pack_id: Uuid,
    pub pack_number: String,
    pub set_id: Option<Uuid>,
    pub set_number: Option<String>,
    pub unique_art: bool,
}

pub async fn get_packs() -> Result<Vec<Pack>, reqwest::Error> {
    reqwest::get(PACKS_API).await?.json().await
}

pub async fn get_cards() -> Result<Vec<Card>, reqwest::Error> {
    reqwest::get(CARDS_API).await?.json().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_cards_fixture() {
        let result: Result<Vec<Card>, _> =
            serde_json::from_str(include_str!("../fixtures/cerebro/cards.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn it_parses_packs_fixture() {
        let result: Result<Vec<Pack>, _> =
            serde_json::from_str(include_str!("../fixtures/cerebro/packs.json"));
        assert!(result.is_ok());
    }
}
