pub mod database;
pub mod decks;

use clap::Parser;

#[derive(Parser)]
#[command(name = "dragncards-mc")]
#[command(bin_name = "dragncards-mc")]
pub enum DragncardsMcCli {
    // Build Database of cards
    Database(database::DatabaseArgs),
    Decks(decks::DecksArgs),
}
