use tauri::command;

use super::{actions, Genre};

#[command]
pub fn get_all_genres() -> Result<Vec<Genre>, String> {
    actions::get_all_genres().map_err(|e| {
        eprintln!("Error when getting all genres {e}, {e:?}");
        "Failed to retrieve genres".to_string()
    })
}
