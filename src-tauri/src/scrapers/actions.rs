use scraper::{Html, Selector};
use serde::Deserialize;

use crate::error::AppError;
#[derive(Debug, Deserialize)]
struct Film {
    title: Option<String>,
    genres: Vec<String>,
    release_date: Option<String>,
    plot: Option<String>,
    run_time: Option<String>,
    color: Option<String>,
    directors: Vec<Director>,
    stars: Vec<Star>,
    cover: Option<String>,
    rating: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct Director {
    imdb_id: String,
    real_name: String,
}

#[derive(Debug, Deserialize)]
struct Star {
    imdb_id: String,
    real_name: String,
    character: String,
    avatar: Option<String>,
}

pub async fn scrape_movie(id: &str) -> Result<Film, AppError> {
    let url = format!("https://www.imdb.com/title/{}/", id);
    let raw_html = get_page(&url).await?;
    let parsed_html = Html::parse_document(&raw_html);

    let data_object: serde_json::Value = get_data_object(&parsed_html)?;

    let info = Film {
        title: Some(unescape(data_object["name"].as_str().unwrap_or_default())),
        genres: data_object["genre"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|g| unescape(g.as_str().unwrap_or_default()))
            .collect(),
        release_date: Some(data_object["datePublished"].to_string()),
        plot: Some(unescape(
            data_object["description"].as_str().unwrap_or_default(),
        )),
        run_time: Some(
            data_object["duration"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        ),
        color: color(&parsed_html).ok(),
        directors: directors(&data_object),
        stars: stars(&parsed_html),
        cover: Some(data_object["image"].to_string()),
        rating: data_object["aggregateRating"]["ratingValue"]
            .as_f64()
            .unwrap_or_default()
            .into(),
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
        .ok_or(AppError::ScrapeError("No script element found".to_string()))?;
    let script_text = script_element.inner_html();
    let data_object: serde_json::Value = serde_json::from_str(&script_text)?;
    Ok(data_object)
}

fn unescape(string: &str) -> String {
    html_escape::decode_html_entities(string).to_string()
}

fn color(parsed_html: &Html) -> Result<String, AppError> {
    let selector = Selector::parse(r#"[data-testid="title-techspec_color"]"#)
        .map_err(|e| AppError::ScrapeError(e.to_string()))?;
    let color_element = parsed_html.select(&selector).next();

    match color_element {
        Some(element) => {
            if let Some(last_child) = element.last_child() {
                if let Some(text) = last_child.value().as_text() {
                    return Ok(text.to_string());
                }
            }
            Err(AppError::ScrapeError("Color text not found".to_string()))
        }
        None => Err(AppError::ScrapeError(
            "Color information not found".to_string(),
        )),
    }
}

fn directors(data_object: &serde_json::Value) -> Vec<Director> {
    data_object["director"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|d| Director {
            imdb_id: d["url"]
                .as_str()
                .unwrap_or_default()
                .split("/")
                .nth(4)
                .unwrap_or_default()
                .to_string(),
            real_name: unescape(d["name"].as_str().unwrap_or_default()),
        })
        .collect()
}

fn stars(parsed_html: &Html) -> Vec<Star> {
    let selector = Selector::parse(r#"[data-testid="title-cast-item"]"#).unwrap();
    let star_elements = parsed_html.select(&selector);

    star_elements
        .map(|element| {
            let avatar_selector =
                Selector::parse(r#"[data-testid="title-cast-item__avatar"] img"#).unwrap();
            let character_selector =
                Selector::parse(r#"[data-testid="cast-item-characters-link"]"#).unwrap();
            let actor_selector =
                Selector::parse(r#"[data-testid="title-cast-item__actor"]"#).unwrap();

            let avatar = element.select(&avatar_selector).next().and_then(|e| {
                e.value()
                    .attr("srcset")
                    .and_then(|srcset| srcset.split(", ").last())
                    .map(|s| s.split(" ").next().unwrap_or_default().to_string())
            });

            let character = element
                .select(&character_selector)
                .next()
                .and_then(|e| e.first_child().and_then(|c| c.value().as_text()))
                .map(|item| unescape(item))
                .unwrap_or_default();

            let actor_element = element.select(&actor_selector).next().unwrap();
            let real_name = unescape(actor_element.inner_html().as_str());
            let imdb_id = actor_element
                .value()
                .attr("href")
                .unwrap_or_default()
                .split("/")
                .nth(4)
                .unwrap_or_default()
                .to_string();

            Star {
                imdb_id,
                real_name,
                character,
                avatar,
            }
        })
        .collect()
}
