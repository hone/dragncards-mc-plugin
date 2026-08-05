mod cerebro;
mod cli;
mod dragncards;
mod local;
mod marvelcdb;
mod rules;

use clap::Parser;
use cli::DragncardsMcCli;

#[tokio::main]
async fn main() {
    match DragncardsMcCli::parse() {
        DragncardsMcCli::Database(args) => {
            cli::database::execute(args).await;
        }
        DragncardsMcCli::Decks(args) => {
            cli::decks::execute(args).await;
        }
    }
}
