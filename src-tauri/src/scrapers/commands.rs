use tauri::command;

#[command]
pub async fn scrape_film(id: String) -> bool {
    let scraped_film = match super::actions::scrape_film(id).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e:?}");
            return false;
        }
    };

    println!("{scraped_film:?}");

    true
}
