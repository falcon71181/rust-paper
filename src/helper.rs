use image::ImageFormat;

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
