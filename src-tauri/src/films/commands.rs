use super::Film;

use tauri::command;

#[command]
pub fn get_all_films() -> Result<Vec<Film>, String> {
    match super::actions::get_all_films() {
        Ok(films) => Ok(films),
        Err(error) => {
            eprintln!("{}", error);
            Err(error.to_string()) // Return the error as a String
        }
    }
}

#[command]
pub fn get_film(id: u32) -> Result<Film, String> {
    match super::actions::get_film(id) {
        Ok(film) => Ok(film),
        Err(error) => {
            eprintln!("{}", error);
            Err(error.to_string()) // Return the error as a String
        }
    }
}

#[command]
pub fn sync_new_films() -> Result<(), String> {
    match super::actions::sync_new_films() {
        Ok(_) => Ok(()),
        Err(error) => {
            eprintln!("{}", error);
            Err(error.to_string()) // Return the error as a String
        }
    }
}
