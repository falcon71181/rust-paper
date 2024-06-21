use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};
use users::get_current_username;

#[derive(Debug, Serialize, Deserialize)]
struct LockEntry {
    image_id: String,
    image_location: String,
    sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    entries: Vec<LockEntry>,
}

impl Default for LockFile {
    fn default() -> Self {
        match LockFile::try_default() {
            Ok(lock_file) => lock_file,
            Err(_) => LockFile::new(),
        }
    }
}

impl LockFile {
    pub fn new() -> Self {
        LockFile {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, image_id: String, image_location: String, sha256: String) -> Result<()> {
        let username = get_current_username()
            .ok_or_else(|| anyhow!("Failed to get username"))?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert username to string"))?
            .to_string();
        let lock_file_location = format!("/home/{}/.config/rust-paper/wallpaper.lock", username);

        self.entries.push(LockEntry {
            image_id,
            image_location,
            sha256,
        });

        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&lock_file_location)?;

        let writer = BufWriter::new(lock_file);
        serde_json::to_writer(writer, &self)?;
        Ok(())
    }

    fn try_default() -> Result<Self> {
        let username = get_current_username()
            .ok_or_else(|| anyhow!("Failed to get username"))?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert username to string"))?
            .to_string();

        let lock_file_location = format!("/home/{}/.config/rust-paper/wallpaper.lock", username);

        if Path::new(&lock_file_location).exists() {
            let lock_file = File::open(&lock_file_location)?;
            let buffer_reader = BufReader::new(lock_file);
            let lock_file: LockFile = serde_json::from_reader(buffer_reader)?;
            Ok(lock_file)
        } else {
            Err(anyhow!(""))
        }
    }
}
