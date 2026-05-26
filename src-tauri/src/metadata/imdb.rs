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

// ---- GraphQL details ----

const GRAPHQL_URL: &str = "https://caching.graphql.imdb.com/";

const GRAPHQL_QUERY: &str = r#"
query TitleDetails($id: ID!) {
  title(id: $id) {
    id
    titleText { text }
    titleType { id }
    releaseYear { year endYear }
    releaseDate { day month year }
    plot { plotText { plainText } }
    ratingsSummary { aggregateRating voteCount }
    runtime { seconds }
    genres { genres { id text } }
    primaryImage { url width height }
    principalCredits(filter: { categories: ["director","writer","cast"] }) {
      category { id text }
      credits {
        name { id nameText { text } }
        ... on Cast { characters { name } }
      }
    }
  }
}
"#;

#[derive(Debug, Deserialize)]
struct GraphQLEnvelope {
    data: Option<GraphQLData>,
    #[serde(default)]
    errors: Vec<GraphQLError>,
}

#[derive(Debug, Deserialize)]
struct GraphQLData {
    title: Option<TitleNode>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct TitleNode {
    pub id: String,
    #[serde(default, rename = "titleText")]
    pub title_text: Option<TextNode>,
    #[serde(default, rename = "releaseYear")]
    pub release_year: Option<ReleaseYearNode>,
    #[serde(default, rename = "releaseDate")]
    pub release_date: Option<ReleaseDateNode>,
    #[serde(default)]
    pub plot: Option<PlotNode>,
    #[serde(default, rename = "ratingsSummary")]
    pub ratings_summary: Option<RatingsNode>,
    #[serde(default)]
    pub runtime: Option<RuntimeNode>,
    #[serde(default)]
    pub genres: Option<GenresWrapper>,
    #[serde(default, rename = "primaryImage")]
    pub primary_image: Option<PrimaryImage>,
    #[serde(default, rename = "principalCredits")]
    pub principal_credits: Vec<PrincipalCredits>,
}

#[derive(Debug, Deserialize)]
pub struct TextNode {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseYearNode {
    pub year: Option<i32>,
    #[serde(rename = "endYear")]
    pub end_year: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseDateNode {
    pub day: Option<i32>,
    pub month: Option<i32>,
    pub year: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PlotNode {
    #[serde(rename = "plotText")]
    pub plot_text: Option<PlotTextNode>,
}

#[derive(Debug, Deserialize)]
pub struct PlotTextNode {
    #[serde(rename = "plainText")]
    pub plain_text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RatingsNode {
    #[serde(rename = "aggregateRating")]
    pub aggregate_rating: Option<f64>,
    #[serde(rename = "voteCount")]
    pub vote_count: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RuntimeNode {
    pub seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct GenresWrapper {
    #[serde(default)]
    pub genres: Vec<GenreNode>,
}

#[derive(Debug, Deserialize)]
pub struct GenreNode {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct PrimaryImage {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct PrincipalCredits {
    pub category: CategoryNode,
    #[serde(default)]
    pub credits: Vec<CreditNode>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryNode {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct CreditNode {
    pub name: NameNode,
    #[serde(default)]
    pub characters: Vec<CharacterNode>,
}

#[derive(Debug, Deserialize)]
pub struct NameNode {
    #[serde(rename = "nameText")]
    pub name_text: TextNode,
}

#[derive(Debug, Deserialize)]
pub struct CharacterNode {
    pub name: String,
}

pub async fn fetch_movie_details(client: &Client, imdb_id: &str) -> AppResult<TitleNode> {
    fetch_details_internal(client, imdb_id).await
}

pub async fn fetch_show_details(client: &Client, imdb_id: &str) -> AppResult<TitleNode> {
    fetch_details_internal(client, imdb_id).await
}

async fn fetch_details_internal(client: &Client, imdb_id: &str) -> AppResult<TitleNode> {
    let body = serde_json::json!({
        "operationName": "TitleDetails",
        "variables": { "id": imdb_id },
        "query": GRAPHQL_QUERY,
    });

    let response = client
        .post(GRAPHQL_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(http_err)?;

    let status = response.status();
    if status == StatusCode::ACCEPTED {
        return Err(AppError::Other(
            "imdb_waf: graphql returned 202; see CLAUDE.md".to_string(),
        ));
    }
    if !status.is_success() {
        return Err(AppError::Other(format!(
            "imdb_rate_limited: graphql {status}"
        )));
    }

    let envelope: GraphQLEnvelope = response
        .json()
        .await
        .map_err(|error| AppError::Other(format!("imdb parse: graphql: {error}")))?;

    if let Some(first_error) = envelope.errors.first() {
        return Err(AppError::Other(format!(
            "imdb graphql: {}",
            first_error.message
        )));
    }

    let title = envelope
        .data
        .and_then(|data| data.title)
        .ok_or_else(|| AppError::Other(format!("imdb not_found: {imdb_id}")))?;

    if title.title_text.is_none() {
        return Err(AppError::Other(format!("imdb not_found: {imdb_id}")));
    }

    Ok(title)
}

// ---- Poster download ----

#[derive(Debug, Clone, Copy)]
pub enum PosterSize {
    Small,
    Hero,
}

impl PosterSize {
    fn segment(self) -> &'static str {
        match self {
            PosterSize::Small => "_V1_SX500_",
            PosterSize::Hero => "_V1_QL90_UX1280_",
        }
    }
}

pub async fn download_poster(
    client: &Client,
    image_url: &str,
    dest: &std::path::Path,
    size: PosterSize,
) -> AppResult<()> {
    use tokio::io::AsyncWriteExt;

    let url = rewrite_size(image_url, size);

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

fn rewrite_size(url: &str, size: PosterSize) -> String {
    if let Some(index) = url.rfind("_V1_") {
        let (head, tail) = url.split_at(index);
        // Strip everything between `_V1_` and the next `.`, replace with our segment.
        let dot = tail.rfind('.').unwrap_or(tail.len());
        return format!("{head}{}{}", size.segment(), &tail[dot..]);
    }
    url.to_string()
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

    #[test]
    fn rewrite_size_inserts_segment() {
        let url = "https://m.media-amazon.com/images/M/abc@._V1_.jpg";
        assert_eq!(
            rewrite_size(url, PosterSize::Small),
            "https://m.media-amazon.com/images/M/abc@._V1_SX500_.jpg"
        );
        assert_eq!(
            rewrite_size(url, PosterSize::Hero),
            "https://m.media-amazon.com/images/M/abc@._V1_QL90_UX1280_.jpg"
        );
    }

    #[test]
    fn rewrite_size_leaves_other_urls_alone() {
        let url = "https://example.com/image.jpg";
        assert_eq!(rewrite_size(url, PosterSize::Small), url);
    }
}

#[cfg(test)]
mod fixture_tests {
    use super::*;

    #[test]
    fn parses_suggestion_movie_response() {
        let raw = include_str!("../../tests/fixtures/imdb-suggestion-movie.json");
        let envelope: SuggestionEnvelope = serde_json::from_str(raw).unwrap();
        assert_eq!(envelope.d.len(), 3);
        let movies: Vec<_> = envelope
            .d
            .iter()
            .filter(|entry| entry.id.starts_with("tt") && entry.qid.as_deref() == Some("movie"))
            .collect();
        assert_eq!(movies.len(), 2);
        assert_eq!(movies[0].id, "tt0133093");
        assert_eq!(movies[0].l.as_deref(), Some("The Matrix"));
        assert_eq!(movies[0].y, Some(1999));
    }

    #[test]
    fn parses_graphql_movie_response() {
        let raw = include_str!("../../tests/fixtures/imdb-graphql-movie.json");
        let envelope: GraphQLEnvelope = serde_json::from_str(raw).unwrap();
        let title = envelope.data.unwrap().title.unwrap();
        assert_eq!(title.id, "tt0133093");
        assert_eq!(title.title_text.as_ref().unwrap().text, "The Matrix");
        assert_eq!(title.release_year.as_ref().unwrap().year, Some(1999));
        assert_eq!(title.runtime.as_ref().unwrap().seconds, Some(8160));
        assert_eq!(
            title.ratings_summary.as_ref().unwrap().aggregate_rating,
            Some(8.7),
        );
        let cast = title
            .principal_credits
            .iter()
            .find(|credit| credit.category.id == "cast")
            .unwrap();
        assert_eq!(cast.credits.len(), 2);
        assert_eq!(cast.credits[0].characters[0].name, "Neo");
    }

    #[test]
    fn parses_graphql_show_response() {
        let raw = include_str!("../../tests/fixtures/imdb-graphql-show.json");
        let envelope: GraphQLEnvelope = serde_json::from_str(raw).unwrap();
        let title = envelope.data.unwrap().title.unwrap();
        assert_eq!(title.id, "tt0903747");
        assert_eq!(title.release_year.as_ref().unwrap().end_year, Some(2013));
    }

    #[test]
    fn parses_graphql_edge_case_no_rating() {
        let raw = include_str!("../../tests/fixtures/imdb-graphql-edge.json");
        let envelope: GraphQLEnvelope = serde_json::from_str(raw).unwrap();
        let title = envelope.data.unwrap().title.unwrap();
        assert_eq!(title.ratings_summary.as_ref().unwrap().vote_count, Some(0));
        assert!(title.runtime.is_none());
        assert!(title.primary_image.is_none());
    }
}
