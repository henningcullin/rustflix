//! TMDB v3 client. Stays narrow: only the calls the worker needs.

use std::path::Path;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
use crate::metadata::dispatch::Provider;
use crate::metadata::matching::MatchCandidate;

const API_BASE: &str = "https://api.themoviedb.org/3";
const IMAGE_BASE: &str = "https://image.tmdb.org/t/p/w500";

#[derive(Debug, Deserialize)]
struct SearchEnvelope<T> {
    results: Vec<T>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TmdbMovieResult {
    pub id: i64,
    pub title: String,
    pub release_date: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TmdbShowResult {
    pub id: i64,
    pub name: String,
    pub first_air_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TmdbCredits {
    pub cast: Vec<TmdbCastMember>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TmdbCastMember {
    pub name: String,
    pub character: Option<String>,
    pub order: i64,
}

#[derive(Debug, Deserialize)]
pub struct TmdbGenre {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct TmdbMovieDetails {
    pub id: i64,
    #[allow(dead_code)]
    pub title: String,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f64>,
    pub runtime: Option<i64>,
    pub poster_path: Option<String>,
    pub genres: Vec<TmdbGenre>,
    pub credits: Option<TmdbCredits>,
}

#[derive(Debug, Deserialize)]
pub struct TmdbShowDetails {
    pub id: i64,
    #[allow(dead_code)]
    pub name: String,
    pub overview: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f64>,
    pub poster_path: Option<String>,
    pub genres: Vec<TmdbGenre>,
    pub credits: Option<TmdbCredits>,
}

pub async fn search_movie(
    client: &Client,
    api_key: &str,
    title: &str,
    year: Option<i32>,
) -> AppResult<Vec<MatchCandidate>> {
    let mut request = client
        .get(format!("{API_BASE}/search/movie"))
        .query(&[("api_key", api_key), ("query", title)]);
    if let Some(year_value) = year {
        let year_string = year_value.to_string();
        request = request.query(&[("year", year_string.as_str())]);
    }

    let response = request.send().await.map_err(http_err)?;
    let envelope: SearchEnvelope<TmdbMovieResult> =
        parse_response(response, "search/movie").await?;

    Ok(envelope
        .results
        .into_iter()
        .map(|raw| MatchCandidate {
            provider: Provider::Tmdb,
            provider_id: raw.id.to_string(),
            title: raw.title,
            year: parse_year(raw.release_date.as_deref()),
        })
        .collect())
}

pub async fn search_show(
    client: &Client,
    api_key: &str,
    title: &str,
    year: Option<i32>,
) -> AppResult<Vec<MatchCandidate>> {
    let mut request = client
        .get(format!("{API_BASE}/search/tv"))
        .query(&[("api_key", api_key), ("query", title)]);
    if let Some(year_value) = year {
        let year_string = year_value.to_string();
        request = request.query(&[("first_air_date_year", year_string.as_str())]);
    }

    let response = request.send().await.map_err(http_err)?;
    let envelope: SearchEnvelope<TmdbShowResult> =
        parse_response(response, "search/tv").await?;

    Ok(envelope
        .results
        .into_iter()
        .map(|raw| MatchCandidate {
            provider: Provider::Tmdb,
            provider_id: raw.id.to_string(),
            title: raw.name,
            year: parse_year(raw.first_air_date.as_deref()),
        })
        .collect())
}

pub async fn fetch_movie_details(
    client: &Client,
    api_key: &str,
    tmdb_id: &str,
) -> AppResult<TmdbMovieDetails> {
    let response = client
        .get(format!("{API_BASE}/movie/{tmdb_id}"))
        .query(&[("api_key", api_key), ("append_to_response", "credits")])
        .send()
        .await
        .map_err(http_err)?;

    parse_response(response, "movie/details").await
}

pub async fn fetch_show_details(
    client: &Client,
    api_key: &str,
    tmdb_id: &str,
) -> AppResult<TmdbShowDetails> {
    let response = client
        .get(format!("{API_BASE}/tv/{tmdb_id}"))
        .query(&[("api_key", api_key), ("append_to_response", "credits")])
        .send()
        .await
        .map_err(http_err)?;

    parse_response(response, "tv/details").await
}

/// Downloads `poster_path` (a relative TMDB path like `/abc.jpg`) into
/// `dest`. Streams the response body to keep memory bounded.
pub async fn download_poster(
    client: &Client,
    poster_path: &str,
    dest: &Path,
) -> AppResult<()> {
    let url = format!("{IMAGE_BASE}{poster_path}");
    let mut response = client.get(&url).send().await.map_err(http_err)?;

    if !response.status().is_success() {
        return Err(AppError::Other(format!(
            "poster download failed: {} {}",
            response.status(),
            url
        )));
    }

    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = tokio::fs::File::create(dest).await?;
    while let Some(chunk) = response.chunk().await.map_err(http_err)? {
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(())
}

fn http_err(error: reqwest::Error) -> AppError {
    AppError::Other(format!("tmdb http: {error}"))
}

async fn parse_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
    endpoint: &str,
) -> AppResult<T> {
    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(AppError::Other(format!(
            "auth_required: {endpoint} returned 401"
        )));
    }
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::Other(format!(
            "tmdb {endpoint} returned {status}: {body}"
        )));
    }

    response
        .json::<T>()
        .await
        .map_err(|error| AppError::Other(format!("tmdb {endpoint} parse: {error}")))
}

fn parse_year(date: Option<&str>) -> Option<i32> {
    date.and_then(|d| d.get(0..4)).and_then(|year| year.parse().ok())
}
