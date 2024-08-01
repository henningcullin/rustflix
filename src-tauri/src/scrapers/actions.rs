use scraper::{Html, Selector};

use crate::error::AppError;

use super::{ScrapedDirector, ScrapedFilm, ScrapedStar};

pub async fn scrape_movie(id: &str) -> Result<ScrapedFilm, AppError> {
    let url = format!("https://www.imdb.com/title/{}/", id);
    let raw_html = get_page(&url).await?;
    let parsed_html = Html::parse_document(&raw_html);

    let data_object = get_data_object(&parsed_html)?;

    let title = unescape_str(data_object["name"].as_str());
    let genres = data_object["genre"].as_array().map_or(vec![], |array| {
        array
            .iter()
            .filter_map(|g| unescape_str(g.as_str()))
            .collect()
    });
    let release_date = unescape_str(data_object["datePublished"].as_str());
    let plot = unescape_str(data_object["description"].as_str());
    let run_time = unescape_str(data_object["duration"].as_str());
    let color = color(&parsed_html).ok();
    let directors = directors(&data_object)?;
    let stars = stars(&parsed_html)?;
    let cover = unescape_str(data_object["image"].as_str());
    let rating = data_object["aggregateRating"]["ratingValue"].as_f64();

    let info = ScrapedFilm {
        title,
        genres,
        release_date,
        plot,
        run_time,
        color,
        directors,
        stars,
        cover,
        rating,
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

fn unescape_str(value: Option<&str>) -> Option<String> {
    match value {
        Some(string) => Some(html_escape::decode_html_entities(string).to_string()),
        None => None,
    }
}

fn unescape_string(value: String) -> Option<String> {
    match value.len() {
        0 => None,
        _ => Some(html_escape::decode_html_entities(&value).to_string()),
    }
}

fn color(parsed_html: &Html) -> Result<String, AppError> {
    let selector = Selector::parse(r#"[data-testid="title-techspec_color"]"#)?;
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

fn directors(data_object: &serde_json::Value) -> Result<Vec<ScrapedDirector>, AppError> {
    data_object["director"]
        .as_array()
        .ok_or_else(|| AppError::ScrapeError("No director array found".to_string()))?
        .iter()
        .map(|d| {
            let imdb_id = d["url"]
                .as_str()
                .and_then(|url| url.split('/').nth(4))
                .map(|id| id.to_string())
                .ok_or_else(|| AppError::ScrapeError("No IMDb ID found".to_string()))?;

            let real_name = unescape_str(d["name"].as_str())
                .ok_or_else(|| AppError::ScrapeError("No name found".to_string()))?;

            Ok(ScrapedDirector { imdb_id, real_name })
        })
        .collect()
}

fn stars(parsed_html: &Html) -> Result<Vec<ScrapedStar>, AppError> {
    let selector = Selector::parse(r#"[data-testid="title-cast-item"]"#)?;
    let star_elements = parsed_html.select(&selector);

    let stars = star_elements
        .map(|element| {
            let avatar_selector =
                Selector::parse(r#"[data-testid="title-cast-item__avatar"] img"#)?;
            let character_selector =
                Selector::parse(r#"[data-testid="cast-item-characters-link"]"#)?;
            let actor_selector = Selector::parse(r#"[data-testid="title-cast-item__actor"]"#)?;

            let avatar = element.select(&avatar_selector).next().and_then(|e| {
                e.value()
                    .attr("srcset")
                    .and_then(|srcset| srcset.split(", ").last())
                    .and_then(|src| src.split_whitespace().next())
                    .map(|avatar_source| avatar_source.to_string())
            });

            let character = unescape_string(
                element
                    .select(&character_selector)
                    .next()
                    .ok_or_else(|| AppError::ScrapeError("Character element not found".into()))?
                    .first_child()
                    .and_then(|n| n.value().as_text())
                    .map(|text| text.to_string())
                    .ok_or_else(|| AppError::ScrapeError("No inner text".into()))?,
            )
            .ok_or_else(|| AppError::ScrapeError("Empty string escaped".into()))?;

            let actor_element = element
                .select(&actor_selector)
                .next()
                .ok_or_else(|| AppError::ScrapeError("Failed to find actor element".into()))?;

            let real_name = unescape_string(actor_element.inner_html())
                .ok_or_else(|| AppError::ScrapeError("Real name not found".into()))?;

            let imdb_id = actor_element
                .value()
                .attr("href")
                .and_then(|href| href.split('/').nth(4))
                .map(|id| id.to_string())
                .ok_or_else(|| AppError::ScrapeError("Missing IMDb ID".into()))?;

            Ok(ScrapedStar {
                imdb_id,
                real_name,
                character,
                avatar,
            })
        })
        .collect::<Result<Vec<ScrapedStar>, AppError>>()?;

    Ok(stars)
}
