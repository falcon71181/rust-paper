use anyhow::{Context, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

mod config;
mod helper;
mod lock;

use lock::LockFile;

#[derive(Clone)]
pub struct RustPaper {
    config: config::Config,
    config_folder: PathBuf,
    wallpapers: Vec<String>,
    wallpapers_list_file_location: PathBuf,
    lock_file: Arc<Mutex<Option<LockFile>>>,
}

impl RustPaper {
    pub async fn new() -> Result<Self> {
        let config: config::Config =
            confy::load("rust-paper", "config").context("   Failed to load configuration")?;

        let config_folder = helper::get_folder_path().context("   Failed to get folder path")?;
        create_dir_all(&config_folder).await?;
        create_dir_all(&config.save_location).await?;

        let wallpapers_list_file_location = config_folder.join("wallpapers.lst");
        let wallpapers = Self::load_wallpapers(&wallpapers_list_file_location).await?;

        let lock_file = Arc::new(Mutex::new(config.integrity.then(LockFile::default)));

        Ok(Self {
            config,
            config_folder,
            wallpapers,
            wallpapers_list_file_location,
            lock_file,
        })
    }

    async fn load_wallpapers(file_path: &Path) -> Result<Vec<String>> {
        if !file_path.exists() {
            File::create(file_path).await?;
            return Ok(vec![]);
        }

        let file = File::open(file_path).await?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut lines_stream = reader.lines();

        while let Some(line) = lines_stream.next_line().await? {
            lines.push(line);
        }

        Ok(lines)
    }

    pub async fn sync(&self) -> Result<()> {
        let link_config = "https://wallhaven.cc/w";
        let self_arc = Arc::new(self.clone());

        let mut tasks = FuturesUnordered::new();

        for wallpaper in &self.wallpapers {
            let config = self_arc.config.clone();
            let lock_file = Arc::clone(&self_arc.lock_file);
            let wallpaper = wallpaper.clone();

            tasks.push(tokio::spawn(async move {
                process_wallpaper(&config, &lock_file, &wallpaper, link_config).await
            }));
        }

        while let Some(result) = tasks.next().await {
            result??;
        }

        Ok(())
    }
}

async fn process_wallpaper(
    config: &config::Config,
    lock_file: &Arc<Mutex<Option<LockFile>>>,
    wallpaper: &str,
    link_config: &str,
) -> Result<()> {
    let save_location = Path::new(&config.save_location);
    if let Some(existing_path) = find_existing_image(save_location, wallpaper).await? {
        if config.integrity {
            if check_integrity(&existing_path, &wallpaper, &lock_file).await? {
                println!(
                    "   Skipping {}: already exists and integrity check passed",
                    wallpaper
                );
                return Ok(());
            }
            println!(
                "   Integrity check failed for {}: re-downloading",
                wallpaper
            );
        } else {
            println!("   Skipping {}: already exists", wallpaper);
            return Ok(());
        }
    }

    let wallhaven_img_link = format!("{}/{}", link_config, wallpaper.trim());
    let curl_data = helper::get_curl_content(&wallhaven_img_link).await?;
    let image_location = download_and_save(curl_data, wallpaper, &config.save_location).await?;

    if config.integrity {
        let mut lock_file = lock_file.lock().await;
        if let Some(ref mut lock_file) = *lock_file {
            let image_sha256 = helper::calculate_sha256(&image_location).await?;
            lock_file.add(wallpaper.to_string(), image_location, image_sha256)?;
        }
    }

    println!("   Downloaded {}", wallpaper);
    Ok(())
}

async fn find_existing_image(save_location: &Path, wallpaper: &str) -> Result<Option<PathBuf>> {
    let mut entries = tokio::fs::read_dir(save_location).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.file_stem().and_then(|s| s.to_str()) == Some(wallpaper) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}

async fn check_integrity(
    existing_path: &Path,
    wallpaper: &str,
    lock_file: &Arc<Mutex<Option<LockFile>>>,
) -> Result<bool> {
    let lock_file = lock_file.lock().await;
    if let Some(ref lock_file) = *lock_file {
        let existing_image_sha256 = helper::calculate_sha256(existing_path).await?;
        Ok(lock_file.contains(wallpaper, &existing_image_sha256))
    } else {
        Ok(false)
    }
}

async fn download_and_save(curl_data: String, id: &str, save_location: &str) -> Result<String> {
    let img_link = helper::scrape_img_link(&curl_data).await?;
    helper::download_image(&img_link, id, save_location).await
}
