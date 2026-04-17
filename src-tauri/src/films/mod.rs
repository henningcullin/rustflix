use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Film {
    pub id: i64,
    pub file_path: String,
    pub tmdb_id: Option<i64>,
    pub imdb_id: Option<String>,
    pub title: String,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub runtime: Option<i64>,
    pub rating: Option<f64>,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
    pub left_off_point: i64,
    pub watched: i64,
    pub orphaned: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FilmListItem {
    pub id: i64,
    pub title: String,
    pub release_date: Option<String>,
    pub runtime: Option<i64>,
    pub rating: Option<f64>,
    pub poster_path: Option<String>,
    pub watched: i64,
    pub left_off_point: i64,
    pub orphaned: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FilmDetail {
    #[serde(flatten)]
    pub film: Film,
    pub genres: Vec<Genre>,
    pub cast: Vec<CastMember>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Genre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct CastMember {
    pub person_id: i64,
    pub name: String,
    pub profile_path: Option<String>,
    pub character: Option<String>,
    pub role: String,
    pub sort_order: i64,
}

#[tauri::command]
pub async fn list_films(state: tauri::State<'_, AppState>) -> AppResult<Vec<FilmListItem>> {
    let rows = sqlx::query_as::<_, FilmListItem>(
        "SELECT id, title, release_date, runtime, rating, poster_path, watched, left_off_point, orphaned \
         FROM films ORDER BY orphaned ASC, title COLLATE NOCASE ASC",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn list_continue_watching(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> AppResult<Vec<FilmListItem>> {
    let limit = limit.unwrap_or(10).clamp(1, 50);
    let rows = sqlx::query_as::<_, FilmListItem>(
        "SELECT id, title, release_date, runtime, rating, poster_path, watched, left_off_point, orphaned \
         FROM films \
         WHERE orphaned = 0 AND left_off_point > 15 AND watched = 0 \
         ORDER BY updated_at DESC \
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn list_recently_added(
    state: tauri::State<'_, AppState>,
    limit: Option<i64>,
) -> AppResult<Vec<FilmListItem>> {
    let limit = limit.unwrap_or(10).clamp(1, 50);
    let rows = sqlx::query_as::<_, FilmListItem>(
        "SELECT id, title, release_date, runtime, rating, poster_path, watched, left_off_point, orphaned \
         FROM films \
         WHERE orphaned = 0 \
         ORDER BY created_at DESC \
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await?;
    Ok(rows)
}

#[tauri::command]
pub async fn get_film(state: tauri::State<'_, AppState>, id: i64) -> AppResult<FilmDetail> {
    let film = sqlx::query_as::<_, Film>(
        "SELECT id, file_path, tmdb_id, imdb_id, title, original_title, overview, release_date, \
                runtime, rating, poster_path, backdrop_path, left_off_point, watched, orphaned, \
                created_at, updated_at \
         FROM films WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::NotFound)?;

    let genres = sqlx::query_as::<_, Genre>(
        "SELECT g.id, g.name FROM genres g \
         JOIN film_genres fg ON fg.genre_id = g.id \
         WHERE fg.film_id = ? ORDER BY g.name",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    let cast = sqlx::query_as::<_, CastMember>(
        "SELECT p.id AS person_id, p.name, p.profile_path, fp.character, fp.role, fp.sort_order \
         FROM persons p \
         JOIN film_persons fp ON fp.person_id = p.id \
         WHERE fp.film_id = ? ORDER BY fp.role, fp.sort_order",
    )
    .bind(id)
    .fetch_all(&state.db)
    .await?;

    Ok(FilmDetail { film, genres, cast })
}

#[tauri::command]
pub async fn delete_film(state: tauri::State<'_, AppState>, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM films WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn set_left_off_point(
    state: tauri::State<'_, AppState>,
    id: i64,
    seconds: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE films SET left_off_point = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
    )
    .bind(seconds.max(0))
    .bind(id)
    .execute(&state.db)
    .await?;
    Ok(())
}
