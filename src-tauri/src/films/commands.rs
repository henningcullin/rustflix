use super::Film;

use serde_json::json;
use tauri::command;

#[command]
pub fn get_all_films() -> Option<Vec<Film>> {
    match super::actions::get_all_films() {
        Ok(films) => Some(films),
        Err(_) => None,
    }
}

#[command]
pub fn get_all_files() -> Option<String> {
    match super::actions::get_all_files() {
        Ok(files) => Some(json!(files).to_string()),
        Err(_) => None,
    }
}
