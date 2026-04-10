use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

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
    // Log su file: errors in logs/errors.log, tutto il resto su stdout
    let file_appender = tracing_appender::rolling::never("logs", "errors.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("error"))
        .with_writer(file_writer)
        .with_ansi(false)
        .init();

    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().unwrap();
    match cli.command {
        Commands::Fetch => rt.block_on(xml_feed_fetcher::feed::fetch_and_print()),
    }
}
