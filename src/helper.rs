use anyhow::{anyhow, Context, Error, Result};
use image::{self, guess_format, load_from_memory, ImageFormat};
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::{fs::File, io::AsyncReadExt};

pub fn get_img_extension(format: &ImageFormat) -> &'static str {
    let extensions: HashMap<ImageFormat, &'static str> = [
        (ImageFormat::Png, "png"),
        (ImageFormat::Jpeg, "jpeg"),
        (ImageFormat::Gif, "gif"),
        (ImageFormat::WebP, "webp"),
        (ImageFormat::Pnm, "pnm"),
        (ImageFormat::Tiff, "tiff"),
        (ImageFormat::Tga, "tga"),
        (ImageFormat::Dds, "dds"),
        (ImageFormat::Bmp, "bmp"),
        (ImageFormat::Ico, "ico"),
        (ImageFormat::Hdr, "hdr"),
    ]
    .iter()
    .cloned()
    .collect();

    extensions.get(format).unwrap_or(&"jpg")
}

pub async fn get_curl_content(link: &str) -> Result<String> {
    let client = Client::new();

    let response = client.get(link).send().await?;

    let body = response.text().await?;

    Ok(body)
}

pub async fn calculate_sha256(file_path: impl AsRef<Path>) -> Result<String> {
    let file_path = file_path.as_ref();

    if !file_path.exists() {
        return Err(anyhow!(" 󱀷  File does not exist: {}", file_path.display()));
    }

    let mut file = File::open(file_path)
        .await
        .with_context(|| format!(" 󱀷  Failed to open file: {}", file_path.display()))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let n = file
            .read(&mut buffer)
            .await
            .with_context(|| format!(" 󱀷  Failed to read file: {}", file_path.display()))?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn download_image(url: &str, id: &str, save_location: &str) -> Result<String> {
    let url = reqwest::Url::parse(url)?;
    let img_bytes = Client::new().get(url).send().await?.bytes().await?;

    let img = load_from_memory(&img_bytes)?;
    let img_format = guess_format(&img_bytes)?;
    let image_name = format!(
        "{}/{}.{}",
        save_location,
        id,
        get_img_extension(&img_format)
    );

    img.save_with_format(&image_name, img_format)?;

    Ok(image_name)
}

pub fn get_home_location() -> String {
    dirs::home_dir()
        .map(|path| path.to_str().unwrap_or_default().to_string())
        .unwrap_or_else(|| "~".to_string())
}

pub fn get_folder_path() -> Result<PathBuf> {
    let path = confy::get_configuration_file_path("rust-paper", "config").map_err(Error::new)?;
    if let Some(parent) = path.parent() {
        Ok(parent.to_path_buf())
    } else {
        Ok(PathBuf::new())
    }
}

pub fn to_array(comma_separated_values: &str) -> Vec<String> {
    comma_separated_values
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

pub fn is_url(input: &str) -> bool {
    url::Url::parse(input).is_ok()
}
