use tauri::command;

use crate::{
    database::{get_avatar_path, get_cover_path},
    images::store_image,
    scrapers::actions::insert_scraped_film,
};

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

    let persons_with_avatar = match insert_scraped_film(&scraped_film).await {
        Ok(avatars) => avatars,
        Err(error) => {
            eprintln!("{error}");
            return false;
        }
    };

    // Handle image storage asynchronously after the transaction has been committed
    for (person_id, avatar_url) in persons_with_avatar {
        if let Err(e) = store_image(&avatar_url, &person_id.to_string(), get_avatar_path()).await {
            eprintln!("Failed to store image for person_id {}: {:?}", person_id, e);
            return false;
        }
    }

    if let Some(cover_image) = &scraped_film.cover_image {
        if let Err(e) =
            store_image(cover_image, &scraped_film.id.to_string(), get_cover_path()).await
        {
            eprintln!(
                "Failed to store cover image for film_id {}: {:?}",
                scraped_film.id, e
            );
            return false;
        }
    }

    true
}
