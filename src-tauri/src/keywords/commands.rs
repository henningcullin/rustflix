use tauri::command;

use super::{actions, Keyword};

#[command]
pub fn get_all_keywords() -> Result<Vec<Keyword>, String> {
    actions::get_all_keywords().map_err(|e| {
        eprintln!("Error when getting all keywords {e}, {e:?}");
        "Failed to retrieve keywords".to_string()
    })
}

#[command]
pub fn create_keyword(film_id: i32, keyword_id: i32) -> Result<(), String> {
    match actions::create_keyword(film_id, keyword_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Keyword was not created".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!(
                "Error when creating keyword {error}, film_id: {film_id}, keyword_id: {keyword_id}"
            );
            Err("Failed to create keyword".into())
        }
    }
}

#[command]
pub fn delete_keyword(film_id: i32, keyword_id: i32) -> Result<(), String> {
    match actions::delete_keyword(film_id, keyword_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Keyword was not deleted".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!(
                "Error when deleting keyword {error}, film_id: {film_id}, keyword_id: {keyword_id}"
            );
            Err("Failed to delete keyword".into())
        }
    }
}
