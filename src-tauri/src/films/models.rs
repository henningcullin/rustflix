use std::collections::HashMap;

use rusqlite::Row;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Film {
    pub id: u32,
    pub file: String,
    pub link: Option<String>,
    pub title: Option<String>,
    pub release_year: Option<u32>,
    pub duration: Option<u32>,
    pub cover_image: Option<String>,
}

impl Film {
    pub fn from_row(row: &Row) -> Result<Film, rusqlite::Error> {
        Ok(Film {
            id: row.get(0)?,
            file: row.get(1)?,
            link: row.get(2)?,
            title: row.get(3)?,
            release_year: row.get(4)?,
            duration: row.get(5)?,
            cover_image: row.get(6)?,
        })
    }
}

#[derive(Serialize)]
pub struct VideoFiles {
    pub files_by_directory: HashMap<u32, Vec<String>>,
}
