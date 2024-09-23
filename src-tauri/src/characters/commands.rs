use super::actions;
use tauri::command;

#[command]
pub fn delete_character(film_id: i32, actor: i32) -> Result<(), String> {
    match actions::delete_character(film_id, actor) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Character was not found in database".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when deleting character {error}, film_id: {film_id}, actor: {actor}");
            Err("Failed to delete character".into())
        }
    }
}

#[command]
pub fn update_character(
    film_id: i32,
    actor: i32,
    new_description: String,
    new_actor: i32,
) -> Result<(), String> {
    match actions::update_character(film_id, actor, &new_description, new_actor) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Character was not found in database".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!("Error when updating character {error}, film_id {film_id}, actor {actor}, new_description {new_description}, new_actor {new_actor}");
            Err("Failed to update character".into())
        }
    }
}
