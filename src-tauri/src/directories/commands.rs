use tauri::command;

use super::Directory;

#[command]
pub fn add_directory(path: &str) -> Option<Directory> {
    match super::actions::add_directory(path) {
        Ok(directory) => Some(directory),
        Err(_) => None,
    }
}

#[command]
pub async fn select_directory() -> Option<String> {
    match super::actions::select_directory().await {
        Some(path) => Some(path.display().to_string()),
        None => None,
    }
}

#[command]
pub fn get_all_directories() -> Option<Vec<Directory>> {
    match super::actions::get_all_directories() {
        Ok(directories) => Some(directories),
        Err(_) => None,
    }
}
