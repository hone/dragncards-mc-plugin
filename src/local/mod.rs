pub mod models;

use csv::ReaderBuilder;
use models::card::Card;
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
                            cards.extend(read_file(&path));
                        }
                    }
                }
            }
        } else {
            cards.extend(read_file(path));
        }
    }
    cards
}

fn read_file(path: &Path) -> Vec<Card> {
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
