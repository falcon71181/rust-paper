use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    default::Default,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

use crate::helper;

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
        let lock_file_location = helper::get_folder_path()
            .context("   Failed to get folder path")?
            .join("wallpaper.lock");

        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|entry| entry.image_id == image_id)
        {
            entry.image_location = image_location;
            entry.sha256 = sha256;
        } else {
            self.entries.push(LockEntry {
                image_id,
                image_location,
                sha256,
            });
        }

        let lock_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&lock_file_location)?;

        let writer = BufWriter::new(lock_file);
        serde_json::to_writer(writer, &self)?;

        Ok(())
    }

    pub fn contains(&self, image_id: &str, hash: &str) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.image_id == image_id && entry.sha256 == hash)
    }

    fn try_default() -> Result<Self> {
        let lock_file_location = helper::get_folder_path()
            .context("   Failed to get folder path")?
            .join("wallpaper.lock");

        if Path::new(&lock_file_location).exists() {
            let lock_file = File::open(&lock_file_location)?;
            let buffer_reader = BufReader::new(lock_file);
            let lock_file: LockFile = serde_json::from_reader(buffer_reader)?;
            Ok(lock_file)
        } else {
            Err(anyhow!("   Lock file does not exist"))
        }
    }
}
