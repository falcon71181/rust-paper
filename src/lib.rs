mod config;
mod helper;
mod lock;

use anyhow::{anyhow, Result};
use image::{self, DynamicImage};
use std::fs::{create_dir_all, File};
use std::io::{BufRead, BufReader};
use std::path::Path;
use users::get_current_username;

pub struct RustPaper {
    config: config::Config,
    config_folder: String,
    wallpapers: Vec<String>,
    wallpapers_list_file_location: String,
}

impl RustPaper {
    pub fn new() -> Result<Self> {
        let config: config::Config = confy::load("rust-paper", "config")
            .map_err(|e| anyhow!("Failed to load configuration: {}", e))?;
        let username = get_current_username()
            .ok_or_else(|| anyhow!("Failed to get username"))?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert username to string"))?
            .to_string();
        let config_folder = format!("/home/{}/.config/rust-paper", username);

        if !Path::new(&config_folder).exists() {
            create_dir_all(&config_folder)?;
        }

        if !Path::new(&config.save_location).exists() {
            create_dir_all(&config.save_location)?;
        }

        let wallpapers_list_file_location = format!("{}/wallpapers.lst", config_folder);
        let mut wallpapers: Vec<String> = vec![];

        if Path::new(&wallpapers_list_file_location).exists() {
            let wallpapers_list_file = File::open(&wallpapers_list_file_location)?;
            let buffer_reader = BufReader::new(&wallpapers_list_file);

            for line in buffer_reader.lines() {
                match line {
                    Ok(wallpaper) => wallpapers.push(wallpaper),
                    Err(e) => {
                        eprintln!("Error reading line: {}", e);
                        break;
                    }
                }
            }
        } else {
            let _ = File::create(&wallpapers_list_file_location)?;
        }

        Ok(Self {
            config,
            config_folder,
            wallpapers,
            wallpapers_list_file_location,
        })
    }

    pub fn sync(&mut self) -> Result<()> {
        // TODO: make a progress bar
        let link_config: &str = "https://wallhaven.cc/w";

        for wallpaper in &self.wallpapers {
            let wallhaven_img_link = format!("{}/{}", link_config, wallpaper.trim());
            match helper::get_curl_content(&wallhaven_img_link) {
                Ok(curl_data) => {
                    if let Err(e) =
                        download_and_save(curl_data, wallpaper, &self.config.save_location)
                    {
                        eprintln!(
                            "Failed to download and save wallpaper {}: {:?}",
                            wallpaper, e
                        );
                    }
                }
                Err(e) => eprintln!("Failed to get curl content for {}: {:?}", wallpaper, e),
            }
        }

        Ok(())
    }
}

fn download_and_save(curl_data: String, id: &str, save_location: &str) -> Result<DynamicImage> {
    match helper::scrape_img_link(curl_data) {
        Ok(img_link) => helper::download_image(&img_link, id, save_location),
        Err(e) => Err(anyhow!("{:?}", e)),
    }
}
