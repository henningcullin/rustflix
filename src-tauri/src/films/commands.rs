use crate::delete_command;

use super::{actions, Film};

use tauri::command;

#[command]
pub fn get_all_films() -> Result<Vec<Film>, String> {
    match actions::get_all_films() {
        Ok(films) => Ok(films),
        Err(error) => {
            eprintln!("{}", error);
            Err(error.to_string()) // Return the error as a String
        }
    }
}

#[command]
pub fn get_film(id: u32) -> Result<Film, String> {
    match actions::get_film(id) {
        Ok(film) => Ok(film),
        Err(error) => {
            eprintln!("{}", error);
            Err(error.to_string()) // Return the error as a String
        }
    }
}

#[command]
pub fn delete_film(id: i32) -> Result<(), String> {
    delete_command!(
        actions::delete_film(id),
        "Film deleted successfully with id: {id}",
        "Error when deleting film: {error}, id: {id}",
        "Film was not deleted: id: {id}",
        "Failed to remove film"
    )
}

// #[command]
// pub fn sync_new_films() -> Result<(), String> {
//     match super::actions::sync_new_films() {
//         Ok(_) => Ok(()),
//         Err(error) => {
//             eprintln!("{}", error);
//             Err(error.to_string()) // Return the error as a String
//         }
//     }
// }

#[command]
pub fn set_left_off_point(film_id: i32, played: i32) -> Result<(), String> {
    match actions::set_left_off_point(film_id, played) {
        Ok(rows_affected) => match rows_affected {
            0 => Err("Left off point could was not stored".into()),
            _ => Ok(()),
        },
        Err(error) => {
            eprintln!(
                "Error setting left_off_point for film error: {error}, film_id: {film_id}, played: {played}"
            );
            Err("Failed to set Left off point".into())
        }
    }
}
