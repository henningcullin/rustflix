use image::codecs::jpeg::JpegEncoder;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::error::AppError;

pub async fn store_image(
    image_url: &String,
    file_name: &String,
    target_path: PathBuf,
) -> Result<(), AppError> {
    // Send a GET request to download the image
    let response = reqwest::get(image_url).await?;
    let bytes = response.bytes().await?;

    // Load the image from the downloaded bytes
    let img = image::load_from_memory(&bytes)?;

    // Create the full path where the file will be saved
    let file_path = target_path.join(format!("{}.jpg", file_name));

    // Create the file and a buffered writer
    let file = File::create(file_path)?;
    let writer = BufWriter::new(file);

    // Encode the image as JPEG and save it to the specified path
    let mut jpeg_encoder = JpegEncoder::new(writer);
    jpeg_encoder.encode_image(&img)?;

    Ok(())
}
