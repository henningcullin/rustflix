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

use crate::metadata::imdb::{PosterSize, PrincipalCredits, RatingsNode, TitleNode};

pub async fn apply_imdb_movie_details(
    conn: &mut SqliteConnection,
    movie_id: i64,
    details: &TitleNode,
) -> AppResult<Option<(PosterSize, String, String)>> {
    let current_poster_origin: Option<String> =
        sqlx::query_scalar("SELECT poster_origin FROM movies WHERE id = ?1")
            .bind(movie_id)
            .fetch_one(&mut *conn)
            .await?;

    let download_target = compute_imdb_poster(
        current_poster_origin.as_deref(),
        details.primary_image.as_ref().map(|image| image.url.as_str()),
        movie_id,
        "movie",
    );

    let overview = details
        .plot
        .as_ref()
        .and_then(|plot| plot.plot_text.as_ref())
        .and_then(|text| text.plain_text.as_deref());
    let year = details.release_year.as_ref().and_then(|release| release.year);
    let rating = imdb_rating(&details.ratings_summary);
    let genres_json = serde_json::to_string(
        &details
            .genres
            .as_ref()
            .map(|wrapper| {
                wrapper
                    .genres
                    .iter()
                    .map(|genre| genre.text.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let cast_json = build_imdb_cast_json(&details.principal_credits);
    let runtime_minutes = details
        .runtime
        .as_ref()
        .and_then(|runtime| runtime.seconds)
        .map(|seconds| seconds / 60);

    if let Some((_size, _url, local_path)) = download_target.as_ref() {
        sqlx::query(
            "UPDATE movies SET
                 provider = 'imdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 runtime_minutes = COALESCE(?7, runtime_minutes),
                 poster_path = ?8,
                 poster_origin = 'imdb',
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?9",
        )
        .bind(&details.id)
        .bind(overview)
        .bind(year)
        .bind(rating)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(runtime_minutes)
        .bind(local_path)
        .bind(movie_id)
        .execute(&mut *conn)
        .await?;
    } else {
        sqlx::query(
            "UPDATE movies SET
                 provider = 'imdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 runtime_minutes = COALESCE(?7, runtime_minutes),
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?8",
        )
        .bind(&details.id)
        .bind(overview)
        .bind(year)
        .bind(rating)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(runtime_minutes)
        .bind(movie_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(download_target)
}

pub async fn apply_imdb_show_details(
    conn: &mut SqliteConnection,
    show_id: i64,
    details: &TitleNode,
) -> AppResult<Option<(PosterSize, String, String)>> {
    let current_poster_origin: Option<String> =
        sqlx::query_scalar("SELECT poster_origin FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(&mut *conn)
            .await?;

    let download_target = compute_imdb_poster(
        current_poster_origin.as_deref(),
        details.primary_image.as_ref().map(|image| image.url.as_str()),
        show_id,
        "show",
    );

    let overview = details
        .plot
        .as_ref()
        .and_then(|plot| plot.plot_text.as_ref())
        .and_then(|text| text.plain_text.as_deref());
    let year = details.release_year.as_ref().and_then(|release| release.year);
    let rating = imdb_rating(&details.ratings_summary);
    let genres_json = serde_json::to_string(
        &details
            .genres
            .as_ref()
            .map(|wrapper| {
                wrapper
                    .genres
                    .iter()
                    .map(|genre| genre.text.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let cast_json = build_imdb_cast_json(&details.principal_credits);
    let first_air_date = details.release_date.as_ref().and_then(|date| {
        match (date.year, date.month, date.day) {
            (Some(y), Some(m), Some(d)) => Some(format!("{y:04}-{m:02}-{d:02}")),
            _ => None,
        }
    });

    if let Some((_size, _url, local_path)) = download_target.as_ref() {
        sqlx::query(
            "UPDATE shows SET
                 provider = 'imdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 first_air_date = COALESCE(?7, first_air_date),
                 poster_path = ?8,
                 poster_origin = 'imdb',
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?9",
        )
        .bind(&details.id)
        .bind(overview)
        .bind(year)
        .bind(rating)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(first_air_date.as_deref())
        .bind(local_path)
        .bind(show_id)
        .execute(&mut *conn)
        .await?;
    } else {
        sqlx::query(
            "UPDATE shows SET
                 provider = 'imdb',
                 provider_id = ?1,
                 overview = COALESCE(?2, overview),
                 year = COALESCE(?3, year),
                 rating = ?4,
                 genres = ?5,
                 top_cast = ?6,
                 first_air_date = COALESCE(?7, first_air_date),
                 metadata_synced_at = strftime('%s','now')
             WHERE id = ?8",
        )
        .bind(&details.id)
        .bind(overview)
        .bind(year)
        .bind(rating)
        .bind(&genres_json)
        .bind(&cast_json)
        .bind(first_air_date.as_deref())
        .bind(show_id)
        .execute(&mut *conn)
        .await?;
    }

    Ok(download_target)
}

fn imdb_rating(ratings: &Option<RatingsNode>) -> Option<f64> {
    let ratings = ratings.as_ref()?;
    let votes = ratings.vote_count.unwrap_or(0);

    if votes == 0 {
        // Unreleased titles: voteCount is 0 (not null). Treat as no rating.
        return None;
    }

    ratings.aggregate_rating
}

fn build_imdb_cast_json(credits: &[PrincipalCredits]) -> String {
    // Match on category.id == "cast" (stable lowercase id). The response's
    // category.text is the plural display name ("Stars") and would not match.
    let cast_block = credits.iter().find(|block| block.category.id == "cast");

    let payload: Vec<serde_json::Value> = cast_block
        .map(|block| {
            block
                .credits
                .iter()
                .take(10)
                .enumerate()
                .map(|(index, credit)| {
                    serde_json::json!({
                        "name": credit.name.name_text.text,
                        "character": credit.characters.first().map(|character| character.name.clone()),
                        "order": index,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    serde_json::to_string(&payload).unwrap_or_else(|_| "[]".to_string())
}

/// Returns Some((size, source_url, local_filename)) when the row's existing
/// poster_origin is not `'manual'` AND the details payload has an image
/// URL we can download.
fn compute_imdb_poster(
    current_origin: Option<&str>,
    image_url: Option<&str>,
    media_id: i64,
    kind: &str,
) -> Option<(PosterSize, String, String)> {
    if current_origin == Some("manual") {
        return None;
    }

    let url = image_url?;
    let filename = format!("{kind}-{media_id}.jpg");

    Some((PosterSize::Small, url.to_string(), filename))
}
