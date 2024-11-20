use tauri::command;

use super::{actions, Directory};

#[command]
pub fn add_directory(path: &str) -> Result<(), String> {
    match actions::add_directory(path) {
        Ok(id) => {
            println!("Directory added successfully with ID: {id}");
            Ok(())
        }
        Err(error) => {
            eprintln!("Error when inserting directory: {error}, path: {path}");
            Err("Failed to add directory".into())
        }
    }
}

#[command]
pub fn remove_directory(id: u32) -> bool {
    match actions::remove_directory(id) {
        Ok(_) => true,
        Err(_) => false,
    }
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
