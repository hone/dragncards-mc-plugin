use crate::{cerebro, dragncards::database::Card, local};
use csv::WriterBuilder;
use futures::{stream, StreamExt};
use std::{collections::HashMap, path::Path};
use tokio::fs;
use uuid::Uuid;

const CONCURRENT_REQUESTS: usize = 20;
const DEFAULT_DOWNLOAD_SERVER: &str = "http://localhost:5000";

#[derive(clap::Args)]
pub struct DatabaseArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub offline: bool,
    #[arg(long)]
    pub download: Option<std::path::PathBuf>,
    #[arg(long)]
    pub download_server: Option<String>,
    #[arg(long)]
    pub local: Vec<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub api: bool,
}

pub async fn execute(args: DatabaseArgs) {
    let mut cards: Vec<Card> = Vec::new();

    // 1. Local Source
    if !args.local.is_empty() {
        let local_cards = local::read_cards(&args.local);
        let dragn_cards: Vec<Card> = local_cards.into_iter().map(|c| c.into()).collect();
        cards.extend(dragn_cards);
    }

    // 2. API Source (Default if no local, or explicit opt-in)
    if args.api || args.local.is_empty() {
        let pack_handler = tokio::spawn(cerebro::get_packs(Some(args.offline)));
        let card_handler = tokio::spawn(cerebro::get_cards(Some(args.offline)));
        let pack_map: HashMap<Uuid, cerebro::Pack> = pack_handler
            .await
            .unwrap()
            .unwrap()
            .into_iter()
            .map(|pack| (pack.id.clone(), pack))
            .collect();
        let set_map: HashMap<Uuid, cerebro::Set> = tokio::spawn(cerebro::get_sets(Some(args.offline)))
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
    if let Some(download_path) = args.download {
        let download_server = &args
            .download_server
            .clone()
            .unwrap_or_else(|| String::from(DEFAULT_DOWNLOAD_SERVER));
        let client = reqwest::Client::new();
        let new_cards = stream::iter(cards)
            .map(|card| {
                let client = &client;
                let download_path = &download_path;
                async move {
                    let file_path = Path::new(&card.image_url);

                    fs::create_dir_all(download_path.join(file_path.parent().unwrap()))
                        .await
                        .unwrap();
                    let new_image_path = download_path.join(file_path);
                    if !new_image_path.as_path().exists() {
                        let resp = client.get(&card.image_url).send().await.unwrap();
                        let contents = resp.bytes().await.unwrap();
                        fs::write(new_image_path.as_path(), contents).await.unwrap();
                    }
                    let mut new_card = card.clone();
                    new_card.image_url = format!("{}/{}", download_server, &card.image_url);

                    new_card
                }
            })
            .buffered(CONCURRENT_REQUESTS);
        cards = new_cards.collect().await;
    }
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
