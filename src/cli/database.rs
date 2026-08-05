use crate::cli::common;
use csv::WriterBuilder;
use futures::{stream, StreamExt};
use std::path::{Path, PathBuf};
use tokio::fs;

const CONCURRENT_REQUESTS: usize = 20;
const DEFAULT_DOWNLOAD_SERVER: &str = "http://localhost:5000";

#[derive(clap::Args)]
pub struct DatabaseArgs {
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub offline: bool,
    #[arg(long)]
    pub download: Option<PathBuf>,
    #[arg(long)]
    pub download_server: Option<String>,
    #[arg(long)]
    pub local: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub api: bool,
}

pub async fn execute(args: DatabaseArgs) {
    let mut cards: Vec<crate::dragncards::database::Card> =
        common::load_card_database(&args.local, args.api, args.offline)
            .await
            .into_iter()
            .map(|card| card.output)
            .collect();

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
        .unwrap_or_else(|| PathBuf::from("./marvelcdb.tsv"));
    let mut wtr = WriterBuilder::new()
        .delimiter(b'\t')
        .from_path(output)
        .unwrap();

    for card in cards {
        wtr.serialize(card).unwrap();
    }
}
