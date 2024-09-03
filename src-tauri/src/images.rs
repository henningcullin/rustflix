use base64::engine::general_purpose;
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;
use tauri::command;

use crate::database::{get_avatar_path, get_cover_path};
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

    // Check if file already exists before attempting to create it
    if !std::fs::metadata(&file_path).is_ok() {
        // Create the file and a buffered writer
        let file = File::create(file_path)?;
        let writer = BufWriter::new(file);

        // Encode the image as JPEG and save it to the specified path
        let mut jpeg_encoder = JpegEncoder::new(writer);
        jpeg_encoder.encode_image(&img)?;
    }

    Ok(())
}

fn image_to_base64(path: PathBuf) -> Result<String, String> {
    match fs::read(&path) {
        Ok(image_data) => {
            let base64_data = general_purpose::STANDARD.encode(&image_data);
            let mime_type = if path.extension().unwrap_or_default() == "jpg" {
                "image/jpeg"
            } else {
                "image/png"
            };
            Ok(format!("data:{};base64,{}", mime_type, base64_data))
        }
        Err(e) => Err(format!("Failed to read image: {}", e)),
    }
}

#[command]
pub fn get_avatar(id: u32) -> Result<String, String> {
    let path = get_avatar_path();
    let image_path = path.join(format!("{}.jpg", id));
    image_to_base64(image_path)
}

#[command]
pub fn get_cover(id: u32) -> Result<String, String> {
    let path = get_cover_path();
    let image_path = path.join(format!("{}.jpg", id));
    image_to_base64(image_path)
}
