use tauri::command;

use super::{actions, Language};

#[command]
pub fn get_all_languages() -> Result<Vec<Language>, String> {
    actions::get_all_languages().map_err(|e| {
        eprintln!("Error when getting all languages {e}, {e:?}");
        "Failed to retrieve languages".to_string()
    })
}

#[command]
pub fn add_language_to_film(film_id: i32, language_id: i32) -> Result<(), String> {
    match actions::add_language_to_film(film_id, &language_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Language was not added".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when adding language to film {error}, film_id: {film_id}, language: {language_id}");
            Err("Failed to add language".into())
        }
    }
}

#[command]
pub fn remove_language_from_film(film_id: i32, language_id: i32) -> Result<(), String> {
    match actions::remove_language_from_film(film_id, &language_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Language was not removed".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when removing language from film {error}, film_id: {film_id}, language: {language_id}");
            Err("Failed to remove language".into())
        }
    }
}
