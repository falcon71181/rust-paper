mod helper;

use image::{self, guess_format, load_from_memory, DynamicImage, GenericImageView, ImageFormat};
use reqwest;
use std::error::Error;

fn download_image(url: &str) -> Result<DynamicImage, Box<dyn Error>> {
    let img_bytes = reqwest::blocking::get(url)?.bytes()?;
    let img = load_from_memory(&img_bytes)?;
    let img_format = guess_format(&img_bytes)?;
    let (width, height) = img.dimensions();
    println!("{:?} , {:?}, {:?}", width, height, img_format);

    let image_name = format!("{}.{}", "abcT", helper::get_img_extension(&img_format));

    // img.save_with_format(image_name, img_format)?;

    Ok(img)
}

fn main() {
    // if download_image(&"https://w.wallhaven.cc/full/5g/wallhaven-5gqmg7.jpg").is_ok() {
    //     println!("{}", "working");
    // } else {
    //     println!("{}", "NOPE u r dumb");
    // }
    println!(
        "{:?}",
        helper::get_curl_content(&"https://wallhaven.cc/w/5gqmg7")
    );
}
