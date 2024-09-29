use tauri::command;

use super::{actions, Language};

#[command]
pub fn get_all_languages() -> Result<Vec<Language>, String> {
    actions::get_all_languages().map_err(|e| {
        eprintln!("Error when getting all languages {e}, {e:?}");
        "Failed to retrieve languages".to_string()
    })
}
