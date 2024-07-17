#[tauri::command]
pub fn add_directory(path: &str) -> bool {
    match super::actions::add_directory(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}
