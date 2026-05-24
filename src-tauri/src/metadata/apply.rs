//! Pure DB writes for a fetched-and-matched TMDB payload. Called inside
//! the worker's per-job transaction. Never overwrites a manual poster;
//! the caller is responsible for checking metadata_locked beforehand.

use sqlx::SqliteConnection;

use crate::error::AppResult;
use crate::metadata::tmdb::{TmdbCastMember, TmdbMovieDetails, TmdbShowDetails};

/// Apply a fetched movie payload onto an existing `movies` row. Returns
/// `Some(extension)` if the caller should download the poster into
/// `movie-{id}.{extension}`, or `None` if the current row has a manual
/// poster (must not be overwritten).
pub async fn apply_movie_details(
    conn: &mut SqliteConnection,
    movie_id: i64,
    details: &TmdbMovieDetails,
) -> AppResult<Option<String>> {
    let current_poster_origin: Option<String> =
        sqlx::query_scalar("SELECT poster_origin FROM movies WHERE id = ?1")
            .bind(movie_id)
            .fetch_one(&mut *conn)
            .await?;

    let download_extension =
        compute_poster_extension(current_poster_origin.as_deref(), details.poster_path.as_deref());

    let genres_json = serde_json::to_string(
        &details.genres.iter().map(|g| g.name.clone()).collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let cast_json = build_cast_json(details.credits.as_ref().map(|c| &c.cast));
    let year = parse_year(details.release_date.as_deref());

    if let Some(extension) = download_extension.as_ref() {
        let local_path = format!("movie-{movie_id}.{extension}");
        sqlx::query(
            "UPDATE movies SET
                 provider = 'tmdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 runtime_minutes = ?7,
                 poster_path = ?8,
                 poster_origin = 'tmdb',
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?9",
        )
        .bind(details.id.to_string())
        .bind(details.overview.as_deref())
        .bind(year)
        .bind(details.vote_average)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(details.runtime)
        .bind(&local_path)
        .bind(movie_id)
        .execute(&mut *conn)
        .await?;
    } else {
        sqlx::query(
            "UPDATE movies SET
                 provider = 'tmdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 runtime_minutes = ?7,
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?8",
        )
        .bind(details.id.to_string())
        .bind(details.overview.as_deref())
        .bind(year)
        .bind(details.vote_average)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(details.runtime)
        .bind(movie_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(download_extension)
}

pub async fn apply_show_details(
    conn: &mut SqliteConnection,
    show_id: i64,
    details: &TmdbShowDetails,
) -> AppResult<Option<String>> {
    let current_poster_origin: Option<String> =
        sqlx::query_scalar("SELECT poster_origin FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(&mut *conn)
            .await?;

    let download_extension =
        compute_poster_extension(current_poster_origin.as_deref(), details.poster_path.as_deref());

    let genres_json = serde_json::to_string(
        &details.genres.iter().map(|g| g.name.clone()).collect::<Vec<_>>(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let cast_json = build_cast_json(details.credits.as_ref().map(|c| &c.cast));
    let year = parse_year(details.first_air_date.as_deref());

    if let Some(extension) = download_extension.as_ref() {
        let local_path = format!("show-{show_id}.{extension}");
        sqlx::query(
            "UPDATE shows SET
                 provider = 'tmdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 first_air_date = ?7,
                 poster_path = ?8,
                 poster_origin = 'tmdb',
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?9",
        )
        .bind(details.id.to_string())
        .bind(details.overview.as_deref())
        .bind(year)
        .bind(details.vote_average)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(details.first_air_date.as_deref())
        .bind(&local_path)
        .bind(show_id)
        .execute(&mut *conn)
        .await?;
    } else {
        sqlx::query(
            "UPDATE shows SET
                 provider = 'tmdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 first_air_date = ?7,
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?8",
        )
        .bind(details.id.to_string())
        .bind(details.overview.as_deref())
        .bind(year)
        .bind(details.vote_average)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(details.first_air_date.as_deref())
        .bind(show_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(download_extension)
}

fn build_cast_json(cast: Option<&Vec<TmdbCastMember>>) -> String {
    let trimmed: Vec<_> = cast
        .map(|members| members.iter().take(10).collect())
        .unwrap_or_default();

    let payload: Vec<serde_json::Value> = trimmed
        .into_iter()
        .map(|member| {
            serde_json::json!({
                "name": member.name,
                "character": member.character,
                "order": member.order,
            })
        })
        .collect();

    serde_json::to_string(&payload).unwrap_or_else(|_| "[]".to_string())
}

fn compute_poster_extension(
    current_origin: Option<&str>,
    poster_path: Option<&str>,
) -> Option<String> {
    if current_origin == Some("manual") {
        return None;
    }

    let path = poster_path?;
    let extension = path
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("jpg")
        .to_lowercase();

    Some(extension)
}

fn parse_year(date: Option<&str>) -> Option<i32> {
    date.and_then(|d| d.get(0..4)).and_then(|year| year.parse().ok())
}
