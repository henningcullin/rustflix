use std::path::PathBuf;

use rusqlite::params;
use tauri::api::dialog::blocking::FileDialogBuilder;

use crate::database::create_connection;

use super::Directory;

pub fn add_directory(path: &str) -> Result<Directory, rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute("INSERT INTO directories (path) VALUES (?1)", params![path])?;

    // Retrieve the last inserted row ID
    let id = conn.last_insert_rowid() as i32;

    // Fetch the newly created directory
    let mut stmt = conn.prepare("SELECT id, path FROM directories WHERE id = ?1")?;
    let directory = stmt.query_row(params![id], |row| {
        Ok(Directory {
            id: row.get(0)?,
            path: row.get(1)?,
        })
    })?;

    Ok(directory)
}

pub fn delete_directory(id: u32) -> Result<(), rusqlite::Error> {
    let conn = create_connection()?;

    conn.execute("DELETE FROM directories WHERE id = ?1", params![id])?;

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
