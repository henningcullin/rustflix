use tauri::command;

use super::actions;

#[command]
pub fn delete_director(film_id: i32, person_id: i32) -> Result<(), String> {
    match actions::delete_director(film_id, person_id) {
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

#[command]
pub fn create_director(film_id: i32, person_id: i32) -> Result<(), String> {
    match actions::create_director(film_id, person_id) {
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
