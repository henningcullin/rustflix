//! Pure matching logic. Given a scanner-derived title + year and a list of
//! provider search results, decide whether one is a confident match.

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchCandidate {
    pub provider_id: String,
    pub title: String,
    pub year: Option<i32>,
}

/// Returns `Some(candidate)` only when query_title (after normalization)
/// matches exactly one candidate whose year is within ±1 of query_year.
/// Otherwise returns `None` so the caller can leave the item unlinked.
pub fn pick_confident_match<'a>(
    query_title: &str,
    query_year: Option<i32>,
    candidates: &'a [MatchCandidate],
) -> Option<&'a MatchCandidate> {
    let normalized_query = normalize(query_title);

    let surviving: Vec<&MatchCandidate> = candidates
        .iter()
        .filter(|candidate| year_matches(candidate.year, query_year))
        .filter(|candidate| normalize(&candidate.title) == normalized_query)
        .collect();

    if surviving.len() == 1 {
        Some(surviving[0])
    } else {
        None
    }
}

fn year_matches(candidate_year: Option<i32>, query_year: Option<i32>) -> bool {
    match (candidate_year, query_year) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(a), Some(b)) => (a - b).abs() <= 1,
    }
}

/// NFKD-fold, drop diacritics, strip "(US)" / "(2005)" disambiguators,
/// drop leading article "the"/"a"/"an", lowercase, collapse whitespace.
pub fn normalize(raw: &str) -> String {
    let folded: String = raw.nfkd().filter(|character| character.is_ascii()).collect();
    let lower = folded.to_lowercase();

    let without_parens = strip_parenthetical(&lower);
    let without_article = strip_leading_article(&without_parens);

    without_article
        .split_whitespace()
        .filter(|token| !token.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
}

fn strip_parenthetical(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut depth = 0i32;

    for character in input.chars() {
        match character {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ if depth == 0 => output.push(character),
            _ => {}
        }
    }

    output
}

fn strip_leading_article(input: &str) -> String {
    let trimmed = input.trim_start();
    for article in ["the ", "a ", "an "] {
        if let Some(rest) = trimmed.strip_prefix(article) {
            return rest.to_string();
        }
    }
    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: &str, title: &str, year: Option<i32>) -> MatchCandidate {
        MatchCandidate {
            provider_id: id.to_string(),
            title: title.to_string(),
            year,
        }
    }

    #[test]
    fn unique_match_returns_it() {
        let candidates = vec![candidate("1", "Breaking Bad", Some(2008))];
        let picked = pick_confident_match("Breaking Bad", Some(2008), &candidates);
        assert_eq!(picked.map(|m| m.provider_id.as_str()), Some("1"));
    }

    #[test]
    fn the_office_us_vs_uk_resolved_by_year() {
        let candidates = vec![
            candidate("us", "The Office (US)", Some(2005)),
            candidate("uk", "The Office", Some(2001)),
        ];
        let picked = pick_confident_match("The Office", Some(2005), &candidates);
        assert_eq!(picked.map(|m| m.provider_id.as_str()), Some("us"));
    }

    #[test]
    fn ambiguous_without_year_returns_none() {
        let candidates = vec![
            candidate("us", "The Office (US)", Some(2005)),
            candidate("uk", "The Office", Some(2001)),
        ];
        let picked = pick_confident_match("The Office", None, &candidates);
        assert!(picked.is_none());
    }

    #[test]
    fn nfkd_fold_pokemon() {
        let candidates = vec![candidate("1", "Pokemon", Some(1997))];
        let picked = pick_confident_match("Pokémon", None, &candidates);
        assert_eq!(picked.map(|m| m.provider_id.as_str()), Some("1"));
    }

    #[test]
    fn year_plus_minus_one_accepted() {
        let candidates = vec![candidate("1", "Foo", Some(2009))];
        let picked = pick_confident_match("Foo", Some(2010), &candidates);
        assert!(picked.is_some());
    }

    #[test]
    fn year_more_than_one_off_rejected() {
        let candidates = vec![candidate("1", "Foo", Some(2008))];
        let picked = pick_confident_match("Foo", Some(2010), &candidates);
        assert!(picked.is_none());
    }

    #[test]
    fn two_matches_in_same_year_window_returns_none() {
        let candidates = vec![
            candidate("1", "Foo", Some(2010)),
            candidate("2", "Foo", Some(2011)),
        ];
        let picked = pick_confident_match("Foo", Some(2010), &candidates);
        assert!(picked.is_none());
    }

    #[test]
    fn leading_article_stripped() {
        let candidates = vec![candidate("1", "The Matrix", Some(1999))];
        let picked = pick_confident_match("Matrix", Some(1999), &candidates);
        assert!(picked.is_some());
    }

    #[test]
    fn parenthetical_year_stripped() {
        let candidates = vec![candidate("1", "Dune (2021)", Some(2021))];
        let picked = pick_confident_match("Dune", Some(2021), &candidates);
        assert!(picked.is_some());
    }

    #[test]
    fn whitespace_collapsed() {
        let candidates = vec![candidate("1", "Foo   Bar", Some(2010))];
        let picked = pick_confident_match("Foo Bar", Some(2010), &candidates);
        assert!(picked.is_some());
    }
}
