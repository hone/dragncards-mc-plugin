use clap::Parser;

#[derive(Parser)]
#[command(name = "dragncards-mc")]
#[command(bin_name = "dragncards-mc")]
pub enum DragncardsMcCli {
    // Build Database of cards
    Database(DatabaseArgs),
    Decks,
}

#[derive(clap::Args)]
pub struct DatabaseArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
}
