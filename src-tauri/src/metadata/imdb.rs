//! IMDB metadata client.
//!
//! HTML scraping of www.imdb.com is blocked by AWS WAF as of late 2025.
//! This module uses two undocumented JSON endpoints instead:
//!
//!   * Suggestion API for search:
//!     https://v3.sg.media-imdb.com/suggestion/<letter>/<slug>.json
//!   * GraphQL for details:
//!     https://caching.graphql.imdb.com/
//!
//! ## TOS disclaimer
//!
//! The GraphQL response carries this disclaimer on every payload:
//!
//! > Public, commercial, and/or non-private use of the IMDb data
//! > provided by this API is not allowed. For limited non-commercial use
//! > of IMDb data and the associated requirements see
//! > https://help.imdb.com/article/imdb/general-information/can-i-use-imdb-data-in-my-software/G5JTRESSHJBBHTGX
//!
//! rustflix neither redistributes IMDb data nor uses it commercially.
//! Users are responsible for their own compliance with the linked terms.
//! The tmdb_only mode remains a functional escape hatch.

use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::metadata::dispatch::Provider;
use crate::metadata::matching::MatchCandidate;

const SUGGESTION_BASE: &str = "https://v3.sg.media-imdb.com/suggestion";

#[derive(Debug, Deserialize)]
struct SuggestionEnvelope {
    #[serde(default)]
    d: Vec<SuggestionEntry>,
}

#[derive(Debug, Deserialize)]
struct SuggestionEntry {
    id: String,
    #[serde(default)]
    l: Option<String>,
    #[serde(default)]
    y: Option<i32>,
    #[serde(default)]
    qid: Option<String>,
}

pub async fn search_movie(
    client: &Client,
    title: &str,
    year: Option<i32>,
) -> AppResult<Vec<MatchCandidate>> {
    search_internal(client, title, year, &["movie"]).await
}

pub async fn search_show(
    client: &Client,
    title: &str,
    year: Option<i32>,
) -> AppResult<Vec<MatchCandidate>> {
    search_internal(client, title, year, &["tvSeries", "tvMiniSeries"]).await
}

async fn search_internal(
    client: &Client,
    title: &str,
    year: Option<i32>,
    qid_filter: &[&str],
) -> AppResult<Vec<MatchCandidate>> {
    let slug = slugify(title);
    if slug.is_empty() {
        return Ok(vec![]);
    }

    // Try year-augmented slug first (better CDN filtering), fall back to plain slug.
    let envelope = match year {
        Some(year_value) => {
            let augmented = format!("{slug}_{year_value}");
            let result = fetch_suggestion(client, &augmented).await?;
            if result.d.is_empty() {
                fetch_suggestion(client, &slug).await?
            } else {
                result
            }
        }
        None => fetch_suggestion(client, &slug).await?,
    };

    Ok(envelope
        .d
        .into_iter()
        .filter(|entry| entry.id.starts_with("tt"))
        .filter(|entry| match entry.qid.as_deref() {
            Some(qid) => qid_filter.contains(&qid),
            None => false,
        })
        .filter_map(|entry| {
            entry.l.map(|title| MatchCandidate {
                provider: Provider::Imdb,
                provider_id: entry.id,
                title,
                year: entry.y,
            })
        })
        .collect())
}

async fn fetch_suggestion(client: &Client, slug: &str) -> AppResult<SuggestionEnvelope> {
    let first = slug.chars().next().unwrap_or('a');
    let shard = first.to_lowercase().next().unwrap_or('a');
    let url = format!("{SUGGESTION_BASE}/{shard}/{slug}.json");

    let response = client.get(&url).send().await.map_err(http_err)?;
    let status = response.status();
    if status == StatusCode::ACCEPTED {
        return Err(AppError::Other(
            "imdb_waf: suggestion endpoint returned 202; see CLAUDE.md".to_string(),
        ));
    }
    if !status.is_success() {
        return Err(AppError::Other(format!(
            "imdb_rate_limited: suggestion {status}"
        )));
    }
    response
        .json::<SuggestionEnvelope>()
        .await
        .map_err(|error| AppError::Other(format!("imdb parse: suggestion: {error}")))
}

/// Lowercase + non-alphanumeric → `_`, no trailing underscore.
fn slugify(title: &str) -> String {
    let mut output = String::with_capacity(title.len());
    let mut last_was_underscore = false;
    for character in title.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            last_was_underscore = false;
        } else if !last_was_underscore && !output.is_empty() {
            output.push('_');
            last_was_underscore = true;
        }
    }
    if output.ends_with('_') {
        output.pop();
    }
    output
}

fn http_err(error: reqwest::Error) -> AppError {
    if error.is_status() {
        return AppError::Other(format!("imdb_rate_limited: {error}"));
    }
    AppError::Other(format!("imdb http: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_lowercases_and_replaces_non_alnum() {
        assert_eq!(slugify("The Matrix"), "the_matrix");
        assert_eq!(slugify("It's a Wonderful Life"), "it_s_a_wonderful_life");
        assert_eq!(slugify("Pokémon"), "pok_mon");
        assert_eq!(slugify("Dune (2021)"), "dune_2021");
    }

    #[test]
    fn slugify_handles_trailing_punctuation() {
        assert_eq!(slugify("The End."), "the_end");
    }
}
