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
///
/// While the IMDB module is unimplemented (PR A of the rollout),
/// `imdb_only` and `prefer_imdb` degrade to `[Tmdb]` when a key is
/// present, else park as `NoProviderAvailable`. PR B replaces those
/// branches with real IMDB walks.
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
        // Until PR B lands, IMDB modes degrade to TMDB when possible.
        "imdb_only" | "prefer_imdb" => {
            if has_tmdb_key {
                Ok(vec![Tmdb])
            } else {
                Err(ParkReason::NoProviderAvailable)
            }
        }
        _ => {
            // prefer_tmdb (default). IMDB fallback arrives in PR B.
            if has_tmdb_key {
                Ok(vec![Tmdb])
            } else {
                Err(ParkReason::NoProviderAvailable)
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
    fn imdb_only_without_key_parks_in_pr_a_degrade() {
        let result = providers_for_mode("imdb_only", false);
        assert_eq!(result.unwrap_err(), ParkReason::NoProviderAvailable);
    }

    #[test]
    fn imdb_only_with_key_degrades_to_tmdb_in_pr_a() {
        assert_eq!(providers_for_mode("imdb_only", true).unwrap(), vec![Provider::Tmdb]);
    }

    #[test]
    fn prefer_tmdb_with_key_returns_tmdb() {
        assert_eq!(providers_for_mode("prefer_tmdb", true).unwrap(), vec![Provider::Tmdb]);
    }

    #[test]
    fn prefer_tmdb_without_key_parks_in_pr_a() {
        let result = providers_for_mode("prefer_tmdb", false);
        assert_eq!(result.unwrap_err(), ParkReason::NoProviderAvailable);
    }

    #[test]
    fn prefer_imdb_with_key_returns_tmdb_in_pr_a() {
        assert_eq!(providers_for_mode("prefer_imdb", true).unwrap(), vec![Provider::Tmdb]);
    }

    #[test]
    fn unknown_mode_treated_as_prefer_tmdb_default() {
        assert_eq!(providers_for_mode("garbage", true).unwrap(), vec![Provider::Tmdb]);
        assert_eq!(
            providers_for_mode("garbage", false).unwrap_err(),
            ParkReason::NoProviderAvailable,
        );
    }

    #[test]
    fn sentinel_values_match_db_strings() {
        assert_eq!(ParkReason::TmdbAuthRequired.sentinel(), "tmdb_auth_required");
        assert_eq!(ParkReason::NoProviderAvailable.sentinel(), "no_provider_available");
    }
}
