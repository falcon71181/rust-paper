mod lock;
use rust_paper::RustPaper;
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut rust_paper = RustPaper::new().expect("Unable to create rust-paper struct");
    // let _ = rust_paper.sync();
    let mut lock_config = lock::LockFile::default();
    lock_config.add(
        String::from("pp"),
        1080,
        1920,
        String::from("jpg"),
        String::from("nugno34gi4gi"),
    );
}
