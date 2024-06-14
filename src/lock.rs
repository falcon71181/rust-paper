use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use sha2::{digest::Update, Digest, Sha256};
use std::{fs::File, io::Read, path::Path};

#[derive(Debug, Serialize, Deserialize)]
struct LockEntry {
    image_id: String,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    entries: Vec<LockEntry>,
}

impl LockFile {
    // TODO: make default instead of new()
    // pub fn new() -> Self {
    //     LockFile {
    //         entries: Vec::new(),
    //     }
    // }

    pub fn add(&mut self, image_id: String, sha256: String) {
        self.entries.push(LockEntry { image_id, sha256 });
    }

    pub fn calculate_sha256(file_path: &str) -> Result<String> {
        if !Path::new(file_path).exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1024];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            Update::update(&mut hasher, &buffer[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
}
