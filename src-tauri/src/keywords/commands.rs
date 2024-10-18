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

#[command]
pub fn add_keyword_to_film(film_id: i32, keyword_id: i32) -> Result<(), String> {
    match actions::add_keyword_to_film(film_id, &keyword_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Keywords was not added".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!(
                "Error when adding keyword to film {error}, film_id: {film_id}, keyword: {keyword_id}"
            );
            Err("Failed to add keyword".into())
        }
    }
}

#[command]
pub fn remove_keyword_from_film(film_id: i32, keyword_id: i32) -> Result<(), String> {
    match actions::remove_keyword_from_film(film_id, &keyword_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Keywords was not removed".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when removing keyword from film {error}, film_id: {film_id}, keyword: {keyword_id}");
            Err("Failed to remove keyword".into())
        }
    }
}
