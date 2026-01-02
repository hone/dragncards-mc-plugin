use crate::{cerebro, dragncards::database::Card, local};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

pub async fn load_card_database(local_paths: &[PathBuf], api: bool, offline: bool) -> Vec<Card> {
    let mut cards: Vec<Card> = Vec::new();

    // 1. Local Source
    if !local_paths.is_empty() {
        let local_cards = local::read_cards(local_paths);
        let dragn_cards: Vec<Card> = local_cards.into_iter().map(|c| c.into()).collect();
        cards.extend(dragn_cards);
    }

    // 2. API Source (Default if no local, or explicit opt-in)
    if api || local_paths.is_empty() {
        let pack_handler = tokio::spawn(cerebro::get_packs(Some(offline)));
        let card_handler = tokio::spawn(cerebro::get_cards(Some(offline)));
        let pack_map: HashMap<Uuid, cerebro::Pack> = pack_handler
            .await
            .unwrap()
            .unwrap()
            .into_iter()
            .map(|pack| (pack.id.clone(), pack))
            .collect();
        let set_map: HashMap<Uuid, cerebro::Set> = tokio::spawn(cerebro::get_sets(Some(offline)))
            .await
            .unwrap()
            .unwrap()
            .into_iter()
            .map(|set| (set.id.clone(), set))
            .collect();
        let cerebro_cards: Vec<Card> = card_handler
            .await
            .unwrap()
            .unwrap()
            .into_iter()
            .filter_map(|card| {
                if card.official {
                    Some(Card::new(card, &pack_map, &set_map))
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        cards.extend(cerebro_cards);
    }

    cards.sort_by(|a, b| a.cerebro_id.cmp(&b.cerebro_id));
    cards
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_load_local_only() {
        let path = PathBuf::from("fixtures/local/cards.tsv");
        let cards = load_card_database(&[path], false, true).await;
        // local_cards.tsv has 8 rows
        assert_eq!(cards.len(), 8);
        assert_eq!(cards[0].name, "Spider-Man");
    }

    #[tokio::test]
    async fn test_load_api_only() {
        // No local paths, offline=true (uses fixtures)
        let cards = load_card_database(&[], false, true).await;
        // Fixtures have thousands of cards
        assert!(cards.len() > 1000);
    }

    #[tokio::test]

    async fn test_load_additive() {
        // 1. Get Baseline (API Only)

        let api_cards = load_card_database(&[], false, true).await;

        let api_count = api_cards.len();

        // 2. Get Combined

        let path = PathBuf::from("fixtures/local/cards.tsv");

        let combined_cards = load_card_database(&[path], true, true).await;

        // 3. Assert Exact Count

        // Note: It might be less if local cards overwrite official ones with same ID!

        // Our logic is: cards.extend(cerebro_cards).

        // If IDs collide, they are duplicates in the Vec unless we dedupe?

        // Current logic: `cards.extend(dragn_cards); cards.extend(cerebro_cards);`

        // It produces DUPLICATES if IDs match.

        // Spider-Man 01001a is in both local.tsv and official fixture.

        // So we expect duplicates.

        // Local has 8 cards.

        assert_eq!(combined_cards.len(), api_count + 8);
    }
}
