pub mod models;

use csv::ReaderBuilder;
use models::card::Card;
use models::deck::Deck;
use std::path::{Path, PathBuf};

pub fn read_cards(paths: &[PathBuf]) -> Vec<Card> {
    let mut cards = Vec::new();
    for path in paths {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "tsv") {
                            cards.extend(read_card_file(&path));
                        }
                    }
                }
            }
        } else {
            cards.extend(read_card_file(path));
        }
    }
    cards
}

fn read_card_file(path: &Path) -> Vec<Card> {
    let mut cards = Vec::new();
    if let Ok(rdr) = ReaderBuilder::new().delimiter(b'\t').from_path(path) {
        let mut rdr = rdr;
        for result in rdr.deserialize() {
            if let Ok(card) = result {
                cards.push(card);
            } else {
                eprintln!(
                    "Warning: Failed to parse row in {:?}: {:?}",
                    path,
                    result.err()
                );
            }
        }
    } else {
        eprintln!("Warning: Failed to open file {:?}", path);
    }
    cards
}

pub fn read_decks(paths: &[PathBuf]) -> Vec<Deck> {
    let mut decks = Vec::new();
    for path in paths {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() && path.extension().map_or(false, |ext| ext == "toml") {
                            decks.extend(read_deck_file(&path));
                        }
                    }
                }
            }
        } else {
            decks.extend(read_deck_file(path));
        }
    }
    decks
}

fn read_deck_file(path: &Path) -> Vec<Deck> {
    if let Ok(content) = std::fs::read_to_string(path) {
        match toml::from_str::<models::deck::LocalDecks>(&content) {
            Ok(local_decks) => local_decks.decks,
            Err(e) => {
                eprintln!("Warning: Failed to parse TOML in {:?}: {:?}", path, e);
                Vec::new()
            }
        }
    } else {
        eprintln!("Warning: Failed to open file {:?}", path);
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::local::models::deck::DeckType;

    #[test]
    fn test_read_cards_from_fixture() {
        let path = PathBuf::from("fixtures/local/cards.tsv");
        let cards = read_cards(&[path]);
        assert_eq!(cards.len(), 8);
        assert_eq!(cards[0].name, "Spider-Man");
    }

    #[test]
    fn test_read_decks_from_fixture() {
        let path = PathBuf::from("fixtures/local/decks.toml");
        let decks = read_decks(&[path]);
        assert_eq!(decks.len(), 2);
        assert_eq!(decks[0].name, "Spider-Man Starter Deck");
        assert_eq!(decks[0].r#type, DeckType::Hero);
    }
}
