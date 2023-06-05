mod dragncards;
mod marvelcdb;

use csv::WriterBuilder;

#[tokio::main]
async fn main() {
    let cards: Vec<dragncards::Card> = marvelcdb::get_cards()
        .await
        .unwrap()
        .into_iter()
        .map(dragncards::Card::from)
        .collect();
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path("./marvelcdb.tsv")
        .unwrap();

    for card in cards {
        wtr.serialize(card).unwrap();
    }
}
