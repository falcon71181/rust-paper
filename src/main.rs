use rust_paper::RustPaper;
use std::{env, fs::File, io::BufReader};
mod helper;
mod lock;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut rust_paper = RustPaper::new().await.unwrap();
    rust_paper.sync().await.unwrap();
    // println!("{:?}", helper::get_folder_path().join("wallpaper.lock"));
    // println!("{:?}", helper::get_home_location());
}
