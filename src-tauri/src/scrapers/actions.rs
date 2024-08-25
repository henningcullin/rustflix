use html_escape::decode_html_entities;
use scraper::{Html, Selector};

use crate::error::AppError;

use super::{ScrapedDirector, ScrapedFilm, ScrapedStar};

pub async fn scrape_film(imdb_id: String, database_id: u32) -> Result<ScrapedFilm, AppError> {
    let url = format!("https://www.imdb.com/title/{}/", imdb_id);
    let raw_html = get_page(&url).await?;
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

async fn get_page(url: &str) -> Result<String, AppError> {
    let response = reqwest::get(url).await?;
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
