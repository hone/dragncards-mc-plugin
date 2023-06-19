use crate::{cerebro, dragncards::database::Card};
use csv::WriterBuilder;

#[derive(clap::Args)]
pub struct DatabaseArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub offline: bool,
}

pub async fn execute(args: DatabaseArgs) {
    let cards: Vec<Card> = cerebro::get_cards(Some(args.offline))
        .await
        .unwrap()
        .into_iter()
        .filter_map(|card| {
            if card.official {
                Some(Card::from(card))
            } else {
                None
            }
        })
        .collect();
    let output = args
        .output
        .unwrap_or_else(|| std::path::PathBuf::from("./marvelcdb.tsv"));
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(output)
        .unwrap();

    for card in cards {
        wtr.serialize(card).unwrap();
    }
}
