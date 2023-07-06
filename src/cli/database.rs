use crate::{cerebro, dragncards::database::Card};
use csv::WriterBuilder;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(clap::Args)]
pub struct DatabaseArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub offline: bool,
}

pub async fn execute(args: DatabaseArgs) {
    let pack_map: HashMap<Uuid, cerebro::Pack> = cerebro::get_packs(Some(args.offline))
        .await
        .unwrap()
        .into_iter()
        .map(|pack| (pack.id.clone(), pack))
        .collect();
    let cards: Vec<Card> = cerebro::get_cards(Some(args.offline))
        .await
        .unwrap()
        .into_iter()
        .filter_map(|card| {
            if card.official {
                Some(Card::new(card, &pack_map))
            } else {
                None
            }
        })
        .flatten()
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
