// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;
mod directories;
mod films;

use films::Film;

#[tauri::command]
fn get_all_films() -> Option<Vec<Film>> {
    match films::get_all_films() {
        Ok(films) => Some(films),
        Err(_) => None,
    }
}

#[tauri::command]
fn add_directory(path: &str) -> bool {
    match directories::add_directory(path) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_all_films, add_directory])
        .setup(|_app| {
            match database::initialize_database() {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{:?}", err);
                }
            };
            match database::check_for_new_films() {
                Ok(_) => {}
                Err(_) => {}
            };
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
