use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Film {
    title: String,
    genres: Vec<String>,
    release_date: String,
    plot: String,
    run_time: String,
    color: String,
    directors: Vec<Director>,
    stars: Vec<Star>,
    cover: String,
    rating: f64,
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

async fn scrape_movie(id: &str) -> Result<Film, Box<dyn std::error::Error>> {
    let url = format!("https://www.imdb.com/title/{}/", id);
    let raw_html = get_page(&url).await?;
    let parsed_html = Html::parse_document(&raw_html);

    let data_object: serde_json::Value = get_data_object(&parsed_html)?;
    let info = Film {
        title: unescape(data_object["name"].as_str().unwrap_or_default()),
        genres: data_object["genre"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|g| unescape(g.as_str().unwrap_or_default()))
            .collect(),
        release_date: data_object["datePublished"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        plot: unescape(data_object["description"].as_str().unwrap_or_default()),
        run_time: data_object["duration"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        color: color(&parsed_html),
        directors: directors(&data_object),
        stars: stars(&parsed_html),
        cover: data_object["image"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        rating: data_object["aggregateRating"]["ratingValue"]
            .as_f64()
            .unwrap_or_default(),
    };

    Ok(info)
}

async fn get_page(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let html = response.text().await?;
    Ok(html)
}

fn get_data_object(parsed_html: &Html) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let selector = Selector::parse(r#"[type="application/ld+json"]"#).unwrap();
    let script_element = parsed_html
        .select(&selector)
        .next()
        .ok_or("No script element found")?;
    let script_text = script_element.inner_html();
    let data_object: serde_json::Value = serde_json::from_str(&script_text)?;
    Ok(data_object)
}

fn unescape(string: &str) -> String {
    html_escape::decode_html_entities(string).to_string()
}

fn color(parsed_html: &Html) -> String {
    let selector = Selector::parse(r#"[data-testid="title-techspec_color"]"#).unwrap();
    let color_element = parsed_html.select(&selector).next();
    match color_element {
        Some(element) => element
            .last_child()
            .unwrap()
            .value()
            .as_text()
            .unwrap()
            .to_string(),
        None => String::new(),
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
                .map(|e| unescape(e.first_child().unwrap().value().as_text().unwrap()))
                .unwrap_or_default();

            let actor_element = element.select(&actor_selector).next().unwrap();
            let real_name = unescape(actor_element.inner_html().as_str());
            let imdb_id = actor_element
                .value()
                .attr("href")
                .unwrap()
                .split("/")
                .nth(4)
                .unwrap()
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
