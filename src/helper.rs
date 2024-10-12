use anyhow::{anyhow, Error, Result};
use image::{self, guess_format, load_from_memory, GenericImageView, ImageFormat};
use regex::Regex;
use reqwest::Client;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio::io::AsyncReadExt;

pub fn get_img_extension(format: &ImageFormat) -> &str {
    match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpeg",
        ImageFormat::Gif => "gif",
        ImageFormat::WebP => "webp",
        ImageFormat::Pnm => "pnm",
        ImageFormat::Tiff => "tiff",
        ImageFormat::Tga => "tga",
        ImageFormat::Dds => "dds",
        ImageFormat::Bmp => "bmp",
        ImageFormat::Ico => "ico",
        ImageFormat::Hdr => "hdr",
        _ => "jpg",
    }
}

pub async fn get_curl_content(link: &str) -> Result<String> {
    let client = Client::new();

    let response = client.get(link).send().await?;

    let body = response.text().await?;

    Ok(body)
}

pub fn scrape_img_link(curl_data: String) -> Result<String> {
    let regex_pattern = r#"<img[^>]*id="wallpaper"[^>]*src="([^">]+)""#;
    let regex = Regex::new(regex_pattern).unwrap();
    let mut links: Vec<String> = Vec::new();

    for cap in regex.captures_iter(curl_data.as_str()) {
        links.push(cap[1].to_string());
    }

    match links.len() {
        0 => Err(anyhow!("   Unable to scrape img link")),
        _ => Ok(links.into_iter().next().unwrap()),
    }
}

pub async fn calculate_sha256(file_path: &str) -> Result<String> {
    if !Path::new(file_path).exists() {
        return Err(anyhow!(" 󱀷  File does not exist: {}", file_path));
    }

    let mut file = tokio::fs::File::open(file_path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub async fn download_image(url: &str, id: &str, save_location: &str) -> Result<String> {
    // Validate URL
    let url = reqwest::Url::parse(url).map_err(Error::new)?;

    let client = Client::new();

    // Fetch the image bytes asynchronously
    let img_bytes = client
        .get(url)
        .send()
        .await
        .map_err(Error::new)?
        .bytes()
        .await
        .map_err(Error::new)?;

    // Load the image from the fetched bytes
    let img = load_from_memory(&img_bytes).map_err(Error::new)?;

    // Guess the image format
    let img_format = guess_format(&img_bytes).map_err(Error::new)?;
    let (_width, _height) = img.dimensions();

    // Generate the image name with the appropriate format
    let image_name = format!(
        "{}/{}.{}",
        save_location,
        id,
        get_img_extension(&img_format)
    );

    // Save the image with the guessed format
    img.save_with_format(&image_name, img_format)
        .map_err(Error::new)?;

    Ok(image_name)
}

pub fn get_home_location() -> String {
    let utf8 = Command::new("sh")
        .arg("-c")
        .arg("echo $HOME")
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8(utf8.stdout)
        .expect("Unable to get home location")
        .trim_matches(&['\n', '\r'][..])
        .to_string();

    output
}

pub fn get_folder_path() -> Result<PathBuf> {
    let path = confy::get_configuration_file_path("rust-paper", "config").map_err(Error::new)?;
    if let Some(parent) = path.parent() {
        Ok(parent.to_path_buf())
    } else {
        Ok(PathBuf::new())
    }
}
