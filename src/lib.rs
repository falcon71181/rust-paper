mod config;
mod helper;
mod lock;

use anyhow::{anyhow, Context, Error, Result};
use futures::StreamExt;
use lock::LockFile;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::{read_dir, File};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::Mutex;

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
        let config: config::Config = confy::load("rust-paper", "config")
            .map_err(|e| anyhow!("   Failed to load configuration: {}", e))?;

        let config_folder = helper::get_folder_path().context("   Failed to get folder path")?;
        if !Path::new(&config_folder).exists() {
            create_dir_all(&config_folder)?;
        }

        if !Path::new(&config.save_location).exists() {
            create_dir_all(&config.save_location)?;
        }

        let wallpapers_list_file_location = config_folder.join("wallpapers.lst");
        let wallpapers = Self::load_wallpapers(&wallpapers_list_file_location).await?;

        let lock_file = Arc::new(Mutex::new(if config.integrity {
            Some(LockFile::default())
        } else {
            None
        }));

        Ok(Self {
            config,
            config_folder,
            wallpapers,
            wallpapers_list_file_location,
            lock_file,
        })
    }

    async fn load_wallpapers(file_path: &Path) -> Result<Vec<String>> {
        if !tokio::fs::try_exists(file_path).await? {
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
        let link_config: &str = "https://wallhaven.cc/w";
        let self_arc = Arc::new(self.clone());

        let tasks = futures::stream::iter(self.wallpapers.iter().cloned().map(|wallpaper| {
            let config = self_arc.config.clone(); // Clone the config
            let lock_file = Arc::clone(&self_arc.lock_file);

            tokio::spawn(async move {
                let save_location = Path::new(&config.save_location);
                let existing_image = find_existing_image(&save_location, &wallpaper).await?;

                if let Some(existing_path) = existing_image {
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
                match helper::get_curl_content(&wallhaven_img_link).await {
                    Ok(curl_data) => {
                        match download_and_save(curl_data, &wallpaper, &config.save_location).await
                        {
                            Ok(image_location) => {
                                if config.integrity {
                                    let mut lock_file = lock_file.lock().await;
                                    if let Some(ref mut lock_file) = *lock_file {
                                        let image_location_clone = image_location.clone();
                                        let image_sha256 =
                                            helper::calculate_sha256(&image_location_clone).await?;
                                        let _ = lock_file.add(
                                            wallpaper.clone(),
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
                Ok(())
            })
        }))
        .buffer_unordered(10) // Limit concurrent tasks
        .map(|r| r.map_err(Error::new)?)
        .collect::<Vec<Result<()>>>();

        for result in tasks.await {
            result?;
        }

        Ok(())
    }
}

async fn find_existing_image(save_location: &Path, wallpaper: &str) -> Result<Option<String>> {
    let mut entries = read_dir(save_location).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(file_stem) = path.file_stem() {
            if file_stem == wallpaper {
                return Ok(Some(path.to_string_lossy().into_owned()));
            }
        }
    }
    Ok(None)
}

async fn check_integrity(
    existing_path: &str,
    wallpaper: &str,
    lock_file: &Arc<Mutex<Option<LockFile>>>,
) -> Result<bool> {
    let lock_file = lock_file.lock().await;
    if let Some(ref lock_file) = *lock_file {
        let existing_path = existing_path.to_string();
        let existing_image_sha256 = helper::calculate_sha256(&existing_path).await?;
        Ok(lock_file.contains(wallpaper, &existing_image_sha256))
    } else {
        Ok(false)
    }
}

async fn download_and_save(curl_data: String, id: &str, save_location: &str) -> Result<String> {
    match helper::scrape_img_link(curl_data) {
        Ok(img_link) => helper::download_image(&img_link, id, save_location).await,
        Err(e) => Err(anyhow!("   {:?}", e)),
    }
}
