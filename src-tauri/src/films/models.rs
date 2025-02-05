use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    characters::Character, genres::Genre, keywords::Keyword, languages::Language, persons::Person,
    FromRow,
};

#[derive(Debug, Serialize)]
pub struct Film {
    pub id: u32,
    pub file: Option<String>,
    pub imdb_id: Option<String>,
    pub title: Option<String>,
    pub genres: Vec<Genre>,
    pub release_date: Option<NaiveDate>,
    pub plot: Option<String>,
    pub run_time: Option<u32>, // seconds
    pub has_color: Option<bool>,
    pub rating: Option<f64>,
    pub languages: Vec<Language>,
    pub keywords: Vec<Keyword>,
    pub directors: Vec<Person>,
    pub stars: Vec<Character>,
    pub has_watched: bool,
    pub left_off_point: Option<u32>,
}

impl FromRow for Film {
    fn from_row(row: &rusqlite::Row) -> Result<Film, rusqlite::Error> {
        let id: u32 = row.get("film_id")?;
        let file: Option<String> = row.get("file")?;

        let imdb_id: Option<String> = row.get("imdb_id")?;
        let title: Option<String> = row.get("title")?;
        let plot: Option<String> = row.get("plot")?;
        let run_time: Option<u32> = row.get("run_time")?;
        let has_color: Option<bool> = row.get("has_color")?;
        let rating: Option<f64> = row.get("rating")?;
        let has_watched: bool = row.get("has_watched")?;
        let left_off_point: Option<u32> = row.get("left_off_point")?;

        let release_date: Option<NaiveDate> = row
            .get::<_, Option<String>>("release_date")?
            .and_then(|d| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok());

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
        let keywords: Vec<Keyword> = row
            .get::<_, Option<String>>("keywords")?
            .unwrap_or_default()
            .split(',')
            .filter_map(|keyword| {
                let mut parts = keyword.split(':');
                Some(Keyword {
                    id: parts.next()?.parse().ok()?,
                    name: parts.next()?.to_string(),
                })
            })
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
            imdb_id,
            title,
            genres,
            release_date,
            plot,
            run_time,
            has_color,
            rating,
            languages,
            keywords,
            directors,
            stars,
            has_watched,
            left_off_point,
        })
    }
}
