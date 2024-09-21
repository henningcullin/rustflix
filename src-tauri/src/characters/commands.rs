use super::actions;
use tauri::command;

#[command]
pub fn delete_character(film_id: i32, actor: i32) -> Result<(), String> {
    match actions::delete_character(film_id, actor) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Character was not found in database".into()),
            _ => Ok(()),
        },
        Err(_) => Err("Error when deleting character".into()),
    }
}
