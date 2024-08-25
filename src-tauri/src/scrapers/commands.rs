use tauri::command;

use crate::{database::get_cover_path, images::store_image};

#[command]
pub async fn scrape_film(imdb_id: String, database_id: u32) -> bool {
    let scraped_film = match super::actions::scrape_film(imdb_id, database_id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e:?}");
            return false;
        }
    };

    if let Some(cover_image) = &scraped_film.cover_image {
        let _ = store_image(cover_image, &scraped_film.id.to_string(), get_cover_path());
    }

    let scraped_json = serde_json::to_string(&scraped_film).unwrap();

    println!("{scraped_json}");

    true
}
