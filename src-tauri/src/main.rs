// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod database;

use database::Film;

#[tauri::command]
fn get_all_films() -> Option<Vec<Film>> {
    match database::get_all_films() {
        Ok(films) => Some(films),
        Err(_) => None,
    }
}

#[tauri::command]
fn add_directory(path: &str) -> bool {
    match database::add_directory(path) {
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
