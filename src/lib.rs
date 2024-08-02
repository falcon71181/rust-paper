mod config;
mod helper;
mod lock;

use anyhow::{anyhow, Result};
use lock::LockFile;
use std::ffi::OsStr;
use std::fs::{create_dir_all, read_dir, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct RustPaper {
    config: config::Config,
    config_folder: PathBuf,
    wallpapers: Vec<String>,
    wallpapers_list_file_location: PathBuf,
    lock_file: Option<LockFile>,
}

impl RustPaper {
    pub fn new() -> Result<Self> {
        let config: config::Config = confy::load("rust-paper", "config")
            .map_err(|e| anyhow!("   Failed to load configuration: {}", e))?;

        let config_folder = helper::get_folder_path();
        if !Path::new(&config_folder).exists() {
            create_dir_all(&config_folder)?;
        }

        if !Path::new(&config.save_location).exists() {
            create_dir_all(&config.save_location)?;
        }

        let wallpapers_list_file_location = config_folder.join("wallpapers.lst");
        let mut wallpapers: Vec<String> = vec![];

        if Path::new(&wallpapers_list_file_location).exists() {
            let wallpapers_list_file = File::open(&wallpapers_list_file_location)?;
            let buffer_reader = BufReader::new(&wallpapers_list_file);

            for line in buffer_reader.lines() {
                match line {
                    Ok(wallpaper) => wallpapers.push(wallpaper),
                    Err(e) => {
                        eprintln!("   Error reading line: {}", e);
                        break;
                    }
                }
            }
        } else {
            let _ = File::create(&wallpapers_list_file_location)?;
        }

        let mut lock_file: Option<LockFile> = None;
        if config.integrity {
            lock_file = Some(LockFile::default());
        }

        Ok(Self {
            config,
            config_folder,
            wallpapers,
            wallpapers_list_file_location,
            lock_file,
        })
    }

    pub fn sync(&mut self) -> Result<()> {
        // TODO: make a progress bar
        // TODO: use multithreading inorder to make it fast
        let link_config: &str = "https://wallhaven.cc/w";

        for wallpaper in &self.wallpapers {
            let mut image_exists = false;
            let mut existing_image_path = String::new();

            // Check if the image already exists in the save location
            for entry in read_dir(&self.config.save_location)? {
                let path = entry?.path();
                if let Some(file_stem) = path.file_stem() {
                    if file_stem == OsStr::new(wallpaper) {
                        image_exists = true;
                        existing_image_path = path.to_string_lossy().into_owned();
                        break;
                    }
                }
            }

            if image_exists {
                if self.config.integrity {
                    let existing_image_sha256 = helper::calculate_sha256(&existing_image_path)?;
                    if let Some(ref lock_file) = self.lock_file {
                        if lock_file.contains(wallpaper, &existing_image_sha256) {
                            println!(
                                "   Skipping {}: already exists and integrity check passed",
                                wallpaper
                            );
                            continue;
                        } else {
                            println!(
                                "   Integrity check failed for {}: re-downloading",
                                wallpaper
                            );
                        }
                    }
                } else {
                    println!("   Skipping {}: already exists", wallpaper);
                    continue;
                }
            }

            let wallhaven_img_link = format!("{}/{}", link_config, wallpaper.trim());
            match helper::get_curl_content(&wallhaven_img_link) {
                Ok(curl_data) => {
                    match download_and_save(curl_data, wallpaper, &self.config.save_location) {
                        Ok(image_location) => {
                            if self.config.integrity {
                                if let Some(ref mut lock_file) = self.lock_file {
                                    let image_sha256 = helper::calculate_sha256(&image_location)?;
                                    let _ = lock_file.add(
                                        wallpaper.to_string(),
                                        image_location.clone(),
                                        image_sha256,
                                    );
                                }
                            }
                            println!("   Downloaded {}", wallpaper);
                        }
                        Err(e) => {
                            eprintln!("   Failed to download and save {}: {:?}", wallpaper, e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!(" 󰤫  Failed to get curl content for {}: {:?}", wallpaper, e);
                }
            }
        }
        Ok(())
    }
}

fn download_and_save(curl_data: String, id: &str, save_location: &str) -> Result<String> {
    match helper::scrape_img_link(curl_data) {
        Ok(img_link) => helper::download_image(&img_link, id, save_location),
        Err(e) => Err(anyhow!("{:?}", e)),
    }
}
