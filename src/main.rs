mod cerebro;
mod dragncards;

use csv::WriterBuilder;

#[tokio::main]
async fn main() {
    let mut cards: Vec<dragncards::Card> = cerebro::get_cards()
        .await
        .unwrap()
        .into_iter()
        .filter_map(|card| {
            if card.official {
                Some(dragncards::Card::from(card))
            } else {
                None
            }
        })
        .collect();
    cards.sort_by(|a, b| a.cerebro_id.cmp(&b.cerebro_id));
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path("./marvelcdb.tsv")
        .unwrap();

    for card in cards {
        wtr.serialize(card).unwrap();
    }
}
