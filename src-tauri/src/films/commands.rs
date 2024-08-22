use super::Film;

use tauri::command;

#[command]
pub fn get_all_films() -> Option<Vec<Film>> {
    match super::actions::get_all_films() {
        Ok(films) => Some(films),
        Err(error) => {
            eprintln!("{}", error);
            None
        }
    }
}

#[command]
pub fn get_film(id: u32) -> Option<Film> {
    match super::actions::get_film(id) {
        Ok(film) => Some(film),
        Err(error) => {
            eprintln!("{}", error);
            None
        }
    }
}
