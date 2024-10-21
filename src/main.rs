mod helper;
mod lock;

use anyhow::Error;
use clap::{Parser, Subcommand};
use rust_paper::RustPaper;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Sync wallpapers
    Sync,

    /// add wallpapers
    Add {
        #[arg(required = true)]
        paths: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize RustPaper
    let mut rust_paper = RustPaper::new().await?;

    match cli.command {
        Command::Sync => {
            // Call the sync method
            rust_paper.sync().await?;
        }
        Command::Add { mut paths } => {
            // Call the add method
            rust_paper.add(&mut paths).await?;
        }
    }

    Ok(())
}
