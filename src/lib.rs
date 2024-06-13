mod config;
use anyhow::{anyhow, Result};
use users::get_current_username;

pub struct RustPaper {
    config: config::Config,
    config_folder: String,
    wallpapers: Vec<String>,
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
        let wallpapers: Vec<String> = vec![];

        Ok(Self {
            config: config,
            config_folder: config_folder,
            wallpapers: wallpapers,
        })
    }
}
