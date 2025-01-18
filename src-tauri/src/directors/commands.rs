use tauri::command;

use crate::{create_command, delete_command};

use super::actions;

#[command]
pub fn add_director_to_film(film_id: i32, person_id: i32) -> Result<(), String> {
    create_command!(
        actions::add_director_to_film(film_id, person_id),
        "Director ({person_id}) added successfully to film ({film_id}), film_directors id ({id})",
        "Error when adding director ({person_id}) to film ({film_id}), error: ({error})",
        "Failed to add director to film"
    )
}

#[command]
pub fn remove_director_from_film(film_id: i32, person_id: i32) -> Result<(), String> {
    delete_command!(
        actions::remove_director_from_film(film_id, person_id),
        "Director ({person_id}) successfully removed from film ({film_id})",
        "Error when removing director ({person_id}) from film ({film_id}), error: {error}",
        "Director ({person_id}) was not removed from film ({film_id})",
        "Failed to remove director from film"
    )
}
