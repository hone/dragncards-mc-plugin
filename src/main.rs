mod dragncards;
mod marvelcdb;

use csv::WriterBuilder;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let pack_index: HashMap<String, u32> = marvelcdb::get_packs()
        .await
        .unwrap()
        .into_iter()
        .map(|pack| (pack.code.clone(), pack.position))
        .collect();
    let cards: Vec<dragncards::Card> = marvelcdb::get_cards()
        .await
        .unwrap()
        .into_iter()
        .map(|card| dragncards::Card::new(card, &pack_index))
        .collect();
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path("./marvelcdb.tsv")
        .unwrap();

    for card in cards {
        wtr.serialize(card).unwrap();
    }
}
