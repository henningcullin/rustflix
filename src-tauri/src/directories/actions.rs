use std::path::PathBuf;

use rusqlite::params;
use tauri::api::dialog::blocking::FileDialogBuilder;

use crate::database::create_connection;

use super::Directory;

pub fn add_directory(path: &str) -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute("INSERT INTO directories (path) VALUES (?1)", params![path])?;

    Ok(())
}

pub fn get_all_directories() -> Result<Vec<Directory>, rusqlite::Error> {
    let conn = create_connection()?;

    let mut stmt = conn.prepare("SELECT id, path FROM directories")?;
    let directories: Vec<Directory> = stmt
        .query_map([], |row| {
            Ok(Directory {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(directories)
}

pub async fn select_directory() -> Option<PathBuf> {
    FileDialogBuilder::new().pick_folder()
}
