//! Provider routing for the metadata worker. Owns the `Provider` enum,
//! the `ParkReason` enum, and the `providers_for_mode` walk-builder.
//!
//! The worker reads `metadata_mode` per job and consults this module to
//! decide which providers to try in which order.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Tmdb,
    Imdb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParkReason {
    TmdbAuthRequired,
    NoProviderAvailable,
}

impl ParkReason {
    pub fn sentinel(self) -> &'static str {
        match self {
            ParkReason::TmdbAuthRequired => "tmdb_auth_required",
            ParkReason::NoProviderAvailable => "no_provider_available",
        }
    }
}

/// Returns the ordered list of providers to try for a given mode and
/// key state, or a typed `ParkReason` when no provider can run.
pub fn providers_for_mode(
    mode: &str,
    has_tmdb_key: bool,
) -> Result<Vec<Provider>, ParkReason> {
    use Provider::*;

    match mode {
        "off" => Ok(vec![]),
        "tmdb_only" => {
            if has_tmdb_key {
                Ok(vec![Tmdb])
            } else {
                Err(ParkReason::NoProviderAvailable)
            }
        }
        "imdb_only" => Ok(vec![Imdb]),
        "prefer_imdb" => {
            if has_tmdb_key {
                Ok(vec![Imdb, Tmdb])
            } else {
                Ok(vec![Imdb])
            }
        }
        _ => {
            // prefer_tmdb (default) and unknown modes.
            if has_tmdb_key {
                Ok(vec![Tmdb, Imdb])
            } else {
                Ok(vec![Imdb])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_returns_empty_walk() {
        assert_eq!(providers_for_mode("off", true).unwrap(), Vec::<Provider>::new());
        assert_eq!(providers_for_mode("off", false).unwrap(), Vec::<Provider>::new());
    }

    #[test]
    fn tmdb_only_with_key_returns_tmdb() {
        assert_eq!(providers_for_mode("tmdb_only", true).unwrap(), vec![Provider::Tmdb]);
    }

    #[test]
    fn tmdb_only_without_key_parks() {
        let result = providers_for_mode("tmdb_only", false);
        assert_eq!(result.unwrap_err(), ParkReason::NoProviderAvailable);
    }

    #[test]
    fn imdb_only_returns_imdb_always() {
        assert_eq!(providers_for_mode("imdb_only", true).unwrap(), vec![Provider::Imdb]);
        assert_eq!(providers_for_mode("imdb_only", false).unwrap(), vec![Provider::Imdb]);
    }

    #[test]
    fn prefer_tmdb_with_key_returns_tmdb_then_imdb() {
        assert_eq!(
            providers_for_mode("prefer_tmdb", true).unwrap(),
            vec![Provider::Tmdb, Provider::Imdb],
        );
    }

    #[test]
    fn prefer_tmdb_without_key_returns_imdb_only() {
        assert_eq!(
            providers_for_mode("prefer_tmdb", false).unwrap(),
            vec![Provider::Imdb],
        );
    }

    #[test]
    fn prefer_imdb_with_key_returns_imdb_then_tmdb() {
        assert_eq!(
            providers_for_mode("prefer_imdb", true).unwrap(),
            vec![Provider::Imdb, Provider::Tmdb],
        );
    }

    #[test]
    fn prefer_imdb_without_key_returns_imdb_only() {
        assert_eq!(
            providers_for_mode("prefer_imdb", false).unwrap(),
            vec![Provider::Imdb],
        );
    }

    #[test]
    fn unknown_mode_falls_back_to_prefer_tmdb_default() {
        assert_eq!(
            providers_for_mode("garbage", true).unwrap(),
            vec![Provider::Tmdb, Provider::Imdb],
        );
        assert_eq!(
            providers_for_mode("garbage", false).unwrap(),
            vec![Provider::Imdb],
        );
    }

    #[test]
    fn sentinel_values_match_db_strings() {
        assert_eq!(ParkReason::TmdbAuthRequired.sentinel(), "tmdb_auth_required");
        assert_eq!(ParkReason::NoProviderAvailable.sentinel(), "no_provider_available");
    }
}
