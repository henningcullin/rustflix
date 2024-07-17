use tauri::command;

#[command]
pub fn add_directory(path: &str) -> bool {
    match super::actions::add_directory(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[command]
pub async fn select_directory() -> Option<String> {
    match super::actions::select_directory().await {
        Some(path) => Some(path.display().to_string()),
        None => None,
    }
}
