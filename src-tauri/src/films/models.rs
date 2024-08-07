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
    pub has_watched: bool,
    pub left_off_point: Option<u32>,
    pub registered: bool,
}

impl Film {
    pub fn from_row(row: &Row) -> Result<Film, rusqlite::Error> {
        Ok(Film {
            id: row.get(0)?,
            file: row.get(1)?,
            directory: row.get(2)?,
            imdb_id: row.get(3)?,
            title: row.get(4)?,
            genres: row.get(5)?,
            release_date: row.get(6)?,
            plot: row.get(7)?,
            run_time: row.get(8)?,
            color: row.get(9)?,
            rating: row.get(10)?,
            languages: row.get(11)?,
            keywords: row.get(12)?,
            cover_image: row.get(13)?,
            directors: row.get(14)?,
            stars: row.get(15)?,
            has_watched: row.get(16)?,
            left_off_point: row.get(17)?,
            registered: row.get(18),
        })
    }
}
