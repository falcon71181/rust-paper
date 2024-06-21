use rust_paper::RustPaper;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut rust_paper = RustPaper::new().expect("Unable to create rust-paper struct");
    if !args.is_empty() {
        let command = &args[0];

        match &command[..] {
            "sync" => rust_paper.sync().unwrap(),
            _ => rust_paper.sync().unwrap(),
        }
    }
}
