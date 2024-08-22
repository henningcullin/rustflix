use chrono::NaiveDate;
use rusqlite::Row;
use serde::Serialize;
use src_macro::Fields;

use crate::{characters::Character, genres::Genre, languages::Language, persons::Person};

#[derive(Debug, Serialize, Fields)]
pub struct Film {
    pub id: u32,
    pub file: String,
    pub directory: u32,
    pub imdb_id: Option<String>,
    pub title: Option<String>,
    pub genres: Vec<Genre>,
    pub release_date: Option<NaiveDate>,
    pub plot: Option<String>,
    pub run_time: Option<u32>, // seconds
    pub color: Option<bool>,
    pub rating: Option<f64>,
    pub languages: Vec<Language>,
    pub keywords: Vec<String>,
    pub cover_image: Option<String>,
    pub directors: Vec<Person>,
    pub stars: Vec<Character>,
    pub has_watched: bool,
    pub left_off_point: Option<u32>,
    pub registered: bool,
}

impl Film {
    pub fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Film> {
        let id: u32 = row.get("film_id")?;
        let file: String = row.get("file")?;
        let directory: u32 = row.get("directory")?;
        let imdb_id: Option<String> = row.get("imdb_id")?;
        let title: Option<String> = row.get("title")?;
        let release_date: Option<String> = row.get("release_date")?;
        let plot: Option<String> = row.get("plot")?;
        let run_time: Option<u32> = row.get("run_time")?;
        let color: Option<bool> = row.get("color")?;
        let rating: Option<f64> = row.get("rating")?;
        let cover_image: Option<String> = row.get("cover_image")?;
        let has_watched: bool = row.get("has_watched")?;
        let left_off_point: Option<u32> = row.get("left_off_point")?;
        let registered: bool = row.get("registered")?;

        // Parse genres
        let genres: Vec<Genre> = row
            .get::<_, Option<String>>("genres")?
            .unwrap_or_default()
            .split(',')
            .filter_map(|genre| {
                let mut parts = genre.split(':');
                Some(Genre {
                    id: parts.next()?.parse().ok()?,
                    name: parts.next()?.to_string(),
                })
            })
            .collect();

        // Parse languages
        let languages: Vec<Language> = row
            .get::<_, Option<String>>("languages")?
            .unwrap_or_default()
            .split(',')
            .filter_map(|language| {
                let mut parts = language.split(':');
                Some(Language {
                    id: parts.next()?.parse().ok()?,
                    name: parts.next()?.to_string(),
                })
            })
            .collect();

        // Parse keywords
        let keywords: Vec<String> = row
            .get::<_, Option<String>>("keywords")?
            .unwrap_or_default()
            .split(',')
            .map(|s| s.to_string())
            .collect();

        // Parse directors
        let directors: Vec<Person> = row
            .get::<_, Option<String>>("directors")?
            .unwrap_or_default()
            .split(',')
            .filter_map(|director| {
                let mut parts = director.split(':');
                Person::from_parts(&mut parts)
            })
            .collect();

        // Parse stars (characters)
        let stars: Vec<Character> = row
            .get::<_, Option<String>>("stars")?
            .unwrap_or_default()
            .split(',')
            .filter_map(|star| {
                let mut parts = star.split(':');
                Character::from_parts(&mut parts)
            })
            .collect();

        Ok(Film {
            id,
            file,
            directory,
            imdb_id,
            title,
            genres,
            release_date: release_date.map(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").unwrap()),
            plot,
            run_time,
            color,
            rating,
            languages,
            keywords,
            cover_image,
            directors,
            stars,
            has_watched,
            left_off_point,
            registered,
        })
    }
}
