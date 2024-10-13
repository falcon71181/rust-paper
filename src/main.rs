mod helper;
mod lock;

use anyhow::Error;
use clap::{Parser, Subcommand};
use rust_paper::RustPaper;

#[derive(Parser)]
struct Cli {
    /// add wallpapers
    // #[arg(short, long, value_parser)]
    // add: Option<String>,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sync wallpapers
    Sync,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize RustPaper
    let rust_paper = RustPaper::new().await?;

    match cli.command {
        Command::Sync => {
            // Call the sync method
            rust_paper.sync().await?;
        }
    }

    Ok(())
}
