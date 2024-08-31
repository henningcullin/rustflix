use tauri::command;

use crate::scrapers::actions::insert_scraped_film;

#[command]
pub async fn scrape_film(imdb_id: String, database_id: u32) -> bool {
    let scraped_film = match super::actions::scrape_film(imdb_id, database_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e:?}");
            return false;
        }
    };

    let scraped_json = serde_json::to_string(&scraped_film).unwrap();

    println!("{scraped_json}");

    match insert_scraped_film(scraped_film) {
        Ok(()) => true,
        Err(error) => {
            eprintln!("{error}");
            false
        }
    }
}
