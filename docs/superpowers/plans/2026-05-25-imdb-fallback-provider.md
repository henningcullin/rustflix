# IMDB Fallback Provider Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add IMDB as a user-selectable second metadata provider alongside TMDB so users without a TMDB API key can still sync metadata. Mode-driven worker dispatch picks one of `off | tmdb_only | imdb_only | prefer_tmdb | prefer_imdb` (default `prefer_tmdb`). IMDB access uses two undocumented JSON endpoints because HTML scraping is blocked by AWS WAF as of late 2025.

**Architecture:** A new `metadata::dispatch` module owns the `Provider` enum and `providers_for_mode` mode-to-walk mapping. The worker reads `metadata_mode` per job, walks providers, classifies the outcome (matched / TMDB-auth-park / backoff / delete). HTTP stays outside the SQLite transaction; the tx wraps only the lock re-check + apply + delete-job. Hand-linked rows take a fast path that bypasses mode entirely. Settings infrastructure becomes a generic `get_app_setting` / `set_app_setting` Tauri pair with server-side `validate()` and an `on_setting_changed` dispatch for post-write side-effects; the bespoke `set_tmdb_api_key` command is removed.

**Tech Stack:** Rust (Tauri 2, sqlx 0.8, reqwest 0.12, serde_json — all already in deps). SvelteKit 5 (Svelte 5 runes, shadcn-svelte). The IMDB module uses no HTML parser — `serde_json` over `reqwest::Client` covers both endpoints. Spec at `docs/superpowers/specs/2026-05-25-imdb-fallback-provider-design.md`.

> Branch numbering: max existing is fix/42 (the spec). Implementation lives in fix/43, fix/44, fix/45.

---

## File Structure

### fix/43 — settings infra + mode wiring + sentinel rename

Behaviour after this PR: identical to today for default mode. `imdb_only` and `prefer_imdb` modes are pickable in the UI but degrade to `[Tmdb]` (or park as `NoProviderAvailable` when no key) until fix/44 lands the IMDB module.

- **Create**
  - `src-tauri/src/metadata/dispatch.rs` — `Provider` enum, `ParkReason` enum, `providers_for_mode` helper, unit tests.
  - `src/lib/settings.ts` — typed TypeScript wrapper around `get_app_setting` / `set_app_setting`.
- **Modify**
  - `src-tauri/src/metadata/mod.rs` — declare `pub mod dispatch;`.
  - `src-tauri/src/metadata/queries.rs` — sentinel rename in `next_due`, `wake_parked`; add `park_with_reason` taking `ParkReason`; remove old `park_auth`. Update existing tests for the rename.
  - `src-tauri/src/metadata/worker.rs` — major refactor: read `metadata_mode` and `tmdb_api_key` per loop, call `providers_for_mode`, walk providers via `dispatch_provider`, end-of-walk classifier with key-snapshot race guard. `dispatch_provider` initially handles only `Provider::Tmdb`.
  - `src-tauri/src/queries.rs` — add `validate()` and `default_for()`. Extend `metadata_status_counts` to include `tmdb_auth_required` and `no_provider_available` columns; update SQL to exclude both sentinels in `pending` / `failed`.
  - `src-tauri/src/commands.rs` — add generic `get_app_setting` and `set_app_setting` Tauri commands. `set_app_setting` calls `validate` then `on_setting_changed`. Delete `get_tmdb_api_key` and `set_tmdb_api_key` bespoke commands.
  - `src-tauri/src/lib.rs` — drop the two bespoke command registrations, add the two generic ones.
  - `src-tauri/src/models.rs` — extend `MetadataStatusCounts` with two new `i64` fields.
  - `src-tauri/src/db.rs` — rename `dedupe_shows_and_index` → `post_migration_fixups`; add one-shot sentinel rename SQL.
  - `src/lib/api.ts` — remove `getTmdbApiKey` / `setTmdbApiKey`; add generic `getAppSetting` / `setAppSetting`; extend `MetadataStatusCounts` interface.
  - `src/routes/settings/metadata/+page.svelte` — add `metadata_mode` Select via `settings.ts`; route TMDB key save through generic `setAppSetting`.

### fix/44 — IMDB module + dispatch

Behaviour after this PR: `imdb_only` and `prefer_imdb` fully work. `prefer_tmdb` falls back to IMDB on TMDB miss.

- **Create**
  - `src-tauri/src/metadata/imdb.rs` — suggestion API client + GraphQL client + poster downloader + unit tests.
  - `src-tauri/tests/fixtures/imdb-suggestion-movie.json` — sample suggestion response.
  - `src-tauri/tests/fixtures/imdb-suggestion-show.json` — sample suggestion response (TV).
  - `src-tauri/tests/fixtures/imdb-graphql-movie.json` — full GraphQL response (The Matrix).
  - `src-tauri/tests/fixtures/imdb-graphql-show.json` — full GraphQL response (Breaking Bad).
  - `src-tauri/tests/fixtures/imdb-graphql-edge.json` — unreleased title (null rating, null runtime).
- **Modify**
  - `src-tauri/src/metadata/mod.rs` — declare `pub mod imdb;`.
  - `src-tauri/src/metadata/matching.rs` — add `provider: Provider` to `MatchCandidate`; update 14 test sites to pass `Provider::Tmdb` in the helper.
  - `src-tauri/src/metadata/apply.rs` — add `apply_imdb_movie_details`, `apply_imdb_show_details`.
  - `src-tauri/src/metadata/dispatch.rs` — extend `providers_for_mode` so IMDB modes return real walks (drop the fix/43 degrade); add `dispatch_provider` IMDB branch.
  - `src-tauri/src/metadata/worker.rs` — add the hand-linked fast path before `providers_for_mode`. Read the new `scrape_language` setting (default `"en"`) once per loop.
  - `src-tauri/src/commands.rs` — `metadata_search` and `link_metadata` take a `provider` parameter; `link_metadata` SQL stops hardcoding `'tmdb'`.
  - `src/lib/api.ts` — update `metadataSearch` and `linkMetadata` signatures.
  - `src-tauri/Cargo.toml` — no new deps needed.

### fix/45 — match-sheet provider toggle + banners

Behaviour after this PR: users can manually search either provider regardless of mode, and the metadata settings page surfaces the auth-bad / mode-off conditions.

- **Modify**
  - `src/lib/components/MetadataMatchSheet.svelte` — two-tab UI (TMDB / IMDB); default tab follows active mode; TMDB tab disabled when no key.
  - `src/routes/settings/metadata/+page.svelte` — banners for `tmdb_auth_bad` and mode-off.

---

# PR A — fix/43-metadata-settings-infrastructure

### Task A.0: Create the branch

**Files:** none

- [ ] **Step 1: Sync master and branch**

```bash
cd /mnt/c/Programmering/Rust/rustflix
git checkout master && git pull --ff-only
git status --porcelain
```
Expected: clean tree, on master, up to date.

- [ ] **Step 2: Create branch**

```bash
git checkout -b fix/43-metadata-settings-infrastructure
```

---

### Task A.1: Define the `Provider` and `ParkReason` enums in a new dispatch module

**Files:**
- Create: `src-tauri/src/metadata/dispatch.rs`
- Modify: `src-tauri/src/metadata/mod.rs`

- [ ] **Step 1: Create the dispatch module skeleton**

Create `src-tauri/src/metadata/dispatch.rs`:

```rust
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
/// While the IMDB module is unimplemented (fix/43), `imdb_only` and
/// `prefer_imdb` degrade to `[Tmdb]` when a key is present, else park
/// as `NoProviderAvailable`. fix/44 replaces those branches with real
/// IMDB walks.
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
        // Until fix/44 lands, IMDB modes degrade to TMDB when possible.
        "imdb_only" | "prefer_imdb" => {
            if has_tmdb_key {
                Ok(vec![Tmdb])
            } else {
                Err(ParkReason::NoProviderAvailable)
            }
        }
        _ => {
            // prefer_tmdb (default). IMDB fallback arrives in fix/44.
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
    fn imdb_only_without_key_parks_in_fix43_degrade() {
        let result = providers_for_mode("imdb_only", false);
        assert_eq!(result.unwrap_err(), ParkReason::NoProviderAvailable);
    }

    #[test]
    fn imdb_only_with_key_degrades_to_tmdb_in_fix43() {
        assert_eq!(providers_for_mode("imdb_only", true).unwrap(), vec![Provider::Tmdb]);
    }

    #[test]
    fn prefer_tmdb_with_key_returns_tmdb() {
        assert_eq!(providers_for_mode("prefer_tmdb", true).unwrap(), vec![Provider::Tmdb]);
    }

    #[test]
    fn prefer_tmdb_without_key_parks_in_fix43() {
        let result = providers_for_mode("prefer_tmdb", false);
        assert_eq!(result.unwrap_err(), ParkReason::NoProviderAvailable);
    }

    #[test]
    fn prefer_imdb_with_key_returns_tmdb_in_fix43() {
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
```

- [ ] **Step 2: Wire the module in `metadata/mod.rs`**

Read `src-tauri/src/metadata/mod.rs`. It currently declares:

```rust
pub mod apply;
pub mod matching;
pub mod queries;
pub mod tmdb;
pub mod worker;
```

Add `pub mod dispatch;` so the file becomes:

```rust
//! Metadata sync subsystem. See
//! docs/superpowers/specs/2026-05-25-imdb-fallback-provider-design.md.

pub mod apply;
pub mod dispatch;
pub mod matching;
pub mod queries;
pub mod tmdb;
pub mod worker;
```

- [ ] **Step 3: Run the dispatch tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata::dispatch 2>&1 | tail -20
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 10 passing tests in `metadata::dispatch::tests::*`.

- [ ] **Step 4: Commit**

```bash
cd /mnt/c/Programmering/Rust/rustflix
git add src-tauri/src/metadata/dispatch.rs src-tauri/src/metadata/mod.rs
git commit -m "feat(metadata): Provider enum + providers_for_mode + ParkReason

Lays the routing foundation for multi-provider metadata sync. The
IMDB module doesn't exist yet (fix/44), so IMDB-requiring modes
degrade to [Tmdb] when a TMDB key is present, otherwise park as
NoProviderAvailable. 10 unit tests cover the full mode x key matrix."
```

---

### Task A.2: Rename `auth_required` sentinel to `tmdb_auth_required` and add `park_with_reason`

**Files:**
- Modify: `src-tauri/src/metadata/queries.rs`

- [ ] **Step 1: Read the current state of the queue helpers**

```bash
grep -n "auth_required\|park_auth\|wake_parked\|next_due\|fn park" src-tauri/src/metadata/queries.rs
```

Expected output: locations of `next_due` (filter clause), `park_auth`, `wake_parked`, and the `tests` module that uses these.

- [ ] **Step 2: Rewrite `next_due` to exclude both park sentinels**

In `src-tauri/src/metadata/queries.rs`, find the `pub async fn next_due` and replace its SQL with:

```rust
pub async fn next_due(pool: &SqlitePool) -> AppResult<Option<MetadataJob>> {
    let job: Option<MetadataJob> = sqlx::query_as(
        "SELECT kind, media_id, attempts, last_error, next_attempt_at
         FROM metadata_jobs
         WHERE COALESCE(last_error, '') NOT IN ('tmdb_auth_required', 'no_provider_available')
           AND attempts < 8
           AND next_attempt_at <= strftime('%s','now')
         ORDER BY next_attempt_at ASC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    Ok(job)
}
```

- [ ] **Step 3: Replace `park_auth` with `park_with_reason`**

Find the existing `pub async fn park_auth` and replace it with:

```rust
use crate::metadata::dispatch::ParkReason;

pub async fn park_with_reason(
    pool: &SqlitePool,
    kind: &str,
    media_id: i64,
    reason: ParkReason,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET last_error = ?3
         WHERE kind = ?1 AND media_id = ?2",
    )
    .bind(kind)
    .bind(media_id)
    .bind(reason.sentinel())
    .execute(pool)
    .await?;
    Ok(())
}
```

- [ ] **Step 4: Update `wake_parked` to clear both sentinels**

Replace the body of `pub async fn wake_parked`:

```rust
pub async fn wake_parked(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET
             last_error = NULL,
             next_attempt_at = strftime('%s','now')
         WHERE last_error IN ('tmdb_auth_required', 'no_provider_available')",
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

- [ ] **Step 5: Update existing tests for the new function name and sentinel**

Find the existing test that calls `park_auth` and rename it to use `park_with_reason(pool, "show", show_id, ParkReason::TmdbAuthRequired)`. The assertion that checks `last_error == "auth_required"` becomes `last_error == "tmdb_auth_required"`.

Concretely, find the test named `park_auth_excludes_job_from_next_due` and replace with:

```rust
#[tokio::test]
async fn park_with_tmdb_auth_excludes_job_from_next_due() {
    let pool = fresh_pool().await;
    let show_id = seed_show(&pool).await;
    enqueue(&pool, "show", show_id).await.unwrap();

    park_with_reason(&pool, "show", show_id, ParkReason::TmdbAuthRequired)
        .await
        .unwrap();

    let job = next_due(&pool).await.unwrap();
    assert!(job.is_none(), "parked job should be excluded");
}
```

Add a sibling test for the other sentinel:

```rust
#[tokio::test]
async fn park_with_no_provider_excludes_job_from_next_due() {
    let pool = fresh_pool().await;
    let show_id = seed_show(&pool).await;
    enqueue(&pool, "show", show_id).await.unwrap();

    park_with_reason(&pool, "show", show_id, ParkReason::NoProviderAvailable)
        .await
        .unwrap();

    let job = next_due(&pool).await.unwrap();
    assert!(job.is_none(), "parked job should be excluded");
}
```

The existing `wake_parked_clears_sentinel` test still works but rename it for clarity to `wake_parked_clears_either_sentinel` and call `park_with_reason` instead of `park_auth`.

- [ ] **Step 6: Run the queue tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata::queries 2>&1 | tail -20
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: all queue tests pass, including the two new park tests.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/metadata/queries.rs
git commit -m "refactor(metadata): rename auth_required sentinel and add park_with_reason

next_due and wake_parked now handle both park sentinels:
tmdb_auth_required (the renamed former auth_required) and the new
no_provider_available. park_auth is replaced by the typed
park_with_reason. Tests cover both."
```

---

### Task A.3: Rename `dedupe_shows_and_index` and add the one-shot sentinel-rename fixup

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Read the current function**

```bash
grep -n "dedupe_shows_and_index" src-tauri/src/db.rs
```

The function is called from `open()` after migrations.

- [ ] **Step 2: Rename the function and add the sentinel-rename fixup**

In `src-tauri/src/db.rs`, find `async fn dedupe_shows_and_index` and rename it to `post_migration_fixups`. Update the only caller in `pub async fn open`. At the end of the function (just before the final `Ok(())`), add:

```rust
    // One-shot: rename the legacy auth_required sentinel that PRs
    // before the IMDB-fallback work emitted. Idempotent.
    sqlx::query(
        "UPDATE metadata_jobs SET last_error = 'tmdb_auth_required'
         WHERE last_error = 'auth_required'",
    )
    .execute(pool)
    .await?;
```

This runs every startup but only touches rows whose `last_error` matches the legacy value — zero impact after the first run.

- [ ] **Step 3: Update the function's doc comment**

The doc comment for the renamed function should describe its expanded role. Replace whatever's there with:

```rust
/// Runs once after every migration pass. Handles dedup of pre-0002 shows
/// (the legacy fingerprint backfill + duplicate merge), ensures the
/// unique index exists, and renames any legacy `auth_required` sentinel
/// in `metadata_jobs` to `tmdb_auth_required`. Idempotent — re-runs
/// every startup but only writes when rows actually need fixing.
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -8
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "refactor(db): rename dedupe_shows_and_index to post_migration_fixups

The function grew past its original name (now also handles sentinel
rename for the IMDB-fallback work). Doc comment updated. One-shot
UPDATE renames any legacy auth_required rows to tmdb_auth_required;
idempotent across startups."
```

---

### Task A.4: Extend `MetadataStatusCounts` and update the count query for both sentinels

**Files:**
- Modify: `src-tauri/src/models.rs`
- Modify: `src-tauri/src/queries.rs`

- [ ] **Step 1: Extend the struct**

In `src-tauri/src/models.rs`, find `pub struct MetadataStatusCounts`. Today it has `pending`, `failed`, `auth_required`, `dead_letter`, `needs_review`. Replace `auth_required` with the renamed field and add `no_provider_available`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MetadataStatusCounts {
    pub pending: i64,
    pub failed: i64,
    pub tmdb_auth_required: i64,
    pub no_provider_available: i64,
    pub dead_letter: i64,
    pub needs_review: i64,
}
```

- [ ] **Step 2: Update the count SQL**

In `src-tauri/src/queries.rs`, find `pub async fn metadata_status_counts`. Replace its body with:

```rust
pub async fn metadata_status_counts(
    pool: &SqlitePool,
) -> AppResult<crate::models::MetadataStatusCounts> {
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs
         WHERE attempts = 0
           AND COALESCE(last_error, '') NOT IN ('tmdb_auth_required', 'no_provider_available')",
    )
    .fetch_one(pool)
    .await?;

    let failed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs
         WHERE attempts > 0 AND attempts < 8
           AND COALESCE(last_error, '') NOT IN ('tmdb_auth_required', 'no_provider_available')",
    )
    .fetch_one(pool)
    .await?;

    let tmdb_auth_required: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE last_error = 'tmdb_auth_required'",
    )
    .fetch_one(pool)
    .await?;

    let no_provider_available: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE last_error = 'no_provider_available'",
    )
    .fetch_one(pool)
    .await?;

    let dead_letter: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE attempts >= 8",
    )
    .fetch_one(pool)
    .await?;

    let needs_review: i64 = sqlx::query_scalar(
        "SELECT
             (SELECT COUNT(*) FROM shows
                WHERE provider IS NULL
                  AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                                  WHERE j.kind = 'show' AND j.media_id = shows.id))
           + (SELECT COUNT(*) FROM movies
                WHERE provider IS NULL
                  AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                                  WHERE j.kind = 'movie' AND j.media_id = movies.id))",
    )
    .fetch_one(pool)
    .await?;

    Ok(crate::models::MetadataStatusCounts {
        pending,
        failed,
        tmdb_auth_required,
        no_provider_available,
        dead_letter,
        needs_review,
    })
}
```

- [ ] **Step 3: Verify compile**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -8
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models.rs src-tauri/src/queries.rs
git commit -m "feat(metadata): split auth_required counts and add no_provider_available

MetadataStatusCounts now has tmdb_auth_required (renamed from
auth_required) and no_provider_available, plus pending and failed
that exclude both sentinels in their SQL. The frontend can render
the breakdown so a tmdb_only-no-key state is distinguishable from
a real key-broken state."
```

---

### Task A.5: Add Rust-side `validate()` and `default_for()` helpers

**Files:**
- Modify: `src-tauri/src/queries.rs`

- [ ] **Step 1: Find a home and add the helpers**

In `src-tauri/src/queries.rs`, near the bottom (after `delete_app_setting`), add:

```rust
/// Server-side validator keyed on setting name. Enum-shaped settings
/// reject unknown values; free-form keys pass through. Defense in depth
/// against frontend bugs and hand-edited DB rows.
pub fn validate(key: &str, value: Option<&str>) -> AppResult<()> {
    match key {
        "metadata_mode" => {
            let allowed = ["off", "tmdb_only", "imdb_only", "prefer_tmdb", "prefer_imdb"];
            match value {
                Some(v) if allowed.contains(&v) => Ok(()),
                Some(other) => Err(AppError::Other(format!(
                    "metadata_mode: invalid value '{other}'"
                ))),
                None => Ok(()),
            }
        }
        "scrape_language" | "ui_language" | "theme" | "tmdb_api_key" | "tmdb_auth_bad" => Ok(()),
        // Unknown keys allowed (forward compat with future settings).
        _ => Ok(()),
    }
}

/// Canonical default value for a known setting, as a string. Returned by
/// `get_app_setting` callers when the row is missing. The TS wrapper has
/// the parsed-type defaults; this is a parallel string version for the
/// Rust read path.
pub fn default_for(key: &str) -> Option<&'static str> {
    match key {
        "metadata_mode" => Some("prefer_tmdb"),
        "scrape_language" => Some("en"),
        "ui_language" => Some("en"),
        "theme" => Some("system"),
        _ => None,
    }
}

#[cfg(test)]
mod settings_tests {
    use super::*;

    #[test]
    fn validate_accepts_known_mode_values() {
        for value in ["off", "tmdb_only", "imdb_only", "prefer_tmdb", "prefer_imdb"] {
            assert!(validate("metadata_mode", Some(value)).is_ok(), "{value}");
        }
    }

    #[test]
    fn validate_rejects_unknown_mode_value() {
        assert!(validate("metadata_mode", Some("perfer_tmdb")).is_err());
        assert!(validate("metadata_mode", Some("")).is_err());
    }

    #[test]
    fn validate_accepts_null_for_any_key() {
        assert!(validate("metadata_mode", None).is_ok());
        assert!(validate("tmdb_api_key", None).is_ok());
    }

    #[test]
    fn validate_passes_through_free_form_keys() {
        assert!(validate("scrape_language", Some("en-US")).is_ok());
        assert!(validate("tmdb_api_key", Some("abc123")).is_ok());
        assert!(validate("future_setting", Some("anything")).is_ok());
    }

    #[test]
    fn default_for_known_keys() {
        assert_eq!(default_for("metadata_mode"), Some("prefer_tmdb"));
        assert_eq!(default_for("scrape_language"), Some("en"));
        assert_eq!(default_for("theme"), Some("system"));
    }

    #[test]
    fn default_for_unknown_keys_is_none() {
        assert_eq!(default_for("tmdb_api_key"), None);
        assert_eq!(default_for("nonexistent"), None);
    }
}
```

- [ ] **Step 2: Run the new tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib settings_tests 2>&1 | tail -15
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 6 passing tests.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/queries.rs
git commit -m "feat(settings): add validate() and default_for() helpers

validate() rejects unknown metadata_mode values server-side regardless
of what the frontend wrapper enforces. default_for() returns the
canonical default for each known setting key so the worker can fall
back consistently when a row is missing. 6 unit tests cover both."
```

---

### Task A.6: Refactor `set_app_setting` into a generic Tauri command with `on_setting_changed`

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add generic `get_app_setting` and `set_app_setting` Tauri commands**

In `src-tauri/src/commands.rs`, find the existing `get_tmdb_api_key` / `set_tmdb_api_key` block. Replace **both** commands (and any helpers like `wake_worker`) with:

```rust
#[tauri::command]
pub async fn get_app_setting(db: State<'_, Db>, key: String) -> AppResult<Option<String>> {
    queries::get_app_setting(&db, &key).await
}

#[tauri::command]
pub async fn set_app_setting(
    app: AppHandle,
    db: State<'_, Db>,
    key: String,
    value: Option<String>,
) -> AppResult<()> {
    queries::validate(&key, value.as_deref())?;

    let previous = queries::get_app_setting(&db, &key).await?;
    match value.as_deref() {
        Some(v) => queries::set_app_setting(&db, &key, v).await?,
        None => queries::delete_app_setting(&db, &key).await?,
    }

    on_setting_changed(&app, &db, &key, previous.as_deref(), value.as_deref()).await?;
    Ok(())
}

async fn on_setting_changed(
    app: &AppHandle,
    db: &Db,
    key: &str,
    previous: Option<&str>,
    next: Option<&str>,
) -> AppResult<()> {
    match key {
        "tmdb_api_key" => {
            if next.is_some() {
                queries::delete_app_setting(db, "tmdb_auth_bad").await?;
                crate::metadata::queries::wake_parked(db).await?;
            }
            wake_worker(app);
        }
        "metadata_mode" => {
            if previous != next {
                crate::metadata::queries::wake_parked(db).await?;
            }
            if matches!(next, Some("off") | Some("imdb_only")) {
                queries::delete_app_setting(db, "tmdb_auth_bad").await?;
            }
            wake_worker(app);
        }
        "scrape_language" => wake_worker(app),
        _ => {}
    }
    Ok(())
}

fn wake_worker(app: &AppHandle) {
    if let Some(notify) = app.try_state::<std::sync::Arc<tokio::sync::Notify>>() {
        notify.notify_one();
    }
}
```

- [ ] **Step 2: Update consumers in `commands.rs`**

The `set_app_setting` call inside `scan_libraries` (or anywhere else that calls `queries::set_app_setting` directly without going through `on_setting_changed`) needs to keep working — those internal writes don't need side-effect dispatch and continue to call the `queries::` helper directly. Verify no `Tauri` command code paths still call the deleted `set_tmdb_api_key`.

```bash
grep -rn "set_tmdb_api_key\|get_tmdb_api_key" src-tauri/src/
```

Expected: no matches (if there are, delete the lines).

- [ ] **Step 3: Update `lib.rs` handler list**

In `src-tauri/src/lib.rs`, find the `invoke_handler` macro. Remove `commands::get_tmdb_api_key` and `commands::set_tmdb_api_key`. Add `commands::get_app_setting` and `commands::set_app_setting`. The new block should look like (showing only the changed lines):

```rust
            commands::get_app_setting,
            commands::set_app_setting,
            commands::metadata_status_counts,
            // ... rest unchanged
```

- [ ] **Step 4: Verify the Rust side compiles**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -10
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(settings): generic set_app_setting with validate + on_setting_changed

Replaces bespoke get_tmdb_api_key / set_tmdb_api_key Tauri commands
with generic get_app_setting / set_app_setting. The new commands
validate values server-side and fan out side-effects through
on_setting_changed (wake_parked on key/mode change, clear
tmdb_auth_bad when no TMDB needed, notify_worker on everything that
matters). All known settings now share one entry point."
```

---

### Task A.7: Refactor worker to use `providers_for_mode` and end-of-walk classification

**Files:**
- Modify: `src-tauri/src/metadata/worker.rs`

This is the biggest backend task. Skim `src-tauri/src/metadata/worker.rs` first to understand the current shape — `run`, `run_job`, `fetch_movie`, `fetch_show`, `handle_failure`, `unix_now`.

- [ ] **Step 1: Add the `dispatch_provider` helper for TMDB**

At the top of `src-tauri/src/metadata/worker.rs`, add imports:

```rust
use crate::metadata::dispatch::{providers_for_mode, ParkReason, Provider};
```

Below the existing `MAX_ATTEMPTS` / `PACING_MS` constants, add:

```rust
/// Outcome of attempting one provider against one job.
enum Outcome {
    Matched,
    NoMatch,
}
```

Then **rename** the existing two-argument shape of `fetch_movie` / `fetch_show` to take the TMDB API key as a `&str` argument (it already does) and produce a single dispatcher. Below the existing helpers, add:

```rust
async fn dispatch_provider(
    provider: Provider,
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    match provider {
        Provider::Tmdb => dispatch_tmdb(pool, http, app, api_key, job).await,
        Provider::Imdb => {
            // IMDB module lands in fix/44. Reaching here means
            // providers_for_mode degraded incorrectly; treat as a real
            // error so the job goes through backoff and we notice.
            Err(AppError::Other(
                "imdb provider not implemented in fix/43".to_string(),
            ))
        }
    }
}

/// Replaces the old `fetch_movie` / `fetch_show` pair with a single
/// per-kind dispatcher. HTTP stays outside the tx; the tx wraps only
/// the lock re-check + apply + delete_in_tx. The poster downloads
/// post-tx, best-effort.
async fn dispatch_tmdb(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let download = match job.kind.as_str() {
        "movie" => dispatch_tmdb_movie(pool, http, api_key, job.media_id).await?,
        "show" => dispatch_tmdb_show(pool, http, api_key, job.media_id).await?,
        other => {
            return Err(AppError::Other(format!("unknown job kind: {other}")));
        }
    };

    let outcome = match download {
        Some((poster_path, filename)) => {
            let dest = posters_dir.join(filename);
            if let Err(error) = tmdb::download_poster(http, &poster_path, &dest).await {
                eprintln!("tmdb poster download failed for {dest:?}: {error}");
            }
            Outcome::Matched
        }
        None => Outcome::Matched,  // matched but no poster URL (rare)
    };

    Ok(outcome)
}
```

**Important:** the existing per-kind `fetch_movie` / `fetch_show` functions in this file already do the right thing — re-read row + http + tx-wrap. Rename them to `dispatch_tmdb_movie` / `dispatch_tmdb_show` and add to each function: return `Ok(None)` to mean "matched but no poster" or `Ok(Some((poster_path, filename)))` to mean "matched with poster to download." On `Outcome::NoMatch`, change to return `AppResult<Outcome>` directly — see Step 2.

Actually, simpler: keep the current functions returning `AppResult<Option<(String, String)>>` (None = no match OR matched-no-poster). Add a separate signal for "really no match" — see next step.

- [ ] **Step 2: Make `dispatch_provider` distinguish no-match from matched-no-poster**

Change `dispatch_tmdb_movie` and `dispatch_tmdb_show` to return `AppResult<Option<(String, String)>>` where the `Option` is the poster download — but we also need to signal whether a match happened at all. Refactor to return `AppResult<MatchOutcome>`:

```rust
/// A pending poster download, queued post-tx as best-effort.
/// `size` is only consulted for IMDB; TMDB ignores it.
pub struct PosterDownload {
    pub url: String,
    pub filename: String,
    pub size: Option<crate::metadata::imdb::PosterSize>,
}

enum MatchOutcome {
    NoMatch,
    Matched { poster: Option<PosterDownload> },
}
```

Note: `PosterSize` doesn't exist yet (lands in fix/44 B.3). For fix/43, change the field type to `Option<()>` — really a placeholder — and use just `url + filename`. Or, more practically, define `PosterDownload` without the `size` field in fix/43 and add the field in fix/44 B.6 as a backwards-compatible change. **Recommended: ship fix/43 with `size: Option<&'static str>` initially set to `None` everywhere; fix/44 introduces `PosterSize`, then this field gets retyped to `Option<PosterSize>` in B.6.**

Update the two TMDB-per-kind functions to return `MatchOutcome::NoMatch` when `pick_confident_match` returned None or the row was locked / missing, and `MatchOutcome::Matched { poster }` when apply succeeded.

Then `dispatch_tmdb` becomes:

```rust
async fn dispatch_tmdb(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let result = match job.kind.as_str() {
        "movie" => dispatch_tmdb_movie(pool, http, api_key, job.media_id).await?,
        "show" => dispatch_tmdb_show(pool, http, api_key, job.media_id).await?,
        other => return Err(AppError::Other(format!("unknown job kind: {other}"))),
    };

    match result {
        MatchOutcome::NoMatch => Ok(Outcome::NoMatch),
        MatchOutcome::Matched { poster } => {
            if let Some((poster_path, filename)) = poster {
                let dest = posters_dir.join(filename);
                if let Err(error) = tmdb::download_poster(http, &poster_path, &dest).await {
                    eprintln!("tmdb poster download failed for {dest:?}: {error}");
                }
            }
            Ok(Outcome::Matched)
        }
    }
}
```

Important: the existing `fetch_movie` / `fetch_show` already separate "no match" from "matched, no poster"; pull that distinction out into the new `MatchOutcome` enum.

- [ ] **Step 3: Rewrite the `run` loop**

Replace the existing `pub async fn run` body with:

```rust
async fn run(
    pool: SqlitePool,
    http: reqwest::Client,
    app: AppHandle,
    notify: Arc<Notify>,
) -> AppResult<()> {
    loop {
        // Per-iteration setting reads. No worker-side cache by design.
        let mode = app_queries::get_app_setting(&pool, "metadata_mode")
            .await?
            .unwrap_or_else(|| "prefer_tmdb".to_string());

        let api_key = app_queries::get_app_setting(&pool, "tmdb_api_key").await?;
        let key_at_job_start = api_key.clone();

        let Some(job) = queries::next_due(&pool).await? else {
            notify.notified().await;
            continue;
        };

        if job.attempts >= MAX_ATTEMPTS {
            // Dead-letter: leave it in the queue but skip it. Sleep
            // briefly so we don't spin-loop on dead rows.
            sleep(Duration::from_secs(60)).await;
            continue;
        }

        let now = unix_now();
        if job.next_attempt_at > now {
            let wait = Duration::from_secs((job.next_attempt_at - now) as u64);
            tokio::select! {
                _ = sleep(wait) => {},
                _ = notify.notified() => {},
            }
            continue;
        }

        let providers = match providers_for_mode(&mode, api_key.is_some()) {
            Ok(list) => list,
            Err(reason) => {
                queries::park_with_reason(&pool, &job.kind, job.media_id, reason).await?;
                continue;
            }
        };

        if providers.is_empty() {
            // mode == "off" — delete the job so it doesn't pile up.
            let mut tx = pool.begin().await?;
            queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
            tx.commit().await?;
            continue;
        }

        let mut last_err: Option<AppError> = None;
        let mut saw_tmdb_auth = false;
        let mut matched = false;

        for (index, provider) in providers.iter().enumerate() {
            if index > 0 {
                sleep(Duration::from_millis(PACING_MS)).await;
            }
            // We only have an api_key for TMDB; safe to default to "" for
            // IMDB since dispatch_imdb (fix/44) doesn't read it.
            let key_for_call = api_key.as_deref().unwrap_or("");
            match dispatch_provider(*provider, &pool, &http, &app, key_for_call, &job).await {
                Ok(Outcome::Matched) => {
                    matched = true;
                    break;
                }
                Ok(Outcome::NoMatch) => continue,
                Err(error) => {
                    if error.to_string().starts_with("auth_required") ||
                       error.to_string().starts_with("tmdb_auth_required") {
                        saw_tmdb_auth = true;
                        // Race guard: only set the flag if the key on disk
                        // is still the one we used at job start.
                        let key_now =
                            app_queries::get_app_setting(&pool, "tmdb_api_key").await?;
                        if key_now == key_at_job_start {
                            app_queries::set_app_setting(&pool, "tmdb_auth_bad", "1").await?;
                        }
                    }
                    last_err = Some(error);
                    continue;
                }
            }
        }

        if matched {
            // dispatch_tmdb already committed apply + delete_in_tx in one tx.
        } else if saw_tmdb_auth {
            queries::park_with_reason(
                &pool, &job.kind, job.media_id, ParkReason::TmdbAuthRequired,
            ).await?;
        } else if let Some(error) = last_err {
            queries::record_failure(&pool, &job.kind, job.media_id, &error.to_string()).await?;
        } else {
            let mut tx = pool.begin().await?;
            queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
            tx.commit().await?;
        }

        sleep(Duration::from_millis(PACING_MS)).await;
    }
}
```

- [ ] **Step 4: Update the TMDB-per-kind functions to merge apply + delete into one tx**

In the existing `dispatch_tmdb_movie` (renamed from `fetch_movie`), the current code commits the apply tx and then opens a separate tx for `delete_in_tx`. Merge them. Replace the body with:

```rust
async fn dispatch_tmdb_movie(
    pool: &SqlitePool,
    http: &reqwest::Client,
    api_key: &str,
    movie_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        // Row was deleted between enqueue and us picking it up.
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = tmdb::search_movie(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        // No confident match — delete the job (caller does this for us via
        // the dispatcher's NoMatch path, but for symmetry).
        let mut tx = pool.begin().await?;
        queries::delete_in_tx(&mut *tx, "movie", movie_id).await?;
        tx.commit().await?;
        return Ok(MatchOutcome::NoMatch);
    };

    let details = tmdb::fetch_movie_details(http, api_key, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    // Re-check the lock inside the tx — the user may have edited the
    // title while we were over the wire.
    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(MatchOutcome::NoMatch);
    }

    let download_ext = apply::apply_movie_details(&mut *tx, movie_id, &details).await?;

    // Merged delete: apply + delete go in one tx so a concurrent
    // re-enqueue between the two writes can't be silently dropped.
    queries::delete_in_tx(&mut *tx, "movie", movie_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: match (download_ext, details.poster_path) {
            (Some(extension), Some(poster_path)) => {
                Some((poster_path, format!("movie-{movie_id}.{extension}")))
            }
            _ => None,
        },
    })
}
```

Repeat the same shape for `dispatch_tmdb_show` (operating on `shows` table, `apply::apply_show_details`, file prefix `show-`).

- [ ] **Step 5: Remove the now-unused old delete_in_tx call**

The old `run_job` function called `queries::delete_in_tx` after `dispatch_tmdb`. With the apply+delete merge, that's gone. Remove any dead code paths.

- [ ] **Step 6: Verify it compiles**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -20
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean. If there are unused-import warnings, remove them.

- [ ] **Step 7: Run all metadata tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata 2>&1 | tail -25
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: all existing metadata tests still pass.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/metadata/worker.rs
git commit -m "feat(metadata): worker mode dispatch + end-of-walk classification

Worker now reads metadata_mode per loop iteration and walks providers
in order via providers_for_mode. End-of-walk classifier:
- matched: dispatch_tmdb committed apply+delete in one tx
- saw_tmdb_auth: park with TmdbAuthRequired (don't backoff a broken key)
- has-error: record_failure with backoff
- no-error: delete job, row surfaces in Needs review

Apply + delete merged into one tx closes the concurrent-scanner
re-enqueue race. HTTP stays outside the tx. Key-snapshot race guard
prevents the worker from re-flagging tmdb_auth_bad after the user
already pasted a new key. dispatch_provider for IMDB stubs out with
an error until fix/44 lands."
```

---

### Task A.8: Frontend — typed settings wrapper

**Files:**
- Create: `src/lib/settings.ts`

- [ ] **Step 1: Create the wrapper**

Create `src/lib/settings.ts` with:

```ts
import { invoke } from '@tauri-apps/api/core';

export type MetadataMode =
  | 'off'
  | 'tmdb_only'
  | 'imdb_only'
  | 'prefer_tmdb'
  | 'prefer_imdb';

type SettingDef<T> = {
  default: T;
  parse: (raw: string | null) => T;
  encode: (value: T) => string | null;
};

export const SETTINGS = {
  tmdb_api_key: {
    default: null as string | null,
    parse: (raw: string | null): string | null => raw,
    encode: (value: string | null): string | null => value,
  } satisfies SettingDef<string | null>,

  metadata_mode: {
    default: 'prefer_tmdb' as MetadataMode,
    parse: (raw: string | null): MetadataMode => {
      const valid: readonly MetadataMode[] = [
        'off',
        'tmdb_only',
        'imdb_only',
        'prefer_tmdb',
        'prefer_imdb',
      ];
      if (raw === null || raw === undefined) {
        return 'prefer_tmdb';
      }
      if (!(valid as readonly string[]).includes(raw)) {
        console.warn(`metadata_mode: invalid value "${raw}", falling back to default`);
        return 'prefer_tmdb';
      }
      return raw as MetadataMode;
    },
    encode: (value: MetadataMode): string => value,
  } satisfies SettingDef<MetadataMode>,

  scrape_language: {
    default: 'en',
    parse: (raw: string | null): string => raw ?? 'en',
    encode: (value: string): string => value,
  } satisfies SettingDef<string>,
} as const;

export type SettingKey = keyof typeof SETTINGS;
export type SettingValue<K extends SettingKey> = (typeof SETTINGS)[K]['default'];

export async function getSetting<K extends SettingKey>(key: K): Promise<SettingValue<K>> {
  const raw = await invoke<string | null>('get_app_setting', { key });
  return SETTINGS[key].parse(raw) as SettingValue<K>;
}

export async function setSetting<K extends SettingKey>(
  key: K,
  value: SettingValue<K>,
): Promise<void> {
  const encoded = SETTINGS[key].encode(value as never);
  await invoke<void>('set_app_setting', { key, value: encoded });
}
```

- [ ] **Step 2: Type-check**

```bash
cd /mnt/c/Programmering/Rust/rustflix
CI=true pnpm check 2>&1 | tail -10
```
Expected: zero errors (or just the pre-existing one, which should already be gone after the earlier style-cleanup PR).

- [ ] **Step 3: Commit**

```bash
git add src/lib/settings.ts
git commit -m "feat(settings): typed wrapper around get/set_app_setting

src/lib/settings.ts declares known settings with default values, parse
functions, and encode functions. getSetting<K>(key) returns the typed
default value type; setSetting<K>(key, value) encodes and writes. Loud
warning on invalid metadata_mode raw rather than silent recovery."
```

---

### Task A.9: Update `api.ts` types and remove the old TMDB-key calls

**Files:**
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Extend the `MetadataStatusCounts` type**

In `src/lib/api.ts`, find `export interface MetadataStatusCounts`. Replace with:

```ts
export interface MetadataStatusCounts {
  pending: number;
  failed: number;
  tmdb_auth_required: number;
  no_provider_available: number;
  dead_letter: number;
  needs_review: number;
}
```

- [ ] **Step 2: Remove `getTmdbApiKey` and `setTmdbApiKey` methods from the api object**

Find the `api` object literal. Delete the two methods. The Metadata settings page will use `getSetting('tmdb_api_key')` from the new wrapper instead.

```bash
grep -n "getTmdbApiKey\|setTmdbApiKey" src/lib/api.ts
```

Expected after edit: no matches.

- [ ] **Step 3: Type-check**

```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: zero errors. If `/settings/metadata/+page.svelte` still references the deleted methods, that's the next task.

- [ ] **Step 4: Commit (deferred until the page is updated in A.10)**

Don't commit yet; A.10 lands the page changes together with the api.ts changes.

---

### Task A.10: Update the Metadata settings page with mode picker and switch to `settings.ts`

**Files:**
- Modify: `src/routes/settings/metadata/+page.svelte`

- [ ] **Step 1: Read the current page**

Skim `src/routes/settings/metadata/+page.svelte` to understand what's there: TMDB key input, save button, status counts panel, TMDB attribution.

- [ ] **Step 2: Rewrite the script block**

Replace the `<script lang="ts">` block at the top with:

```svelte
<script lang="ts">
  import { api, type MetadataStatusCounts } from '$lib/api';
  import { getSetting, setSetting, type MetadataMode } from '$lib/settings';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';
  import * as Select from '$lib/components/ui/select';

  let mode = $state<MetadataMode>('prefer_tmdb');
  let keyDraft = $state('');
  let savedKey = $state<string | null>(null);
  let saving = $state(false);
  let savingMode = $state(false);
  let counts = $state<MetadataStatusCounts | null>(null);
  let error = $state<string | null>(null);

  const MODE_LABELS: Record<MetadataMode, string> = {
    off: 'Off (no metadata sync)',
    tmdb_only: 'TMDB only',
    imdb_only: 'IMDB only',
    prefer_tmdb: 'Prefer TMDB, fall back to IMDB',
    prefer_imdb: 'Prefer IMDB, fall back to TMDB',
  };

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      const [keyResult, modeResult, countsResult] = await Promise.all([
        getSetting('tmdb_api_key'),
        getSetting('metadata_mode'),
        api.metadataStatusCounts(),
      ]);
      savedKey = keyResult;
      mode = modeResult;
      counts = countsResult;
      keyDraft = savedKey ?? '';
    } catch (caught) {
      error = String(caught);
    }
  }

  async function saveKey() {
    saving = true;
    error = null;
    try {
      const trimmed = keyDraft.trim();
      await setSetting('tmdb_api_key', trimmed.length === 0 ? null : trimmed);
      savedKey = trimmed.length === 0 ? null : trimmed;
    } catch (caught) {
      error = String(caught);
    } finally {
      saving = false;
    }
  }

  async function saveMode(next: MetadataMode) {
    savingMode = true;
    error = null;
    try {
      await setSetting('metadata_mode', next);
      mode = next;
    } catch (caught) {
      error = String(caught);
    } finally {
      savingMode = false;
    }
  }
</script>
```

- [ ] **Step 3: Rewrite the template**

Replace the entire markup below the script with:

```svelte
<div class="mx-auto max-w-3xl px-6 py-8">
  <header class="mb-6">
    <h1 class="text-3xl font-bold tracking-tight">Metadata</h1>
    <p class="text-sm text-muted-foreground">
      Rustflix can fetch posters, overviews, genres, ratings, and cast from TMDB
      and IMDB.
    </p>
  </header>

  {#if error}
    <div
      class="mb-6 rounded-md border border-destructive/30 bg-destructive/10 px-4 py-3 text-sm text-destructive-foreground"
    >
      {error}
    </div>
  {/if}

  <div class="flex flex-col gap-6">
    <Card>
      <CardHeader>
        <CardTitle>Sync mode</CardTitle>
        <CardDescription>
          Pick which providers to use and in what order.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Select.Root
          type="single"
          value={mode}
          onValueChange={(next) => {
            if (next) {
              void saveMode(next as MetadataMode);
            }
          }}
        >
          <Select.Trigger class="w-full sm:w-[420px]" aria-label="Metadata sync mode">
            {MODE_LABELS[mode]}
          </Select.Trigger>
          <Select.Content>
            <Select.Item value="off" label={MODE_LABELS.off}>
              {MODE_LABELS.off}
            </Select.Item>
            <Select.Item value="tmdb_only" label={MODE_LABELS.tmdb_only}>
              {MODE_LABELS.tmdb_only}
            </Select.Item>
            <Select.Item value="imdb_only" label={MODE_LABELS.imdb_only}>
              {MODE_LABELS.imdb_only}
            </Select.Item>
            <Select.Item value="prefer_tmdb" label={MODE_LABELS.prefer_tmdb}>
              {MODE_LABELS.prefer_tmdb}
            </Select.Item>
            <Select.Item value="prefer_imdb" label={MODE_LABELS.prefer_imdb}>
              {MODE_LABELS.prefer_imdb}
            </Select.Item>
          </Select.Content>
        </Select.Root>
        {#if savingMode}
          <p class="mt-2 text-xs text-muted-foreground">Saving…</p>
        {/if}
      </CardContent>
    </Card>

    {#if mode !== 'off'}
      <Card>
        <CardHeader>
          <CardTitle>TMDB API key</CardTitle>
          <CardDescription>
            Sign up at <a class="underline" href="https://www.themoviedb.org/settings/api">themoviedb.org</a>
            and paste your v3 API key here.
            {#if mode === 'imdb_only'}
              <span class="block mt-1 text-xs">
                Not used while IMDB-only mode is active.
              </span>
            {/if}
          </CardDescription>
        </CardHeader>
        <CardContent class="flex flex-col gap-3">
          <Input
            bind:value={keyDraft}
            placeholder="Paste your TMDB v3 API key"
            type="password"
            disabled={mode === 'imdb_only'}
          />
          <div class="flex items-center gap-3">
            <Button onclick={saveKey} disabled={saving || mode === 'imdb_only'}>
              {saving ? 'Saving…' : savedKey ? 'Update key' : 'Save key'}
            </Button>
            {#if savedKey}
              <span class="text-xs text-muted-foreground">A key is currently stored.</span>
            {/if}
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Sync status</CardTitle>
        </CardHeader>
        <CardContent>
          {#if counts}
            <ul class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-3">
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Pending</div>
                <div class="text-lg font-semibold">{counts.pending}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Failed</div>
                <div class="text-lg font-semibold">{counts.failed}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">TMDB auth needed</div>
                <div class="text-lg font-semibold">{counts.tmdb_auth_required}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">No provider</div>
                <div class="text-lg font-semibold">{counts.no_provider_available}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Dead-letter</div>
                <div class="text-lg font-semibold">{counts.dead_letter}</div>
              </li>
              <li class="rounded-md border border-border bg-background px-3 py-2">
                <div class="text-xs uppercase tracking-wide text-muted-foreground">Needs review</div>
                <div class="text-lg font-semibold">{counts.needs_review}</div>
              </li>
            </ul>
          {:else}
            <p class="text-sm text-muted-foreground">Loading…</p>
          {/if}
        </CardContent>
      </Card>

      <p class="text-xs text-muted-foreground">
        Metadata powered by <a class="underline" href="https://www.themoviedb.org">TMDB</a>.
      </p>
    {:else}
      <div
        class="rounded-md border border-border bg-card px-4 py-3 text-sm text-muted-foreground"
      >
        Metadata sync is disabled. Pick a mode above to enable.
      </div>
    {/if}
  </div>
</div>
```

- [ ] **Step 4: Type-check**

```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: zero errors.

- [ ] **Step 5: Commit (combined with api.ts changes from A.9)**

```bash
git add src/lib/api.ts src/routes/settings/metadata/+page.svelte
git commit -m "feat(ui): metadata_mode picker and switch to generic settings wrapper

Settings → Metadata page now offers a 5-mode Select above the TMDB key
input. Saving the mode routes through setSetting('metadata_mode', …)
which validates server-side, calls on_setting_changed, and notifies
the worker. The TMDB key input greys out in imdb_only mode (still
editable so the user can prepare a switch). The whole page hides when
mode is off.

Removed the bespoke api.getTmdbApiKey / api.setTmdbApiKey from
api.ts — everything goes through getSetting / setSetting now.

MetadataStatusCounts gains two new fields (tmdb_auth_required,
no_provider_available) and the panel renders them as distinct cards
so a mode-no-key state looks different from a broken-key state."
```

---

### Task A.11: Push, open PR, merge

**Files:** none

- [ ] **Step 1: Push**

```bash
git push -u origin fix/43-metadata-settings-infrastructure
```

- [ ] **Step 2: Open PR**

```bash
gh pr create --title "Metadata fix/43: settings infra + mode wiring + sentinel rename" --body "$(cat <<'EOF'
## Summary
- Generic ``get_app_setting`` / ``set_app_setting`` Tauri commands replace ``get_tmdb_api_key`` / ``set_tmdb_api_key``. Server-side ``validate()`` and ``on_setting_changed`` dispatch handle wake_parked / wake_worker / tmdb_auth_bad clearing.
- New ``metadata::dispatch`` module owns ``Provider`` enum, ``ParkReason``, ``providers_for_mode``.
- Sentinel rename: ``auth_required`` → ``tmdb_auth_required``; new ``no_provider_available``. One-shot fixup in ``db::post_migration_fixups`` (renamed from ``dedupe_shows_and_index``).
- Worker rewrites: mode dispatch, end-of-walk classifier, key-snapshot race guard, merged apply+delete tx.
- Frontend: ``src/lib/settings.ts`` typed wrapper, mode Select on Settings → Metadata, expanded counts panel.

IMDB module lands in fix/44; IMDB modes currently degrade to ``[Tmdb]`` (key present) or park as ``NoProviderAvailable``.

## Test plan
- [ ] ``cargo test`` — all tests pass including new dispatch + settings_tests modules.
- [ ] Launch app: pick each mode; for ``tmdb_only`` with no key verify rows park; for ``imdb_only`` verify the key input greys out; for ``off`` the page collapses to a banner.
- [ ] Edit a title inline (existing flow) — ``metadata_locked`` still flips to 1.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Merge**

```bash
gh pr merge --merge --delete-branch
git checkout master && git pull --ff-only
```

---

# PR B — fix/44-imdb-module-and-dispatch

### Task B.0: Create the branch

- [ ] **Step 1: Branch**

```bash
git checkout master && git pull --ff-only
git checkout -b fix/44-imdb-module-and-dispatch
```

---

### Task B.1: Add `provider: Provider` field to `MatchCandidate` and fix the test helper

**Files:**
- Modify: `src-tauri/src/metadata/matching.rs`

- [ ] **Step 1: Add the field and update the helper**

In `src-tauri/src/metadata/matching.rs`, change the struct:

```rust
use crate::metadata::dispatch::Provider;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MatchCandidate {
    pub provider: Provider,
    pub provider_id: String,
    pub title: String,
    pub year: Option<i32>,
}
```

Inside the `#[cfg(test)] mod tests` block at the bottom, update the `candidate` helper:

```rust
    fn candidate(id: &str, title: &str, year: Option<i32>) -> MatchCandidate {
        MatchCandidate {
            provider: Provider::Tmdb,
            provider_id: id.to_string(),
            title: title.to_string(),
            year,
        }
    }
```

- [ ] **Step 2: Update existing TMDB-side callers**

In `src-tauri/src/metadata/tmdb.rs`, every place `MatchCandidate` is constructed (in `search_movie` and `search_show`), add `provider: Provider::Tmdb,` as the first field. Add the import at top:

```rust
use crate::metadata::dispatch::Provider;
```

Find the two existing constructors and add the field:

```rust
.map(|raw| MatchCandidate {
    provider: Provider::Tmdb,
    provider_id: raw.id.to_string(),
    title: raw.title,
    year: parse_year(raw.release_date.as_deref()),
})
```

(Repeat the same for `search_show`.)

- [ ] **Step 3: Run matcher tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata::matching 2>&1 | tail -15
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 10 matcher tests pass.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/metadata/matching.rs src-tauri/src/metadata/tmdb.rs
git commit -m "feat(metadata): MatchCandidate carries its source provider

Adds Provider field to MatchCandidate so a mixed result list (TMDB +
IMDB) in the Needs-review match sheet can be rendered with the right
source label and linked correctly. TMDB constructors fill in
Provider::Tmdb explicitly. Matcher logic is unchanged."
```

---

### Task B.2: Create the IMDB suggestion-API client

**Files:**
- Create: `src-tauri/src/metadata/imdb.rs`
- Modify: `src-tauri/src/metadata/mod.rs`

- [ ] **Step 1: Create the module with the search function**

Create `src-tauri/src/metadata/imdb.rs`:

```rust
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

use std::path::Path;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
use crate::metadata::dispatch::Provider;
use crate::metadata::matching::MatchCandidate;

const SUGGESTION_BASE: &str = "https://v3.sg.media-imdb.com/suggestion";
const GRAPHQL_URL: &str = "https://caching.graphql.imdb.com/";

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
    // Try year-augmented slug first (better filtering), fall back to plain slug.
    let slug = slugify(title);
    let envelope = match (year, &slug) {
        (Some(y), s) if !s.is_empty() => {
            let augmented = format!("{s}_{y}");
            let result = fetch_suggestion(client, &augmented).await?;
            if result.d.is_empty() {
                fetch_suggestion(client, s).await?
            } else {
                result
            }
        }
        (_, s) if !s.is_empty() => fetch_suggestion(client, s).await?,
        _ => return Ok(vec![]),
    };

    Ok(envelope
        .d
        .into_iter()
        .filter(|entry| entry.id.starts_with("tt"))
        .filter(|entry| match entry.qid.as_deref() {
            Some(q) => qid_filter.contains(&q),
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
        return Err(AppError::Other(format!(
            "imdb_waf: suggestion endpoint returned 202; see CLAUDE.md"
        )));
    }
    if !status.is_success() {
        return Err(AppError::Other(format!(
            "imdb_rate_limited: suggestion {status}"
        )));
    }
    response.json::<SuggestionEnvelope>().await.map_err(|error| {
        AppError::Other(format!("imdb parse: suggestion: {error}"))
    })
}

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
```

- [ ] **Step 2: Wire the module**

In `src-tauri/src/metadata/mod.rs`, add `pub mod imdb;` alongside the existing declarations.

- [ ] **Step 3: Run the slugify tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata::imdb 2>&1 | tail -10
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 2 passing tests.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/metadata/imdb.rs src-tauri/src/metadata/mod.rs
git commit -m "feat(metadata): IMDB suggestion-API search client

Hits v3.sg.media-imdb.com/suggestion/<letter>/<slug>.json for the
search step. Year-augmented slug tried first, falls back to the plain
slug. Filters on qid (movie / tvSeries / tvMiniSeries) before mapping
to MatchCandidate. WAF (HTTP 202) and rate-limit (4xx/5xx) errors are
distinct. 2 slugify unit tests."
```

---

### Task B.3: Add the IMDB GraphQL details client

**Files:**
- Modify: `src-tauri/src/metadata/imdb.rs`

- [ ] **Step 1: Add the GraphQL types + fetch functions**

Append to `src-tauri/src/metadata/imdb.rs`:

```rust
// ---- GraphQL details ----

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
    #[serde(default)]
    pub title_text: Option<TextNode>,
    #[serde(default)]
    pub release_year: Option<ReleaseYearNode>,
    #[serde(default)]
    pub release_date: Option<ReleaseDateNode>,
    #[serde(default)]
    pub plot: Option<PlotNode>,
    #[serde(default)]
    pub ratings_summary: Option<RatingsNode>,
    #[serde(default)]
    pub runtime: Option<RuntimeNode>,
    #[serde(default)]
    pub genres: Option<GenresWrapper>,
    #[serde(default)]
    pub primary_image: Option<PrimaryImage>,
    #[serde(default)]
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

pub async fn fetch_movie_details(
    client: &Client,
    imdb_id: &str,
) -> AppResult<TitleNode> {
    fetch_details_internal(client, imdb_id).await
}

pub async fn fetch_show_details(
    client: &Client,
    imdb_id: &str,
) -> AppResult<TitleNode> {
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
        return Err(AppError::Other(format!(
            "imdb_waf: graphql returned 202; see CLAUDE.md"
        )));
    }
    if !status.is_success() {
        return Err(AppError::Other(format!("imdb_rate_limited: graphql {status}")));
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
        .and_then(|d| d.title)
        .ok_or_else(|| AppError::Other(format!("imdb not_found: {imdb_id}")))?;

    if title.title_text.is_none() {
        // Live-verified: unknown ids return data.title without titleText.
        return Err(AppError::Other(format!("imdb not_found: {imdb_id}")));
    }

    Ok(title)
}
```

- [ ] **Step 2: Add the poster downloader with size enum**

Append:

```rust
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
    dest: &Path,
    size: PosterSize,
) -> AppResult<()> {
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
        // Strip everything between `_V1_` and the next `.` and re-insert our segment.
        let dot = tail.rfind('.').unwrap_or(tail.len());
        return format!("{head}{}{}", size.segment(), &tail[dot..]);
    }
    url.to_string()
}

#[cfg(test)]
mod url_tests {
    use super::*;

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
```

- [ ] **Step 3: Compile**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -15
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean. There may be `serde` field-rename warnings; the `#[serde(rename = …)]` attributes match the actual JSON field names.

- [ ] **Step 4: Run URL rewrite tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata::imdb 2>&1 | tail -10
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 4 tests pass (2 slugify + 2 rewrite_size).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/metadata/imdb.rs
git commit -m "feat(metadata): IMDB GraphQL details client + poster downloader

Hits caching.graphql.imdb.com with the one query that covers both
movies and shows. Unknown IDs return data.title with no titleText —
mapped to imdb not_found error per live verification (the errors[]
path is reserved for actual GraphQL syntax errors).

PosterSize enum supports Small (~94KB SX500) and Hero (QL90 UX1280)
size variants by rewriting the _V1_ segment of the Amazon CDN URL.
2 unit tests cover the rewrite."
```

---

### Task B.4: Apply IMDB details to the DB

**Files:**
- Modify: `src-tauri/src/metadata/apply.rs`

- [ ] **Step 1: Add `apply_imdb_movie_details` and `apply_imdb_show_details`**

In `src-tauri/src/metadata/apply.rs`, after the existing TMDB functions, add:

```rust
use crate::metadata::imdb::{PosterSize, TitleNode};

pub async fn apply_imdb_movie_details(
    conn: &mut sqlx::SqliteConnection,
    movie_id: i64,
    details: &TitleNode,
) -> AppResult<Option<(PosterSize, String, String)>> {
    let current_poster_origin: Option<String> = sqlx::query_scalar(
        "SELECT poster_origin FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut *conn)
    .await?;

    let download_target = compute_imdb_poster(
        current_poster_origin.as_deref(),
        details.primary_image.as_ref().map(|i| i.url.as_str()),
        movie_id,
        "movie",
    );

    let overview = details
        .plot
        .as_ref()
        .and_then(|p| p.plot_text.as_ref())
        .and_then(|t| t.plain_text.as_deref());
    let year = details.release_year.as_ref().and_then(|r| r.year);
    let rating = imdb_rating(&details.ratings_summary);
    let genres_json = serde_json::to_string(
        &details
            .genres
            .as_ref()
            .map(|g| g.genres.iter().map(|x| x.text.clone()).collect::<Vec<_>>())
            .unwrap_or_default(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let cast_json = build_imdb_cast_json(&details.principal_credits);
    let runtime_minutes = details
        .runtime
        .as_ref()
        .and_then(|r| r.seconds)
        .map(|seconds| seconds / 60);

    if let Some((_size, _filename, _local_path)) = download_target.as_ref() {
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
        .bind(&download_target.as_ref().unwrap().2)
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

    Ok(download_target.map(|(size, url, filename)| (size, url, filename)))
}

pub async fn apply_imdb_show_details(
    conn: &mut sqlx::SqliteConnection,
    show_id: i64,
    details: &TitleNode,
) -> AppResult<Option<(PosterSize, String, String)>> {
    let current_poster_origin: Option<String> = sqlx::query_scalar(
        "SELECT poster_origin FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_one(&mut *conn)
    .await?;

    let download_target = compute_imdb_poster(
        current_poster_origin.as_deref(),
        details.primary_image.as_ref().map(|i| i.url.as_str()),
        show_id,
        "show",
    );

    let overview = details
        .plot
        .as_ref()
        .and_then(|p| p.plot_text.as_ref())
        .and_then(|t| t.plain_text.as_deref());
    let year = details.release_year.as_ref().and_then(|r| r.year);
    let rating = imdb_rating(&details.ratings_summary);
    let genres_json = serde_json::to_string(
        &details
            .genres
            .as_ref()
            .map(|g| g.genres.iter().map(|x| x.text.clone()).collect::<Vec<_>>())
            .unwrap_or_default(),
    )
    .unwrap_or_else(|_| "[]".to_string());
    let cast_json = build_imdb_cast_json(&details.principal_credits);
    let first_air_date = details.release_date.as_ref().and_then(|d| {
        match (d.year, d.month, d.day) {
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

fn imdb_rating(ratings: &Option<crate::metadata::imdb::RatingsNode>) -> Option<f64> {
    let ratings = ratings.as_ref()?;
    let votes = ratings.vote_count.unwrap_or(0);
    if votes == 0 {
        // Unreleased titles: voteCount is 0 (not null). Treat as no rating.
        return None;
    }
    ratings.aggregate_rating
}

fn build_imdb_cast_json(credits: &[crate::metadata::imdb::PrincipalCredits]) -> String {
    // Match on category.id == "cast" (lowercase id, stable). The
    // response's category.text is plural ("Stars") — don't filter on it.
    let cast_block = credits.iter().find(|c| c.category.id == "cast");
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
                        "character": credit.characters.first().map(|c| c.name.clone()),
                        "order": index,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    serde_json::to_string(&payload).unwrap_or_else(|_| "[]".to_string())
}

/// Returns Some((size, source_url, local_filename)) if the row's existing
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
```

- [ ] **Step 2: Compile**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -10
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/metadata/apply.rs
git commit -m "feat(metadata): apply IMDB details to shows/movies

apply_imdb_movie_details / apply_imdb_show_details mirror the TMDB
versions. provider='imdb' tag, COALESCE for nullable fields, rating
treated as null when voteCount is 0 (unreleased title), cast filtered
on category.id=='cast' (stable lowercase id) and capped at 10. Series
runtime is intentionally ignored for shows since there's no
episode_runtime column."
```

---

### Task B.5: Add fixture files and JSON parsing tests for IMDB

**Files:**
- Create: `src-tauri/tests/fixtures/imdb-suggestion-movie.json`
- Create: `src-tauri/tests/fixtures/imdb-graphql-movie.json`
- Create: `src-tauri/tests/fixtures/imdb-graphql-show.json`
- Create: `src-tauri/tests/fixtures/imdb-graphql-edge.json`
- Modify: `src-tauri/src/metadata/imdb.rs`

- [ ] **Step 1: Create the suggestion fixture**

Create `src-tauri/tests/fixtures/imdb-suggestion-movie.json` with one representative suggestion response (real or hand-built):

```json
{
  "d": [
    { "id": "tt0133093", "l": "The Matrix", "y": 1999, "qid": "movie" },
    { "id": "tt0234215", "l": "The Matrix Reloaded", "y": 2003, "qid": "movie" },
    { "id": "nm0905152", "l": "Lana Wachowski" }
  ],
  "q": "the_matrix",
  "v": 1
}
```

- [ ] **Step 2: Create the GraphQL movie fixture**

Create `src-tauri/tests/fixtures/imdb-graphql-movie.json`:

```json
{
  "data": {
    "title": {
      "id": "tt0133093",
      "titleText": { "text": "The Matrix" },
      "titleType": { "id": "movie" },
      "releaseYear": { "year": 1999, "endYear": null },
      "releaseDate": { "day": 31, "month": 3, "year": 1999 },
      "plot": { "plotText": { "plainText": "Neo discovers reality is a simulation." } },
      "ratingsSummary": { "aggregateRating": 8.7, "voteCount": 2000000 },
      "runtime": { "seconds": 8160 },
      "genres": { "genres": [{ "id": "action", "text": "Action" }, { "id": "scifi", "text": "Sci-Fi" }] },
      "primaryImage": { "url": "https://m.media-amazon.com/images/M/matrix@._V1_.jpg", "width": 2100, "height": 3156 },
      "principalCredits": [
        {
          "category": { "id": "director", "text": "Directors" },
          "credits": [{ "name": { "id": "nm0905152", "nameText": { "text": "Lana Wachowski" } } }]
        },
        {
          "category": { "id": "cast", "text": "Stars" },
          "credits": [
            { "name": { "id": "nm0000206", "nameText": { "text": "Keanu Reeves" } }, "characters": [{ "name": "Neo" }] },
            { "name": { "id": "nm0000401", "nameText": { "text": "Laurence Fishburne" } }, "characters": [{ "name": "Morpheus" }] }
          ]
        }
      ]
    }
  }
}
```

- [ ] **Step 3: Create the GraphQL show fixture**

Create `src-tauri/tests/fixtures/imdb-graphql-show.json`:

```json
{
  "data": {
    "title": {
      "id": "tt0903747",
      "titleText": { "text": "Breaking Bad" },
      "titleType": { "id": "tvSeries" },
      "releaseYear": { "year": 2008, "endYear": 2013 },
      "releaseDate": { "day": 20, "month": 1, "year": 2008 },
      "plot": { "plotText": { "plainText": "Chemistry teacher cooks meth." } },
      "ratingsSummary": { "aggregateRating": 9.5, "voteCount": 2000000 },
      "runtime": { "seconds": 2880 },
      "genres": { "genres": [{ "id": "crime", "text": "Crime" }, { "id": "drama", "text": "Drama" }] },
      "primaryImage": { "url": "https://m.media-amazon.com/images/M/bb@._V1_.jpg", "width": 1600, "height": 2400 },
      "principalCredits": [
        {
          "category": { "id": "cast", "text": "Stars" },
          "credits": [
            { "name": { "id": "nm0186505", "nameText": { "text": "Bryan Cranston" } }, "characters": [{ "name": "Walter White" }] }
          ]
        }
      ]
    }
  }
}
```

- [ ] **Step 4: Create the edge-case fixture (unreleased title, null fields)**

Create `src-tauri/tests/fixtures/imdb-graphql-edge.json`:

```json
{
  "data": {
    "title": {
      "id": "tt31378509",
      "titleText": { "text": "Dune Part Three" },
      "titleType": { "id": "movie" },
      "releaseYear": { "year": 2026, "endYear": null },
      "releaseDate": null,
      "plot": null,
      "ratingsSummary": { "aggregateRating": null, "voteCount": 0 },
      "runtime": null,
      "genres": { "genres": [] },
      "primaryImage": null,
      "principalCredits": []
    }
  }
}
```

- [ ] **Step 5: Add parsing tests using the fixtures**

In `src-tauri/src/metadata/imdb.rs`, extend the `#[cfg(test)]` modules:

```rust
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
            .filter(|e| e.id.starts_with("tt") && e.qid.as_deref() == Some("movie"))
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
            .find(|c| c.category.id == "cast")
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
        assert_eq!(
            title.ratings_summary.as_ref().unwrap().vote_count,
            Some(0),
        );
        assert!(title.runtime.is_none());
        assert!(title.primary_image.is_none());
    }
}
```

- [ ] **Step 6: Run the parser tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata::imdb 2>&1 | tail -20
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: all 8 tests pass (2 slugify, 2 url_tests, 4 fixture_tests).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/metadata/imdb.rs src-tauri/tests/fixtures/
git commit -m "test(metadata): IMDB JSON parser fixtures and tests

5 fixture files (suggestion-movie, graphql-movie, graphql-show,
graphql-edge) and 4 fixture-driven unit tests cover the IMDB module's
JSON parsing surface, including the unreleased-title edge case where
voteCount is 0 and runtime / primaryImage are null."
```

---

### Task B.6: Wire IMDB into `dispatch_provider` and update `providers_for_mode`

**Files:**
- Modify: `src-tauri/src/metadata/dispatch.rs`
- Modify: `src-tauri/src/metadata/worker.rs`

- [ ] **Step 1: Replace `providers_for_mode` with real IMDB walks**

In `src-tauri/src/metadata/dispatch.rs`, replace the `providers_for_mode` function body:

```rust
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
            // prefer_tmdb (default)
            if has_tmdb_key {
                Ok(vec![Tmdb, Imdb])
            } else {
                Ok(vec![Imdb])
            }
        }
    }
}
```

- [ ] **Step 2: Update the dispatch tests**

The tests in the same file from fix/43 still test `[Tmdb]` for IMDB modes. Replace each test to reflect the new real walks:

```rust
    #[test]
    fn imdb_only_returns_imdb_always() {
        assert_eq!(providers_for_mode("imdb_only", true).unwrap(), vec![Provider::Imdb]);
        assert_eq!(providers_for_mode("imdb_only", false).unwrap(), vec![Provider::Imdb]);
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
```

Delete the old "degraded to [Tmdb]" tests from fix/43 — they're obsolete.

- [ ] **Step 3: Implement IMDB dispatch in the worker**

In `src-tauri/src/metadata/worker.rs`, replace the `Provider::Imdb` branch of `dispatch_provider` with a real `dispatch_imdb` call. Add a new `dispatch_imdb` function:

```rust
async fn dispatch_imdb(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let result = match job.kind.as_str() {
        "movie" => dispatch_imdb_movie(pool, http, job.media_id).await?,
        "show" => dispatch_imdb_show(pool, http, job.media_id).await?,
        other => return Err(AppError::Other(format!("unknown job kind: {other}"))),
    };

    match result {
        MatchOutcome::NoMatch => Ok(Outcome::NoMatch),
        MatchOutcome::Matched { poster } => {
            if let Some((size, url, filename)) = poster {
                let dest = posters_dir.join(filename);
                if let Err(error) = imdb::download_poster(http, &url, &dest, size).await {
                    eprintln!("imdb poster download failed for {dest:?}: {error}");
                }
            }
            Ok(Outcome::Matched)
        }
    }
}

async fn dispatch_imdb_movie(
    pool: &SqlitePool,
    http: &reqwest::Client,
    movie_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = imdb::search_movie(http, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(MatchOutcome::NoMatch);
    };

    let details = imdb::fetch_movie_details(http, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(MatchOutcome::NoMatch);
    }

    let download = apply::apply_imdb_movie_details(&mut *tx, movie_id, &details).await?;

    queries::delete_in_tx(&mut *tx, "movie", movie_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: download.map(|(size, url, filename)| (size, url, filename)),
    })
}

async fn dispatch_imdb_show(
    pool: &SqlitePool,
    http: &reqwest::Client,
    show_id: i64,
) -> AppResult<MatchOutcome> {
    let row: Option<(i64, String, Option<i32>)> = sqlx::query_as(
        "SELECT metadata_locked, title, year FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_optional(pool)
    .await?;

    let Some((locked, title, year)) = row else {
        return Ok(MatchOutcome::NoMatch);
    };
    if locked != 0 {
        return Ok(MatchOutcome::NoMatch);
    }

    let candidates = imdb::search_show(http, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(MatchOutcome::NoMatch);
    };

    let details = imdb::fetch_show_details(http, &pick.provider_id).await?;

    let mut tx = pool.begin().await?;

    let still_locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_one(&mut *tx)
    .await?;
    if still_locked != 0 {
        tx.rollback().await?;
        return Ok(MatchOutcome::NoMatch);
    }

    let download = apply::apply_imdb_show_details(&mut *tx, show_id, &details).await?;

    queries::delete_in_tx(&mut *tx, "show", show_id).await?;
    tx.commit().await?;

    Ok(MatchOutcome::Matched {
        poster: download.map(|(size, url, filename)| (size, url, filename)),
    })
}
```

Also update the `MatchOutcome::Matched.poster` field type so it carries `Option<(PosterSize, String, String)>` (size + URL + filename) — and update `dispatch_tmdb` to fill `Some((PosterSize::Small, …, …))` for TMDB too (TMDB always uses Small). The `dispatch_tmdb` `download_poster` call signature changes to take a `PosterSize` (use `PosterSize::Small` everywhere; the TMDB module continues to hardcode `w500` internally — the size enum is IMDB-only conceptually, but the type unification lets the worker not branch).

Alternatively, keep TMDB's `download_poster` unchanged and only pass a `PosterSize` when the provider is IMDB. That keeps the surface narrower but requires the dispatch path to branch. Pick whichever feels cleaner; the spec doesn't mandate.

- [ ] **Step 4: Update `dispatch_provider`'s IMDB branch**

In the same `worker.rs` file, replace the placeholder IMDB branch:

```rust
async fn dispatch_provider(
    provider: Provider,
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<Outcome> {
    match provider {
        Provider::Tmdb => dispatch_tmdb(pool, http, app, api_key, job).await,
        Provider::Imdb => dispatch_imdb(pool, http, app, job).await,
    }
}
```

- [ ] **Step 5: Add the `metadata/imdb` import at the top of worker.rs**

```rust
use crate::metadata::{apply, dispatch::*, imdb, matching, queries, tmdb};
```

- [ ] **Step 6: Compile, run all metadata tests**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo test --lib metadata 2>&1 | tail -25
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: every metadata test passes, including new IMDB dispatch coverage.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/metadata/dispatch.rs src-tauri/src/metadata/worker.rs
git commit -m "feat(metadata): real IMDB walks and dispatch_imdb in worker

providers_for_mode now returns Imdb (alone or before/after Tmdb) for
the IMDB-touching modes. dispatch_imdb mirrors the TMDB shape: HTTP
outside the tx, apply + delete merged inside one tx. dispatch_provider
routes Provider::Imdb to dispatch_imdb. The fix/43-era degrade is
gone."
```

---

### Task B.7: Add the hand-linked fast path to the worker

**Files:**
- Modify: `src-tauri/src/metadata/worker.rs`

- [ ] **Step 1: Add a helper to read the linked provider for a row**

In `src-tauri/src/metadata/worker.rs`, add:

```rust
async fn read_linked_provider(
    pool: &SqlitePool,
    job: &queries::MetadataJob,
) -> AppResult<Option<Provider>> {
    let table = match job.kind.as_str() {
        "movie" => "movies",
        "show" => "shows",
        _ => return Ok(None),
    };
    let provider: Option<String> = sqlx::query_scalar(&format!(
        "SELECT provider FROM {table} WHERE id = ?1"
    ))
    .bind(job.media_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    Ok(match provider.as_deref() {
        Some("tmdb") => Some(Provider::Tmdb),
        Some("imdb") => Some(Provider::Imdb),
        _ => None,
    })
}
```

- [ ] **Step 2: Insert the fast path before `providers_for_mode`**

In the `run` loop, after computing `key_at_job_start` and right before the `providers_for_mode` call, add:

```rust
        // Fast path: hand-linked rows bypass mode. The user's pick wins.
        if let Some(linked) = read_linked_provider(&pool, &job).await? {
            let key_for_call = api_key.as_deref().unwrap_or("");
            match dispatch_provider(linked, &pool, &http, &app, key_for_call, &job).await {
                Ok(Outcome::Matched) => {
                    sleep(Duration::from_millis(PACING_MS)).await;
                    continue;
                }
                Ok(Outcome::NoMatch) => {
                    let mut tx = pool.begin().await?;
                    queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
                    tx.commit().await?;
                    sleep(Duration::from_millis(PACING_MS)).await;
                    continue;
                }
                Err(error) => {
                    queries::record_failure(
                        &pool, &job.kind, job.media_id, &error.to_string(),
                    ).await?;
                    sleep(Duration::from_millis(PACING_MS)).await;
                    continue;
                }
            }
        }
```

This ensures the user's manual pick is authoritative regardless of mode.

- [ ] **Step 3: Compile**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -8
rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/metadata/worker.rs
git commit -m "feat(metadata): hand-linked fast path bypasses mode

When a row already has provider + provider_id set (user manually picked
from the Needs-review match sheet, or a prior sync succeeded and the
user clicked Refresh), the worker dispatches directly against the
linked provider regardless of the current metadata_mode. The user's
manual link is the source of truth."
```

---

### Task B.8: Add `provider` parameter to `metadata_search` and `link_metadata`

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Update `metadata_search` to take a provider**

In `src-tauri/src/commands.rs`, find `pub async fn metadata_search`. Replace with:

```rust
#[tauri::command]
pub async fn metadata_search(
    db: State<'_, Db>,
    http: State<'_, reqwest::Client>,
    kind: String,
    query: String,
    year: Option<i32>,
    provider: String,
) -> AppResult<Vec<crate::metadata::matching::MatchCandidate>> {
    match provider.as_str() {
        "tmdb" => {
            let api_key = queries::get_app_setting(&db, "tmdb_api_key")
                .await?
                .ok_or_else(|| AppError::Other("no TMDB key configured".to_string()))?;
            match kind.as_str() {
                "movie" => crate::metadata::tmdb::search_movie(&http, &api_key, &query, year).await,
                "show" => crate::metadata::tmdb::search_show(&http, &api_key, &query, year).await,
                other => Err(AppError::Other(format!("unknown kind: {other}"))),
            }
        }
        "imdb" => match kind.as_str() {
            "movie" => crate::metadata::imdb::search_movie(&http, &query, year).await,
            "show" => crate::metadata::imdb::search_show(&http, &query, year).await,
            other => Err(AppError::Other(format!("unknown kind: {other}"))),
        },
        other => Err(AppError::Other(format!("unknown provider: {other}"))),
    }
}
```

- [ ] **Step 2: Update `link_metadata` to take and store the provider**

Replace `pub async fn link_metadata`:

```rust
#[tauri::command]
pub async fn link_metadata(
    app: AppHandle,
    db: State<'_, Db>,
    kind: String,
    media_id: i64,
    provider: String,
    provider_id: String,
) -> AppResult<()> {
    if !matches!(provider.as_str(), "tmdb" | "imdb") {
        return Err(AppError::Other(format!("unknown provider: {provider}")));
    }
    let table = match kind.as_str() {
        "show" => "shows",
        "movie" => "movies",
        other => return Err(AppError::Other(format!("unknown kind: {other}"))),
    };

    sqlx::query(&format!(
        "UPDATE {table} SET provider = ?2, provider_id = ?3, metadata_locked = 0
         WHERE id = ?1"
    ))
    .bind(media_id)
    .bind(&provider)
    .bind(&provider_id)
    .execute(&*db)
    .await?;

    crate::metadata::queries::force_enqueue(&db, &kind, media_id).await?;
    wake_worker(&app);

    Ok(())
}
```

- [ ] **Step 3: Update the frontend api.ts bindings**

In `src/lib/api.ts`, replace:

```ts
  metadataSearch: (kind: 'show' | 'movie', query: string, year: number | null) =>
    invoke<MatchCandidate[]>('metadata_search', { kind, query, year }),
  linkMetadata: (kind: 'show' | 'movie', mediaId: number, providerId: string) =>
    invoke<void>('link_metadata', { kind, mediaId, providerId }),
```

With:

```ts
  metadataSearch: (
    kind: 'show' | 'movie',
    query: string,
    year: number | null,
    provider: 'tmdb' | 'imdb',
  ) =>
    invoke<MatchCandidate[]>('metadata_search', { kind, query, year, provider }),
  linkMetadata: (
    kind: 'show' | 'movie',
    mediaId: number,
    provider: 'tmdb' | 'imdb',
    providerId: string,
  ) =>
    invoke<void>('link_metadata', { kind, mediaId, provider, providerId }),
```

Also extend the `MatchCandidate` interface:

```ts
export interface MatchCandidate {
  provider: 'tmdb' | 'imdb';
  provider_id: string;
  title: string;
  year: number | null;
}
```

- [ ] **Step 4: Update the existing `MetadataMatchSheet.svelte` caller (provisionally)**

`src/lib/components/MetadataMatchSheet.svelte` calls `api.metadataSearch` and `api.linkMetadata`. Without a provider toggle in fix/44 (that's fix/45), default to TMDB to keep existing behaviour:

```svelte
candidates = await api.metadataSearch(item.kind, item.title, item.year, 'tmdb');
```

And:

```svelte
await api.linkMetadata(item.kind, item.id, 'tmdb', candidate.provider_id);
```

This is a holdover until fix/45 lands the real toggle.

- [ ] **Step 5: Compile + type-check**

```bash
cd src-tauri
touch binaries/mpv-x86_64-unknown-linux-gnu
cargo check 2>&1 | tail -8
rm -f binaries/mpv-x86_64-unknown-linux-gnu
cd /mnt/c/Programmering/Rust/rustflix
CI=true pnpm check 2>&1 | tail -8
```
Expected: both clean.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/commands.rs src/lib/api.ts src/lib/components/MetadataMatchSheet.svelte
git commit -m "feat(metadata): metadata_search + link_metadata take a provider

Both manual-override Tauri commands now take a provider string
('tmdb' or 'imdb'); link_metadata stops hardcoding provider = 'tmdb'
in the UPDATE. Frontend MatchCandidate gains a provider field that
mirrors the Rust side. MetadataMatchSheet still defaults to TMDB
until fix/45 lands the provider toggle."
```

---

### Task B.9: Push, PR, merge

- [ ] **Step 1: Push and open PR**

```bash
git push -u origin fix/44-imdb-module-and-dispatch
gh pr create --title "Metadata fix/44: IMDB module + worker dispatch + hand-link fast path" --body "$(cat <<'EOF'
## Summary
- New ``metadata::imdb`` module hits ``v3.sg.media-imdb.com/suggestion`` for search and ``caching.graphql.imdb.com`` for details. No HTML scraping. No new crates (reqwest + serde_json suffice).
- ``MatchCandidate`` carries the source provider; both TMDB and IMDB return the same shape.
- ``apply_imdb_*`` writes the same DB columns as TMDB with ``provider = 'imdb'``.
- ``dispatch_provider`` now routes Provider::Imdb to a real dispatcher; HTTP outside the tx, apply + delete merged in one tx.
- Worker fast path: hand-linked rows bypass ``providers_for_mode`` entirely so the user's manual pick is authoritative.
- ``metadata_search`` / ``link_metadata`` take a provider parameter; the UI still defaults to TMDB until fix/45.

## Test plan
- [ ] ``cargo test`` — all tests pass (matcher, dispatch, queries, imdb parser fixtures).
- [ ] Set mode to ``prefer_imdb`` with no key → scan a library → all rows enrich from IMDB.
- [ ] Set mode to ``prefer_tmdb`` with key → known obscure title TMDB misses → IMDB picks it up.
- [ ] Manually link a row to IMDB while mode is ``tmdb_only`` → fast path refreshes against IMDB on the next worker pass (mode ignored).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 2: Merge**

```bash
gh pr merge --merge --delete-branch
git checkout master && git pull --ff-only
```

---

# PR C — fix/45-match-sheet-provider-toggle-and-banners

### Task C.0: Branch

- [ ] **Step 1**

```bash
git checkout master && git pull --ff-only
git checkout -b fix/45-match-sheet-provider-toggle-and-banners
```

---

### Task C.1: Add TMDB / IMDB tabs to `MetadataMatchSheet.svelte`

**Files:**
- Modify: `src/lib/components/MetadataMatchSheet.svelte`

- [ ] **Step 1: Rewrite the sheet with provider tabs**

Replace the entire `src/lib/components/MetadataMatchSheet.svelte` with:

```svelte
<script lang="ts">
  import {
    api,
    type MatchCandidate,
    type NeedsReviewItem,
  } from '$lib/api';
  import { getSetting } from '$lib/settings';
  import { Button } from '$lib/components/ui/button';
  import * as Sheet from '$lib/components/ui/sheet';

  type Props = {
    open: boolean;
    item: NeedsReviewItem | null;
    onClose: () => void;
    onLinked: () => void;
  };

  let { open = $bindable(), item, onClose, onLinked }: Props = $props();

  let activeProvider = $state<'tmdb' | 'imdb'>('tmdb');
  let candidates = $state<MatchCandidate[]>([]);
  let searching = $state(false);
  let error = $state<string | null>(null);
  let hasTmdbKey = $state(false);

  $effect(() => {
    if (open && item) {
      void initialise();
    }
  });

  async function initialise() {
    try {
      const [mode, key] = await Promise.all([
        getSetting('metadata_mode'),
        getSetting('tmdb_api_key'),
      ]);
      hasTmdbKey = key !== null && key.length > 0;
      activeProvider = preferredProviderForMode(mode, hasTmdbKey);
      await runSearch();
    } catch (caught) {
      error = String(caught);
    }
  }

  function preferredProviderForMode(
    mode: string,
    hasKey: boolean,
  ): 'tmdb' | 'imdb' {
    if (mode === 'imdb_only' || mode === 'prefer_imdb') {
      return 'imdb';
    }
    if (mode === 'tmdb_only' || mode === 'prefer_tmdb') {
      return hasKey ? 'tmdb' : 'imdb';
    }
    return 'tmdb';
  }

  async function runSearch() {
    if (!item) {
      return;
    }
    searching = true;
    error = null;
    candidates = [];
    try {
      candidates = await api.metadataSearch(
        item.kind,
        item.title,
        item.year,
        activeProvider,
      );
    } catch (caught) {
      error = String(caught);
    } finally {
      searching = false;
    }
  }

  async function selectProvider(next: 'tmdb' | 'imdb') {
    if (next === 'tmdb' && !hasTmdbKey) {
      return;
    }
    activeProvider = next;
    await runSearch();
  }

  async function pick(candidate: MatchCandidate) {
    if (!item) {
      return;
    }
    error = null;
    try {
      await api.linkMetadata(
        item.kind,
        item.id,
        activeProvider,
        candidate.provider_id,
      );
      onLinked();
      onClose();
    } catch (caught) {
      error = String(caught);
    }
  }
</script>

<Sheet.Root bind:open>
  <Sheet.Content side="right" class="w-full sm:max-w-md">
    <Sheet.Header>
      <Sheet.Title>Find a match</Sheet.Title>
      <Sheet.Description>
        {item ? `${item.title}${item.year ? ` (${item.year})` : ''}` : ''}
      </Sheet.Description>
    </Sheet.Header>

    <div class="mt-4 flex gap-1 rounded-md border border-border bg-card p-1">
      <button
        type="button"
        disabled={!hasTmdbKey}
        onclick={() => selectProvider('tmdb')}
        title={!hasTmdbKey ? 'Add a TMDB key under Settings → Metadata' : undefined}
        class="flex-1 rounded px-3 py-1.5 text-sm transition-colors disabled:opacity-50 {activeProvider === 'tmdb' ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'}"
      >
        TMDB
      </button>
      <button
        type="button"
        onclick={() => selectProvider('imdb')}
        class="flex-1 rounded px-3 py-1.5 text-sm transition-colors {activeProvider === 'imdb' ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'}"
      >
        IMDB
      </button>
    </div>

    {#if error}
      <p class="mt-3 text-sm text-destructive-foreground">{error}</p>
    {/if}

    {#if searching}
      <p class="mt-3 text-sm text-muted-foreground">Searching…</p>
    {:else}
      <ul class="mt-4 flex flex-col gap-2">
        {#each candidates as candidate (candidate.provider + ':' + candidate.provider_id)}
          <li>
            <button
              type="button"
              onclick={() => pick(candidate)}
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-left text-sm transition-colors hover:bg-accent"
            >
              <div class="font-medium">{candidate.title}</div>
              <div class="text-xs text-muted-foreground">
                {candidate.year ?? '—'} · {candidate.provider.toUpperCase()} · {candidate.provider_id}
              </div>
            </button>
          </li>
        {/each}
        {#if candidates.length === 0 && !searching}
          <li class="text-sm text-muted-foreground">No candidates found.</li>
        {/if}
      </ul>
    {/if}

    <Sheet.Footer class="mt-6">
      <Button variant="ghost" onclick={onClose}>Close</Button>
    </Sheet.Footer>
  </Sheet.Content>
</Sheet.Root>
```

- [ ] **Step 2: Type-check**

```bash
CI=true pnpm check 2>&1 | tail -8
```
Expected: zero errors.

- [ ] **Step 3: Commit**

```bash
git add src/lib/components/MetadataMatchSheet.svelte
git commit -m "feat(ui): provider toggle on Needs-review match sheet

Two-tab UI (TMDB / IMDB) at the top of the sheet. Default tab follows
the current metadata_mode (IMDB-mode → IMDB; TMDB-mode + key → TMDB;
TMDB-mode without key → IMDB). TMDB tab disabled with a tooltip when
no key is set. Picking a candidate calls link_metadata with the
active provider so the hand-link is authoritative on future syncs."
```

---

### Task C.2: Add the `tmdb_auth_bad` and mode-off banners to the Metadata settings page

**Files:**
- Modify: `src/routes/settings/metadata/+page.svelte`

- [ ] **Step 1: Read the existing page (after fix/43)**

The page already collapses to a "Metadata sync is disabled" message when mode is off. Add a banner for `tmdb_auth_bad` and tighten the messaging.

- [ ] **Step 2: Add the `tmdb_auth_bad` state and load it**

In the `<script>` block, after the other state declarations, add:

```ts
  let authBad = $state(false);
```

In the `load()` function, parallelise an extra `getSetting`-equivalent read. Since `tmdb_auth_bad` is just a presence-of-value flag, hit it through the generic API:

```ts
  async function load() {
    try {
      const [keyResult, modeResult, countsResult, authBadResult] = await Promise.all([
        getSetting('tmdb_api_key'),
        getSetting('metadata_mode'),
        api.metadataStatusCounts(),
        // tmdb_auth_bad isn't in the typed SETTINGS map; read raw.
        invokeRaw<string | null>('get_app_setting', { key: 'tmdb_auth_bad' }),
      ]);
      savedKey = keyResult;
      mode = modeResult;
      counts = countsResult;
      keyDraft = savedKey ?? '';
      authBad = authBadResult === '1';
    } catch (caught) {
      error = String(caught);
    }
  }
```

The `invokeRaw` helper needs importing from `@tauri-apps/api/core`:

```ts
  import { invoke as invokeRaw } from '@tauri-apps/api/core';
```

(Or add `tmdb_auth_bad` to the SETTINGS map in `settings.ts` as a free-form key — simpler. Pick whichever; the raw-invoke route avoids growing the typed map for an internal flag.)

- [ ] **Step 3: Add the banner to the template**

Just below the existing error banner, add:

```svelte
  {#if authBad && mode !== 'off' && mode !== 'imdb_only'}
    <div
      class="mb-6 rounded-md border border-yellow-500/30 bg-yellow-500/10 px-4 py-3 text-sm text-yellow-200"
    >
      Your TMDB key was rejected on the last sync attempt. Paste a new key below to resume.
    </div>
  {/if}
```

(The mode condition ensures the banner doesn't show when TMDB is irrelevant.)

- [ ] **Step 4: Type-check**

```bash
CI=true pnpm check 2>&1 | tail -8
```
Expected: zero errors.

- [ ] **Step 5: Commit**

```bash
git add src/routes/settings/metadata/+page.svelte
git commit -m "feat(ui): tmdb_auth_bad banner on the Metadata settings page

Shows a yellow alert when the worker's last TMDB call returned 401
and the user hasn't yet refreshed the key. Hidden when mode is off
or imdb_only (TMDB irrelevant in those states). Cleared by
on_setting_changed on the next key save."
```

---

### Task C.3: Push, PR, merge

- [ ] **Step 1: Push and PR**

```bash
git push -u origin fix/45-match-sheet-provider-toggle-and-banners
gh pr create --title "Metadata fix/45: match-sheet provider toggle + auth-bad banner" --body "$(cat <<'EOF'
## Summary
- Two-tab UI on the Needs-review match sheet (TMDB / IMDB). Default follows the active mode. TMDB tab disabled when no key.
- ``tmdb_auth_bad`` banner on Settings → Metadata when the worker has hit a 401 and the user hasn't fixed the key.
- Frontend-only PR; no backend changes.

Closes out the IMDB-fallback rollout.

## Test plan
- [ ] Open Needs-review on a row → sheet defaults to the right provider per mode.
- [ ] Pick an IMDB candidate while in ``tmdb_only`` mode → row hand-links to IMDB; worker fast path refreshes via IMDB on next pass.
- [ ] Break the TMDB key, force a sync, observe the banner appearing on the Settings → Metadata page.
- [ ] Save a new key → banner disappears.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 2: Merge**

```bash
gh pr merge --merge --delete-branch
git checkout master && git pull --ff-only
```

---

## Post-merge verification

After all three PRs land, walk these checks against a real library:

- [ ] Pick each of the five `metadata_mode` values and confirm the worker behaves per the spec.
- [ ] Add a known-obscure non-English title to the library; verify TMDB miss + IMDB fallback (in `prefer_tmdb` mode).
- [ ] Manually link a row to the other provider via the match sheet; verify the fast path on the next worker pass.
- [ ] Break the TMDB key; verify the banner and that IMDB keeps the queue moving in `prefer_tmdb`.
- [ ] Switch modes and watch parked jobs wake (no orphaned `no_provider_available` rows).
- [ ] Refresh a row; verify the lock clears and the worker re-fetches via the linked provider.
