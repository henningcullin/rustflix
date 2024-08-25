use image::jpeg::JpegEncoder;
use image::ImageFormat;
use reqwest::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::error::AppError;

pub async fn download_and_convert_image(
    image_url: &str,
    file_name: &str,
    target_path: &str,
) -> Result<(), AppError> {
    // Send a GET request to download the image
    let response = reqwest::get(image_url).await?;
    let bytes = response.bytes().await?;

    // Load the image from the downloaded bytes
    let img = image::load_from_memory(&bytes)?;

    // Create the full path where the file will be saved
    let file_path = Path::new(target_path).join(file_name);

    // Create the file and a buffered writer
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    // Encode the image as JPEG and save it to the specified path
    let mut jpeg_encoder = JpegEncoder::new(writer);
    jpeg_encoder.encode_image(&img)?;

    Ok(())
}
