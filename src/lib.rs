mod config;
use anyhow::{anyhow, Result};
use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufRead, BufReader};
use users::get_current_username;

pub struct RustPaper {
    config: config::Config,
    config_folder: String,
    wallpapers: Vec<String>,
    wallpapers_list_file_location: String,
}

impl RustPaper {
    pub fn new() -> Result<Self> {
        let config: config::Config =
            confy::load("rust-paper", "config").expect("Failed to load configuration");
        let username = get_current_username()
            .expect("Failed to get username")
            .to_str()
            .expect("Failed to convert username to string")
            .to_string();
        let config_folder = format!("/home/{}/.config/rust-paper", username);
        create_dir_all(&config.save_location)?;
        let wallpapers_list_file_location = format!("{}/wallpapers.lst", config_folder);
        let wallpapers_list_file = OpenOptions::new()
            .create(true)
            .read(true)
            .open(&wallpapers_list_file_location)?;
        let buffer_reader = BufReader::new(&wallpapers_list_file);
        let mut wallpapers: Vec<String> = vec![];

        for line in buffer_reader.lines() {
            let wallpaper = line?;
            wallpapers.push(wallpaper);
        }

        Ok(Self {
            config: config,
            config_folder: config_folder,
            wallpapers: wallpapers,
            wallpapers_list_file_location: wallpapers_list_file_location,
        })
    }
}
