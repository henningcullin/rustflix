use tauri::command;

use crate::{create_command, delete_command, update_command};

use super::{actions, Directory};

#[command]
pub fn create_directory(path: &str) -> Result<(), String> {
    create_command!(
        actions::create_directory(path),
        "Directory added successfully with id: {id}",
        "Error when inserting directory: {error}, path: {path}",
        "Failed to add directory"
    )
}

#[command]
pub fn delete_directory(id: i32) -> Result<(), String> {
    delete_command!(
        actions::delete_directory(id),
        "Directory deleted successfully with id: {id}",
        "Error when deleting directory: {error}, id: {id}",
        "Directory was not deleted: id: {id}",
        "Failed to remove directory"
    )
}

#[command]
pub fn update_directory(id: i32, path: &str) -> Result<(), String> {
    update_command!(
        actions::update_directory(id, path),
        "Directory updated successfully with id: {id}, path: {path}",
        "Error when updating directory: {error}, id: {id}, path: {path}",
        "Directory was not updated: id: {id}, path: {path}",
        "Failed to update directory"
    )
}

#[command]
pub async fn select_directory() -> Option<String> {
    match actions::select_directory().await {
        Some(path) => Some(path.display().to_string()),
        None => None,
    }
}

#[command]
pub fn get_all_directories() -> Option<Vec<Directory>> {
    match actions::get_all_directories() {
        Ok(directories) => Some(directories),
        Err(_) => None,
    }
}
