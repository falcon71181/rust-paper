use anyhow::{anyhow, Error, Result};
use curl::easy::Easy;
use image::{self, guess_format, load_from_memory, DynamicImage, GenericImageView, ImageFormat};
use regex::Regex;
use sha2::{digest::Update, Digest, Sha256};
use std::{
    default::Default,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

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

pub fn get_curl_content(link: &str) -> Result<String> {
    let mut easy = Easy::new();
    easy.url(link).unwrap();

    let mut curl_data = Vec::new();

    {
        let mut transfer = easy.transfer();
        transfer
            .write_function(|data| Ok(curl_data.write_all(data).map(|_| data.len()).expect("400")))
            .unwrap();

        transfer.perform().unwrap();
    }

    match String::from_utf8(curl_data) {
        Ok(string) => Ok(string),
        Err(_) => Err(anyhow!("Unable to curl web page")),
    }
}

pub fn scrape_img_link(curl_data: String) -> Result<String> {
    let regex_pattern = r#"<img[^>]*id="wallpaper"[^>]*src="([^">]+)""#;
    let regex = Regex::new(regex_pattern).unwrap();
    let mut links: Vec<String> = Vec::new();

    for cap in regex.captures_iter(curl_data.as_str()) {
        links.push(cap[1].to_string());
    }

    match links.len() {
        0 => Err(anyhow!("Unable to scrape img link")),
        _ => Ok(links.into_iter().next().unwrap()),
    }
}

pub fn calculate_sha256(file_path: &str) -> Result<String> {
    if !Path::new(file_path).exists() {
        return Err(anyhow!("File does not exist: {}", file_path));
    }

    let mut file = File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        Update::update(&mut hasher, &buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn download_image(url: &str, id: &str, save_location: &str) -> Result<DynamicImage> {
    let img_bytes = reqwest::blocking::get(url)
        .map_err(Error::new)?
        .bytes()
        .map_err(Error::new)?;
    let img = load_from_memory(&img_bytes).map_err(Error::new)?;
    let img_format = guess_format(&img_bytes).map_err(Error::new)?;
    let (width, height) = img.dimensions();
    println!("{:?} , {:?}, {:?}", width, height, img_format);

    let image_name = format!(
        "{}/{}.{}",
        save_location,
        id,
        get_img_extension(&img_format)
    );
    println!("{}", image_name);

    img.save_with_format(image_name, img_format)
        .map_err(Error::new)?;

    Ok(img)
}
