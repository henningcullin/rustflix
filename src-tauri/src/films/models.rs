use chrono::NaiveDate;
use rusqlite::Row;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Film {
    pub id: u32,
    pub file: String,
    pub directory: u32,
    pub link: Option<String>,
    pub title: Option<String>,
    pub synopsis: Option<String>,
    pub release_date: Option<NaiveDate>,
    pub duration: Option<u32>,
    pub cover_image: Option<String>,
    pub registered: bool,
}

impl Film {
    pub fn from_row(row: &Row) -> Result<Film, rusqlite::Error> {
        Ok(Film {
            id: row.get(0)?,
            file: row.get(1)?,
            directory: row.get(2)?,
            link: row.get(3)?,
            title: row.get(4)?,
            synopsis: row.get(5)?,
            release_date: row.get(6)?,
            duration: row.get(7)?,
            cover_image: row.get(8)?,
            registered: row.get(9)?,
        })
    }
}
