use base64::engine::general_purpose;
use base64::Engine;
use image::ImageEncoder;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::PathBuf;

use super::models::ImageSize;
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

    // Determine the MIME type (JPEG or PNG) based on the input
    let mime_type = match image::guess_format(&bytes)? {
        image::ImageFormat::Png => "png",
        _ => "jpg", // Default to JPEG if unknown
    };

    for size in [
        ImageSize::Small,
        ImageSize::Medium,
        ImageSize::Large,
        ImageSize::Full,
    ] {
        // Resize the image for each size
        let resized_image = if let Some((w, h)) = size.dimensions() {
            img.resize(w, h, image::imageops::FilterType::Triangle)
        } else {
            img.clone() // Full size
        };

        // Create the directory for the size if it doesn't exist
        let size_path = target_path.join(size.as_str());
        fs::create_dir_all(&size_path)?;

        // Construct the file path
        let file_path = size_path.join(format!("{}.{}", file_name, mime_type));

        // Check if file already exists before attempting to create it
        if !file_path.exists() {
            let file = File::create(&file_path)?;
            let writer = BufWriter::new(file);

            match mime_type {
                "png" => {
                    let encoder = image::codecs::png::PngEncoder::new(writer);
                    encoder.write_image(
                        resized_image.as_bytes(),
                        resized_image.width(),
                        resized_image.height(),
                        resized_image.color().into(),
                    )?;
                }
                _ => {
                    let mut encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(writer, 80);
                    encoder.encode_image(&resized_image)?;
                }
            }
        }
    }

    Ok(())
}

pub fn image_to_base64(path: PathBuf) -> Result<String, String> {
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
