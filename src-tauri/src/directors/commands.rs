use tauri::command;

use super::actions;

#[command]
pub fn add_director_to_film(film_id: i32, person_id: i32) -> Result<(), String> {
    match actions::add_director_to_film(film_id, person_id) {
        Ok(id) => {
            println!("Director added successfully with id: {id}");
            Ok(())
        }
        Err(error) => {
            eprintln!("Error when inserting director: {error}, film_id: {film_id}, person_id: {person_id}");
            Err("Failed to add director".into())
        }
    }
}

#[command]
pub fn remove_director_from_film(film_id: i32, person_id: i32) -> Result<(), String> {
    match actions::remove_director_from_film(film_id, person_id) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Director was not found in database".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!(
                "Error when deleting director {error}, film_id: {film_id}, person_id: {person_id}"
            );
            Err("Failed to delete director".into())
        }
    }
}
