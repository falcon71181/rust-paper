use curl::easy::Easy;
use image::ImageFormat;
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

pub fn get_curl_content(link: &str) -> String {
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

    String::from_utf8(curl_data).unwrap()
}
