use clap::{Parser, Subcommand};

mod feed;

#[derive(Parser, Debug)]
#[command(name = "xml-feed-fetcher")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Fetch posts from the XML feed defined in .env and print their titles
    Fetch,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Fetch => feed::fetch_and_print(),
    }
}
