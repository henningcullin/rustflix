use tauri::command;

use super::actions;

#[command]
pub fn create_keyword(film_id: i32, keyword: String) -> Result<(), String> {
    match actions::create_keyword(film_id, &keyword) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Keyword was not created".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when creating keyword {error}, film_id: {film_id}, actor: {keyword}");
            Err("Failed to create keyword".into())
        }
    }
}
