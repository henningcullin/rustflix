use base64::engine::general_purpose;
use base64::Engine;
use image::{GenericImageView, ImageEncoder};
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
    let (original_width, original_height) = img.dimensions();

    // Determine the MIME type (JPEG or PNG) based on the input
    let mime_type = match image::guess_format(&bytes)? {
        image::ImageFormat::Png => "png",
        _ => "jpg", // Default to JPEG if unknown
    };

    // Helper function to determine directory and size constraints
    let size_configs = [
        (ImageSize::Small, 400, 400),
        (ImageSize::Medium, 800, 800),
        (ImageSize::Large, 1400, 1400),
        (ImageSize::Full, u32::MAX, u32::MAX), // Full size has no limit
    ];

    for (size, max_width, max_height) in size_configs {
        // Check if the image qualifies for this size category
        if original_width <= max_width && original_height <= max_height {
            // Resize the image while maintaining aspect ratio (only if not Full size)
            let resized_image = if size != ImageSize::Full {
                let (new_width, new_height) =
                    calculate_dimensions(original_width, original_height, max_width, max_height);
                img.resize(new_width, new_height, image::imageops::FilterType::Triangle)
            } else {
                img.clone()
            };

            // Create the directory for the size if it doesn't exist
            let size_path = target_path.join(size.as_str());
            fs::create_dir_all(&size_path)?;

            // Construct the file path
            let file_path = size_path.join(format!("{}.{}", file_name, mime_type));

            // Write the resized image to the appropriate directory
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
/// Calculate resized dimensions while maintaining the aspect ratio.
fn calculate_dimensions(
    original_width: u32,
    original_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
    let aspect_ratio = original_width as f32 / original_height as f32;
    if original_width > max_width || original_height > max_height {
        if aspect_ratio > 1.0 {
            // Landscape image
            let width = max_width;
            let height = (max_width as f32 / aspect_ratio).round() as u32;
            (width, height)
        } else {
            // Portrait or square image
            let height = max_height;
            let width = (max_height as f32 * aspect_ratio).round() as u32;
            (width, height)
        }
    } else {
        // No resizing needed if the image fits within the max dimensions
        (original_width, original_height)
    }
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
