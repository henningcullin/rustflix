use html_escape::decode_html_entities;
use reqwest::{header::HeaderMap, Client};
use rusqlite::params;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::num::ParseIntError;

use crate::{database::create_connection, error::AppError};

use super::{ScrapedDirector, ScrapedFilm, ScrapedStar};

pub async fn scrape_film(imdb_id: String, database_id: u32) -> Result<ScrapedFilm, AppError> {
    let url = format!("https://www.imdb.com/title/{}/", imdb_id);

    // TODO: let user chose region
    // option 1 (easy): Let user choose between local or US
    // option 2 (complicated): Let user choose any region based on imdb languages

    let mut cookies = HashMap::new();
    cookies.insert("lc-main".to_string(), "en_US".to_string());

    let raw_html = get_page(&url, None, Some(cookies)).await?;
    let parsed_html = Html::parse_document(&raw_html);

    let data_object = get_data_object(&parsed_html)?;

    let title = unescape_str(data_object["name"].as_str());
    let genres = genre(data_object["genre"].as_array());
    let release_date = unescape_str(data_object["datePublished"].as_str());
    let plot = unescape_str(data_object["description"].as_str());
    let run_time = unescape_str(data_object["duration"].as_str());
    let color = color(&parsed_html).ok();
    let directors = directors(&data_object);
    let stars = stars(&parsed_html)?;
    let cover_image = unescape_str(data_object["image"].as_str());
    let rating = data_object["aggregateRating"]["ratingValue"].as_f64();
    let languages = languages(&parsed_html);
    let keywords = keywords(data_object["keywords"].as_str());

    let info = ScrapedFilm {
        id: database_id,
        title,
        genres,
        imdb_id,
        release_date,
        plot,
        run_time,
        color,
        directors,
        stars,
        cover_image,
        rating,
        languages,
        keywords,
    };

    Ok(info)
}

async fn get_page(
    url: &str,
    headers: Option<HeaderMap>,
    cookies: Option<HashMap<String, String>>,
) -> Result<String, AppError> {
    let client = Client::new();
    let mut request = client.get(url);

    // Add headers if provided
    if let Some(headers) = headers {
        request = request.headers(headers);
    }

    // Add cookies if provided
    if let Some(cookies) = cookies {
        let cookie_header = cookies
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");

        request = request.header("Cookie", cookie_header);
    }

    let response = request.send().await?;
    let html = response.text().await?;
    Ok(html)
}

fn get_data_object(parsed_html: &Html) -> Result<serde_json::Value, AppError> {
    let selector = Selector::parse(r#"[type="application/ld+json"]"#)?;
    let script_element = parsed_html
        .select(&selector)
        .next()
        .ok_or_else(|| AppError::new("No script element"))?;
    let script_text = script_element.inner_html();
    let data_object: serde_json::Value = serde_json::from_str(&script_text)?;
    Ok(data_object)
}

fn unescape_str(value: Option<&str>) -> Option<String> {
    let string = value?;

    match string.len() {
        0 => None,
        _ => Some(
            decode_html_entities(decode_html_entities(string).to_string().as_str()).to_string(),
        ),
    }
}

fn unescape_string(value: String) -> Option<String> {
    match value.len() {
        0 => None,
        _ => Some(
            decode_html_entities(decode_html_entities(&value).to_string().as_str()).to_string(),
        ),
    }
}

fn genre(genre_field: Option<&Vec<serde_json::Value>>) -> Vec<String> {
    genre_field.map_or(vec![], |array| {
        array
            .iter()
            .filter_map(|g| unescape_str(g.as_str()))
            .collect()
    })
}

fn color(parsed_html: &Html) -> Result<String, AppError> {
    let selector = Selector::parse(r#"[data-testid="title-techspec_color"] span"#)?;
    let color_element = parsed_html.select(&selector).next();

    match color_element {
        Some(element) => {
            let color_text = element.inner_html();
            match unescape_string(color_text) {
                Some(color_type) => Ok(color_type),
                None => Err(AppError::new("Color element empty innertext")),
            }
        }
        None => Err(AppError::new("No color element")),
    }
}

fn languages(parsed_html: &Html) -> Vec<String> {
    let selector = match Selector::parse(
        r#"[data-testid="title-details-languages"] a[href*="/search/title?title_type=feature&primary_language="]"#,
    ) {
        Ok(sel) => sel,
        Err(error) => {
            eprintln!("language selector error {error:?}");
            return Vec::new();
        }
    };

    parsed_html
        .select(&selector)
        .filter_map(|link_element| unescape_string(link_element.inner_html()))
        .collect()
}

fn keywords(keyword_string: Option<&str>) -> Vec<String> {
    match keyword_string {
        Some(string) => string.split(",").map(|str| str.to_string()).collect(),
        None => return Vec::new(),
    }
}
fn directors(data_object: &serde_json::Value) -> Vec<ScrapedDirector> {
    // Attempt to get the array of directors
    let director_array = match data_object["director"].as_array() {
        Some(array) => array,
        None => {
            eprintln!("No director array found");
            return vec![];
        }
    };

    // Process the array elements if it exists
    director_array
        .iter()
        .filter_map(|d| {
            let imdb_id = d["url"]
                .as_str()
                .and_then(|url| url.split('/').nth(4))
                .map(|id| id.to_string());

            let real_name = unescape_str(d["name"].as_str());

            if imdb_id.is_none() {
                eprintln!("Director missing imdb_id for element: {:?}", d);
            }

            if real_name.is_none() {
                eprintln!("Director missing real_name for element: {:?}", d);
            }

            if let (Some(imdb_id), Some(real_name)) = (imdb_id, real_name) {
                Some(ScrapedDirector { imdb_id, real_name })
            } else {
                None
            }
        })
        .collect()
}

fn stars(parsed_html: &Html) -> Result<Vec<ScrapedStar>, AppError> {
    let selector = Selector::parse(r#"[data-testid="title-cast-item"]"#)?;
    let star_elements = parsed_html.select(&selector);

    let avatar_selector = Selector::parse(r#"[data-testid="title-cast-item__avatar"] img"#)?;
    let character_selector = Selector::parse(r#"[data-testid="cast-item-characters-link"] span"#)?;
    let actor_selector = Selector::parse(r#"[data-testid="title-cast-item__actor"]"#)?;

    let stars = star_elements
        .filter_map(|element| {
            let element_html = element.html(); // Capture the element's HTML for logging

            // Attempt to find avatar, can be None
            let avatar = element.select(&avatar_selector).next().and_then(|e| {
                e.value()
                    .attr("srcset")
                    .and_then(|srcset| srcset.split(", ").last())
                    .and_then(|src| src.split_whitespace().next())
                    .map(|avatar_source| avatar_source.to_string())
            });

            // Extract actor element, skip if missing
            let actor_element = match element.select(&actor_selector).next() {
                Some(actor) => actor,
                None => {
                    eprintln!("Missing actor element in:\n{}", element_html);
                    return None;
                }
            };

            // Extract real name, skip if unable to unescape
            let real_name = match unescape_string(actor_element.inner_html()) {
                Some(name) => name,
                None => {
                    eprintln!("Failed to unescape real name in:\n{}", actor_element.html());
                    return None;
                }
            };

            // Extract character, skip if missing or unescaping fails
            let character = match element
                .select(&character_selector)
                .next()
                .and_then(|el| unescape_string(el.inner_html()))
            {
                Some(char_name) => char_name,
                None => {
                    eprintln!(
                        "Missing or failed to unescape character in:\n{}",
                        element_html
                    );
                    return None;
                }
            };

            // Extract IMDb ID, skip if not found
            let imdb_id = match actor_element
                .value()
                .attr("href")
                .and_then(|href| href.split('/').nth(2))
                .map(|id| id.to_string())
            {
                Some(id) => id,
                None => {
                    eprintln!("Missing IMDb ID in:\n{}", actor_element.html());
                    return None;
                }
            };

            // Construct ScrapedStar object if all data is valid
            Some(ScrapedStar {
                imdb_id,
                real_name,
                character,
                avatar,
            })
        })
        .collect::<Vec<ScrapedStar>>();

    Ok(stars)
}

// A function to parse ISO 8601 duration
fn parse_iso_duration(duration_str: &str) -> Result<i64, ParseIntError> {
    // Extract the duration parts from the ISO 8601 format
    let duration = duration_str.trim_start_matches("PT");

    let mut hours = 0;
    let mut minutes = 0;

    if let Some(pos) = duration.find('H') {
        hours = duration[..pos].parse()?;
        let rest = &duration[pos + 1..];
        if let Some(pos) = rest.find('M') {
            minutes = rest[..pos].parse()?;
        }
    } else if let Some(pos) = duration.find('M') {
        minutes = duration[..pos].parse()?;
    }

    Ok((hours * 3600 + minutes * 60) as i64)
}

pub async fn insert_scraped_film(film: &ScrapedFilm) -> Result<Vec<(i64, String)>, AppError> {
    let mut conn = create_connection()?;

    // Start the transaction
    let tx = conn.transaction()?;

    let run_time: Option<i64> = film
        .run_time
        .as_ref()
        .and_then(|string| match parse_iso_duration(string) {
            Ok(duration) => Some(duration),
            Err(e) => {
                eprintln!("Failed to parse duration: {:?}", e);
                None
            }
        });

    let has_color = film.color.as_ref().map(|string| {
        if string == "Color" {
            true
        } else {
            eprintln!("Unrecognized color value: {}", string);
            false
        }
    });

    if let Err(e) = tx.execute(
        r#"--sql
        UPDATE 
            films
        SET
            imdb_id = $1,
            title = $2,
            release_date = $3,
            plot = $4,
            run_time = $5,
            has_color = $6,
            rating = $7,
            registered = 1
        WHERE
            id = $8
    "#,
        params![
            film.imdb_id,
            film.title,
            film.release_date,
            film.plot,
            run_time,
            has_color,
            film.rating,
            film.id
        ],
    ) {
        eprintln!("Failed to update film: {:?}", e);
        return Err(AppError::from(e));
    }

    let genre_ids: Vec<i64> = film
        .genres
        .iter()
        .filter_map(|genre| {
            if let Err(e) = tx.execute(
                r#"INSERT INTO genres (name) VALUES (?) ON CONFLICT(name) DO NOTHING"#,
                params![genre],
            ) {
                eprintln!("Failed to insert genre '{}': {:?}", genre, e);
                return None;
            }
            match tx.query_row(
                r#"SELECT id FROM genres WHERE name = ?"#,
                params![genre],
                |row| row.get(0),
            ) {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!("Failed to retrieve genre ID for '{}': {:?}", genre, e);
                    None
                }
            }
        })
        .collect();

    for genre_id in &genre_ids {
        if let Err(e) = tx.execute(
            r#"INSERT INTO film_genres (film_id, genre_id) VALUES (?, ?) ON CONFLICT(film_id, genre_id) DO NOTHING"#,
            params![film.id, genre_id],
        ) {
            eprintln!("Failed to link film to genre ID {}: {:?}", genre_id, e);
        }
    }

    let language_ids: Vec<i64> = film
        .languages
        .iter()
        .filter_map(|language| {
            if let Err(e) = tx.execute(
                r#"--sql
                INSERT INTO languages (name) VALUES (?) ON CONFLICT(name) DO NOTHING
                "#,
                params![language],
            ) {
                eprintln!("Failed to insert language '{}': {:?}", language, e);
                return None;
            }
            match tx.query_row(
                r#"--sql
                SELECT id FROM languages WHERE name = ?
                "#,
                params![language],
                |row| row.get(0),
            ) {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!("Failed to retrieve language ID for '{}': {:?}", language, e);
                    None
                }
            }
        })
        .collect();

    for language_id in &language_ids {
        if let Err(e) = tx.execute(
            r#"--sql
            INSERT INTO film_languages (film_id, language_id) VALUES (?, ?) ON CONFLICT(film_id, language_id) DO NOTHING
            "#,
            params![film.id, language_id],
        ) {
            eprintln!("Failed to link film to language ID {}: {:?}", language_id, e);
        }
    }

    let keyword_ids: Vec<i64> = film
        .keywords
        .iter()
        .filter_map(|keyword| {
            if let Err(e) = tx.execute(
                r#"--sql
                INSERT INTO keywords (name) VALUES (?) ON CONFLICT(name) DO NOTHING
                "#,
                params![keyword],
            ) {
                eprintln!("Failed to insert keyword '{}': {:?}", keyword, e);
                return None;
            }
            match tx.query_row(
                r#"--sql
                SELECT id FROM keywords WHERE name = ?
                "#,
                params![keyword],
                |row| row.get(0),
            ) {
                Ok(id) => Some(id),
                Err(e) => {
                    eprintln!("Failed to retrieve keyword ID for '{}': {:?}", keyword, e);
                    None
                }
            }
        })
        .collect();

    for keyword_id in &keyword_ids {
        if let Err(e) = tx.execute(
            r#"--sql
            INSERT INTO film_keywords (film_id, keyword_id) VALUES (?, ?) ON CONFLICT(film_id, keyword_id) DO NOTHING
            "#,
            params![film.id, keyword_id],
        ) {
            eprintln!("Failed to link keyword to Film ID {}, Error: {:?}", keyword_id, e);
        }
    }

    let mut persons_with_avatars = Vec::new();

    // Insert directors
    for director in &film.directors {
        if let Err(e) = tx.execute(
            r#"INSERT INTO persons (imdb_id, name) VALUES (?, ?) 
               ON CONFLICT(imdb_id) DO NOTHING"#,
            params![director.imdb_id, director.real_name],
        ) {
            eprintln!(
                "Failed to insert director '{}': {:?}",
                director.real_name, e
            );
            continue;
        }

        let person_id: i64 = match tx.query_row(
            r#"SELECT id FROM persons WHERE imdb_id = ?"#,
            params![director.imdb_id],
            |row| row.get(0),
        ) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "Failed to retrieve person ID for director '{}': {:?}",
                    director.real_name, e
                );
                continue;
            }
        };

        if let Err(e) = tx.execute(
            r#"INSERT INTO film_directors (film_id, person_id) VALUES (?, ?) 
               ON CONFLICT(film_id, person_id) DO NOTHING"#,
            params![film.id, person_id],
        ) {
            eprintln!(
                "Failed to link director ID {} to film ID {}: {:?}",
                person_id, film.id, e
            );
        }
    }

    // Insert stars
    for star in &film.stars {
        if let Err(e) = tx.execute(
            r#"INSERT INTO persons (imdb_id, name) VALUES (?, ?) 
               ON CONFLICT(imdb_id) DO NOTHING"#,
            params![star.imdb_id, star.real_name],
        ) {
            eprintln!("Failed to insert star '{}': {:?}", star.real_name, e);
            continue;
        }

        let person_id: i64 = match tx.query_row(
            r#"SELECT id FROM persons WHERE imdb_id = ?"#,
            params![star.imdb_id],
            |row| row.get(0),
        ) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "Failed to retrieve person ID for star '{}': {:?}",
                    star.real_name, e
                );
                continue;
            }
        };

        if let Some(avatar_url) = &star.avatar {
            persons_with_avatars.push((person_id, avatar_url.clone()));
        }

        // Insert character directly into the characters table with film_id
        if let Err(e) = tx.execute(
            r#"INSERT INTO characters (film_id, description, actor) VALUES (?, ?, ?) 
               ON CONFLICT(film_id, actor) DO NOTHING"#,
            params![film.id, star.character, person_id],
        ) {
            eprintln!(
                "Failed to insert character for star '{}': {:?}",
                star.real_name, e
            );
            continue;
        }
    }

    // Handle persons and characters synchronously

    // Commit the transaction
    if let Err(e) = tx.commit() {
        eprintln!("Failed to commit transaction: {:?}", e);
        return Err(AppError::from(e));
    }

    Ok(persons_with_avatars)
}
