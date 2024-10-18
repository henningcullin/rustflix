use tauri::command;

use super::{actions, Genre};

#[command]
pub fn get_all_genres() -> Result<Vec<Genre>, String> {
    actions::get_all_genres().map_err(|e| {
        eprintln!("Error when getting all genres {e}, {e:?}");
        "Failed to retrieve genres".to_string()
    })
}

#[command]
pub fn add_genre_to_film(film_id: i32, genre_id: i32) -> Result<(), String> {
    match actions::add_genre_to_film(film_id, &genre_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Genre was not added".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!(
                "Error when adding genre to film {error}, film_id: {film_id}, genre: {genre_id}"
            );
            Err("Failed to add genre".into())
        }
    }
}

#[command]
pub fn remove_genre_from_film(film_id: i32, genre_id: i32) -> Result<(), String> {
    match actions::remove_genre_from_film(film_id, &genre_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Genre was not removed".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when removing genre from film {error}, film_id: {film_id}, genre: {genre_id}");
            Err("Failed to remove genre".into())
        }
    }
}
