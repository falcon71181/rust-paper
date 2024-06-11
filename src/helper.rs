use anyhow::{anyhow, Result};
use curl::easy::Easy;
use image::ImageFormat;
use regex::Regex;
use std::io::Write;

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
