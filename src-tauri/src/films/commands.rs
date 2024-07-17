use super::Film;

#[tauri::command]
pub fn get_all_films() -> Option<Vec<Film>> {
    match super::actions::get_all_films() {
        Ok(films) => Some(films),
        Err(_) => None,
    }
}
