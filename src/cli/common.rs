use crate::{
    cerebro::{self, Card as CerebroCard, Printing},
    dragncards::database::Card as DragnCard,
    local::{self, models::card::Card as LocalCard},
};
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

pub enum SourceCard {
    Cerebro {
        card: CerebroCard,
        printing: Printing,
    },
    Local(LocalCard),
}

pub struct LoadedCard {
    pub source: SourceCard,
    pub output: DragnCard,
}

pub async fn load_card_database(
    local_paths: &[PathBuf],
    api: bool,
    offline: bool,
) -> Vec<LoadedCard> {
    let mut cards: Vec<LoadedCard> = Vec::new();

    // 1. Local Source
    if !local_paths.is_empty() {
        let local_cards = local::read_cards(local_paths);
        cards.extend(
            local_cards
                .into_iter()
                .map(|card| LoadedCard {
                    source: SourceCard::Local(card.clone()),
                    output: card.into(),
                })
                .collect::<Vec<LoadedCard>>(),
        );
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
        cards.extend(
            card_handler
                .await
                .unwrap()
                .unwrap()
                .into_iter()
                .filter_map(|cerebro_card| {
                    if cerebro_card.official {
                        Some(
                            DragnCard::from_cerebro_card(&cerebro_card, &pack_map, &set_map)
                                .into_iter()
                                .map(|(dragn_card, cerebro_card, printing)| LoadedCard {
                                    source: SourceCard::Cerebro {
                                        card: cerebro_card,
                                        printing,
                                    },
                                    output: dragn_card,
                                })
                                .collect::<Vec<LoadedCard>>(),
                        )
                    } else {
                        None
                    }
                })
                .flatten()
                .collect::<Vec<LoadedCard>>(),
        );
    }

    cards.sort_by(|a, b| a.output.cerebro_id.cmp(&b.output.cerebro_id));
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
        assert_eq!(cards[0].output.name, "Spider-Man");
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
