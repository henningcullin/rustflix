use tauri::command;

use crate::{create_command, delete_command};

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
    create_command!(
        actions::add_genre_to_film(film_id, &genre_id),
        "Genre ({genre_id}) added successfully to film ({film_id}), film_genres id ({id})",
        "Error when adding genre ({genre_id}) to film ({film_id}), error: ({error})",
        "Failed to add genre to film"
    )
}

#[command]
pub fn remove_genre_from_film(film_id: i32, genre_id: i32) -> Result<(), String> {
    delete_command!(
        actions::remove_genre_from_film(film_id, &genre_id),
        "Genre ({genre_id}) successfully removed from film ({film_id})",
        "Error when removing genre ({genre_id}) from film ({film_id}), error: {error}",
        "Genre ({genre_id}) was not removed from film ({film_id})",
        "Failed to remove genre from film"
    )
}
