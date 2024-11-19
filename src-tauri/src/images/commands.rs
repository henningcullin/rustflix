use tauri::command;

use super::{actions::image_to_base64, ImageSize};
use crate::database::{get_avatar_path, get_cover_path};

#[command]
pub fn get_avatar(id: u32, size: Option<&str>) -> Result<String, String> {
    let size_enum = ImageSize::from(size);
    let path = get_avatar_path().join(size_enum.as_str());
    let image_path = path.join(format!("{}.jpg", id)); // Assume JPEG for simplicity

    image_to_base64(image_path)
}

#[command]
pub fn get_cover(id: u32, size: Option<&str>) -> Result<String, String> {
    let size_enum = ImageSize::from(size);
    let path = get_cover_path().join(size_enum.as_str());
    let image_path = path.join(format!("{}.jpg", id)); // Assume JPEG for simplicity

    image_to_base64(image_path)
}
