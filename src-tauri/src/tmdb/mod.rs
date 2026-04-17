use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::error::{AppError, AppResult};
use crate::images;
use crate::state::AppState;

const TMDB_BASE: &str = "https://api.themoviedb.org/3";
const TMDB_IMG_BASE: &str = "https://image.tmdb.org/t/p/original";

pub struct TmdbClient {
    http: reqwest::Client,
    api_key: RwLock<Option<String>>,
}

impl TmdbClient {
    pub fn new(initial_key: Option<String>) -> Arc<Self> {
        let http = reqwest::Client::builder()
            .user_agent("rustflix/0.1")
            .build()
            .expect("http client");
        Arc::new(Self {
            http,
            api_key: RwLock::new(initial_key),
        })
    }

    pub async fn set_api_key(&self, key: Option<String>) {
        *self.api_key.write().await = key;
    }

    async fn key(&self) -> AppResult<String> {
        self.api_key
            .read()
            .await
            .clone()
            .ok_or(AppError::MissingTmdbKey)
    }

    pub async fn search(&self, query: &str) -> AppResult<TmdbSearchResponse> {
        let key = self.key().await?;
        let encoded = urlencoding_encode(query);
        let url = format!(
            "{TMDB_BASE}/search/movie?api_key={key}&query={encoded}&include_adult=false"
        );
        let resp = self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<TmdbSearchResponse>()
            .await?;
        Ok(resp)
    }

    pub async fn movie_details(&self, tmdb_id: i64) -> AppResult<TmdbMovieDetails> {
        let key = self.key().await?;
        let url = format!(
            "{TMDB_BASE}/movie/{tmdb_id}?api_key={key}&append_to_response=credits,external_ids"
        );
        let resp = self
            .http
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<TmdbMovieDetails>()
            .await?;
        Ok(resp)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSearchResponse {
    pub page: i64,
    pub results: Vec<TmdbSearchResult>,
    #[serde(default)]
    pub total_pages: i64,
    #[serde(default)]
    pub total_results: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbSearchResult {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub vote_average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbMovieDetails {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    pub overview: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub runtime: Option<i64>,
    #[serde(default)]
    pub vote_average: Option<f64>,
    #[serde(default)]
    pub poster_path: Option<String>,
    #[serde(default)]
    pub backdrop_path: Option<String>,
    #[serde(default)]
    pub genres: Vec<TmdbGenre>,
    #[serde(default)]
    pub credits: Option<TmdbCredits>,
    #[serde(default)]
    pub external_ids: Option<TmdbExternalIds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbGenre {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmdbCredits {
    #[serde(default)]
    pub cast: Vec<TmdbCastMember>,
    #[serde(default)]
    pub crew: Vec<TmdbCrewMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbCastMember {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub character: Option<String>,
    #[serde(default)]
    pub profile_path: Option<String>,
    #[serde(default)]
    pub order: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmdbCrewMember {
    pub id: i64,
    pub name: String,
    pub job: String,
    #[serde(default)]
    pub profile_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TmdbExternalIds {
    #[serde(default)]
    pub imdb_id: Option<String>,
}

#[tauri::command]
pub async fn tmdb_search(
    state: tauri::State<'_, AppState>,
    query: String,
) -> AppResult<Vec<TmdbSearchResult>> {
    let resp = state.tmdb.search(&query).await?;
    Ok(resp.results)
}

#[tauri::command]
pub async fn tmdb_import_film(
    state: tauri::State<'_, AppState>,
    file_path: String,
    tmdb_id: i64,
) -> AppResult<i64> {
    let details = state.tmdb.movie_details(tmdb_id).await?;

    let imdb_id = details.external_ids.as_ref().and_then(|e| e.imdb_id.clone());

    let film_id: i64 = sqlx::query_scalar(
        "INSERT INTO films (file_path, tmdb_id, imdb_id, title, original_title, overview, \
                             release_date, runtime, rating, poster_path, backdrop_path) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) \
         ON CONFLICT(file_path) DO UPDATE SET \
           tmdb_id = excluded.tmdb_id, \
           imdb_id = excluded.imdb_id, \
           title = excluded.title, \
           original_title = excluded.original_title, \
           overview = excluded.overview, \
           release_date = excluded.release_date, \
           runtime = excluded.runtime, \
           rating = excluded.rating, \
           poster_path = excluded.poster_path, \
           backdrop_path = excluded.backdrop_path, \
           updated_at = CURRENT_TIMESTAMP \
         RETURNING id",
    )
    .bind(&file_path)
    .bind(details.id)
    .bind(&imdb_id)
    .bind(&details.title)
    .bind(&details.original_title)
    .bind(&details.overview)
    .bind(&details.release_date)
    .bind(details.runtime)
    .bind(details.vote_average)
    .bind(&details.poster_path)
    .bind(&details.backdrop_path)
    .fetch_one(&state.db)
    .await?;

    // Replace genres.
    sqlx::query("DELETE FROM film_genres WHERE film_id = ?")
        .bind(film_id)
        .execute(&state.db)
        .await?;
    for genre in &details.genres {
        sqlx::query("INSERT OR IGNORE INTO genres (id, name) VALUES (?, ?)")
            .bind(genre.id)
            .bind(&genre.name)
            .execute(&state.db)
            .await?;
        sqlx::query("INSERT OR IGNORE INTO film_genres (film_id, genre_id) VALUES (?, ?)")
            .bind(film_id)
            .bind(genre.id)
            .execute(&state.db)
            .await?;
    }

    // Replace cast.
    sqlx::query("DELETE FROM film_persons WHERE film_id = ?")
        .bind(film_id)
        .execute(&state.db)
        .await?;

    if let Some(credits) = &details.credits {
        for member in credits.cast.iter().take(30) {
            let person_id: i64 = sqlx::query_scalar(
                "INSERT INTO persons (tmdb_id, name, profile_path) VALUES (?, ?, ?) \
                 ON CONFLICT(tmdb_id) DO UPDATE SET name = excluded.name, profile_path = excluded.profile_path \
                 RETURNING id",
            )
            .bind(member.id)
            .bind(&member.name)
            .bind(&member.profile_path)
            .fetch_one(&state.db)
            .await?;

            sqlx::query(
                "INSERT OR REPLACE INTO film_persons (film_id, person_id, role, character, sort_order) \
                 VALUES (?, ?, 'actor', ?, ?)",
            )
            .bind(film_id)
            .bind(person_id)
            .bind(&member.character)
            .bind(member.order.unwrap_or(0))
            .execute(&state.db)
            .await?;
        }

        for (i, crew) in credits
            .crew
            .iter()
            .filter(|c| c.job.eq_ignore_ascii_case("director"))
            .enumerate()
        {
            let person_id: i64 = sqlx::query_scalar(
                "INSERT INTO persons (tmdb_id, name, profile_path) VALUES (?, ?, ?) \
                 ON CONFLICT(tmdb_id) DO UPDATE SET name = excluded.name, profile_path = excluded.profile_path \
                 RETURNING id",
            )
            .bind(crew.id)
            .bind(&crew.name)
            .bind(&crew.profile_path)
            .fetch_one(&state.db)
            .await?;

            sqlx::query(
                "INSERT OR REPLACE INTO film_persons (film_id, person_id, role, character, sort_order) \
                 VALUES (?, ?, 'director', NULL, ?)",
            )
            .bind(film_id)
            .bind(person_id)
            .bind(i as i64)
            .execute(&state.db)
            .await?;
        }
    }

    // Download cover.
    if let Some(poster) = &details.poster_path {
        let url = format!("{TMDB_IMG_BASE}{poster}");
        if let Err(err) = images::download_and_store_cover(
            reqwest_client(&state.tmdb),
            &url,
            &state.app_data_dir,
            film_id,
        )
        .await
        {
            eprintln!("cover download failed: {err}");
        } else {
            sqlx::query("UPDATE films SET poster_path = ? WHERE id = ?")
                .bind(format!("covers/{film_id}"))
                .bind(film_id)
                .execute(&state.db)
                .await?;
        }
    }

    Ok(film_id)
}

fn reqwest_client(client: &TmdbClient) -> &reqwest::Client {
    &client.http
}

fn urlencoding_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
