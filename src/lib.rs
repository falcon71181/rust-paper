use anyhow::{Context, Result};
use futures::{stream::FuturesUnordered, StreamExt};
use lazy_static::lazy_static;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{create_dir_all, File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;
use tokio::time::sleep;

mod config;
mod helper;
mod lock;

use lock::LockFile;

const WALLHEAVEN_API: &str = "https://wallhaven.cc/api/v1/w";

lazy_static! {
    static ref MAX_RETRY: u32 = 3;
}

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

        tokio::try_join!(
            create_dir_all(&config_folder),
            create_dir_all(&config.save_location)
        )?;

        let wallpapers_list_file_location = config_folder.join("wallpapers.lst");
        let wallpapers = load_wallpapers(&wallpapers_list_file_location).await?;

        let lock_file = Arc::new(Mutex::new(config.integrity.then(LockFile::default)));

        Ok(Self {
            config,
            config_folder,
            wallpapers,
            wallpapers_list_file_location,
            lock_file,
        })
    }

    pub async fn sync(&self) -> Result<()> {
        let tasks: FuturesUnordered<_> = self
            .wallpapers
            .iter()
            .map(|wallpaper| {
                let config = self.config.clone();
                let lock_file = Arc::clone(&self.lock_file);
                let wallpaper = wallpaper.clone();

                tokio::spawn(
                    async move { process_wallpaper(&config, &lock_file, &wallpaper).await },
                )
            })
            .collect();

        tasks
            .for_each(|result| async {
                if let Err(e) = result.expect("Task panicked") {
                    eprintln!("   Error processing wallpaper: {}", e);
                }
            })
            .await;

        Ok(())
    }

    pub async fn add(&mut self, new_wallpapers: &mut Vec<String>) -> Result<()> {
        *new_wallpapers = new_wallpapers
            .iter()
            .map(|wall| {
                if helper::is_url(wall) {
                    wall.split('/')
                        .last()
                        .unwrap_or_default()
                        .split('?')
                        .next()
                        .unwrap_or_default()
                        .to_string()
                } else {
                    wall.to_string()
                }
            })
            .collect();

        self.wallpapers
            .extend(new_wallpapers.iter().flat_map(|s| helper::to_array(s)));
        self.wallpapers.sort_unstable();
        self.wallpapers.dedup();
        update_wallpaper_list(&self.wallpapers, &self.wallpapers_list_file_location).await
    }
}

async fn update_wallpaper_list(list: &[String], file_path: &Path) -> Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(file_path)
        .await?;

    let mut writer = BufWriter::new(file);

    for wallpaper in list {
        writer.write_all(wallpaper.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    writer.flush().await?;
    Ok(())
}

async fn process_wallpaper(
    config: &config::Config,
    lock_file: &Arc<Mutex<Option<LockFile>>>,
    wallpaper: &str,
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

    let wallhaven_img_link = format!("{}/{}", WALLHEAVEN_API, wallpaper.trim());
    let curl_data = retry_get_curl_content(&wallhaven_img_link).await?;
    let res: Value = serde_json::from_str(&curl_data)?;

    if let Some(error) = res.get("error") {
        eprintln!("Error : {}", error);
        return Err(anyhow::anyhow!("   API error: {}", error));
    }

    let image_location = download_and_save(&res, wallpaper, &config.save_location).await?;

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
        lines.extend(helper::to_array(&line));
    }

    Ok(lines)
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

async fn download_and_save(api_data: &Value, id: &str, save_location: &str) -> Result<String> {
    let img_link = api_data
        .get("data")
        .and_then(|data| data.get("path"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("   Failed to get image link from API response"))?;
    helper::download_image(&img_link, id, save_location).await
}

async fn retry_get_curl_content(url: &str) -> Result<String> {
    for retry_count in 0..*MAX_RETRY {
        match helper::get_curl_content(url).await {
            Ok(content) => return Ok(content),
            Err(e) if retry_count + 1 < *MAX_RETRY => {
                eprintln!(
                    "Error fetching content (attempt {}): {}. Retrying...",
                    retry_count + 1,
                    e
                );
                sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
