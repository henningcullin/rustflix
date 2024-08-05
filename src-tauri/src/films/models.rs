use chrono::{Duration, NaiveDate};
use rusqlite::Row;
use serde::Serialize;
use src_macro::Fields;

use crate::{characters::Character, persons::Person};

#[derive(Debug, Serialize, Fields)]
pub struct Film {
    pub id: u32,
    pub file: String,
    pub directory: u32,
    pub imdb_id: Option<String>,
    pub title: Option<String>,
    pub genres: Vec<String>,
    pub release_date: Option<NaiveDate>,
    pub plot: Option<String>,
    pub run_time: Option<u32>, // seconds
    pub color: Option<String>,
    pub rating: Option<f64>,
    pub languages: Vec<String>,
    pub keywords: Vec<String>,
    pub cover_image: Option<String>,
    pub directors: Vec<Person>,
    pub stars: Vec<Character>,
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
