mod helper;
mod lock;

use clap::{Parser, Subcommand};
use rust_paper::RustPaper;
use std::env;
use std::fs::File;
use std::io::BufReader;

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
async fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Initialize RustPaper
    let rust_paper = RustPaper::new().await.unwrap();

    match cli.command {
        Command::Sync => {
            // Call the sync method
            rust_paper.sync().await.unwrap();
        }
    }

    // println!("{:?}", helper::get_folder_path().join("wallpaper.lock"));
    // println!("{:?}", helper::get_home_location());
}
