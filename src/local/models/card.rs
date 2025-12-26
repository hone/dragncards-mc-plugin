use crate::cerebro::{CardType, Classification};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub subname: Option<String>,
    pub r#type: CardType,
    pub classification: Classification,
    pub traits: Option<String>,
    pub rules: Option<String>,
    pub cost: Option<String>,
    pub resource: Option<String>,
    pub hand: Option<String>,
    pub health: Option<String>,
    pub thwart: Option<String>,
    pub attack: Option<String>,
    pub defense: Option<String>,
    pub recover: Option<String>,
    pub starting_threat: Option<String>,
    pub acceleration: Option<String>,
    pub stage: Option<String>,
    pub set: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::ReaderBuilder;

    #[test]
    fn it_deserializes_local_tsv() {
        let data = include_str!("../../../examples/local.tsv");
        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .from_reader(data.as_bytes());
        
        let result: Vec<Result<Card, _>> = reader.deserialize().collect();
        
        for (i, item) in result.iter().enumerate() {
            if let Err(e) = item {
                panic!("Failed to deserialize row {}: {:?}", i, e);
            }
        }
        
        let cards: Vec<Card> = result.into_iter().map(|r| r.unwrap()).collect();
        assert!(cards.len() > 0);
        assert_eq!(cards[0].name, "Spider-Man");
        assert_eq!(cards[0].id, "01001a");
    }
}
