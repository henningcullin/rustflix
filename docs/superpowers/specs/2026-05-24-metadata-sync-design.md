# Metadata sync — design

Status: approved 2026-05-24, ready for implementation planning.

## Goal

Automatically enrich locally-scanned shows and movies with external metadata
(poster, overview, year, genres, rating, top cast) from TMDB. Survive the
v1.0.0 trap: keep the schema lean, avoid normalising every relationship into
its own table, never overwrite a field the user has hand-edited.

## Non-goals

- Episode-level metadata sync (per-episode posters, summaries, air dates).
- Multiple metadata providers in v1 — TMDB only.
- Person pages, "all movies starring X" queries, normalised people/genres
  tables.
- Backdrop / fanart images.
- Trailer URLs, alternative titles, alternative posters.
- Background re-sync on a schedule. Re-sync is user-triggered via "Refresh
  metadata".
- Bundled TMDB API key. The user provides their own.

## Decisions taken during brainstorming

1. **Field scope** — Tier 2: poster, overview, year, genres, rating, top
   cast (5–10).
2. **Provider abstraction** — none in v1. Single concrete `tmdb` module.
   Extract a trait when a second provider actually appears.
3. **Matching** — confidence-based. Exact title (after normalisation) +
   year ± 1 returning exactly one candidate ⇒ auto-link. Anything else ⇒
   leave unlinked, surface in a "Needs review" list with a picker.
4. **Trigger** — SQLite-backed background queue, single worker, durable
   across app restarts. Scanner enqueues on new inserts.
5. **Storage shape** — JSON blobs on `shows` / `movies` for `genres` and
   `top_cast`. No new normalised tables.
6. **User-edit precedence** — once a user edits any metadata field on an
   item, that item is locked from sync until they explicitly refresh.

## Architecture

A new `metadata` subsystem in `src-tauri/src/metadata/`:

- `tmdb.rs` — concrete TMDB client. `reqwest` + a few hand-rolled response
  structs. Owns the image-URL base path.
- `matching.rs` — pure function `pick_confident_match` plus a `normalize`
  helper. Heavily testable.
- `queries.rs` — SQL helpers for the job queue (`enqueue`, `next_due`,
  `record_failure`, `delete_on_success`) and metadata write
  (`apply_show_details`, `apply_movie_details`).
- `worker.rs` — single tokio task drained by an `Arc<Notify>`. Owned by
  Tauri app state.

Posters fetched from TMDB are downloaded to
`<app_data>/posters/{kind}-{id}.{ext}` — the same path manual uploads use
today — and stamped with `poster_origin = 'tmdb'`. The existing "Reset to
auto" UI semantics are preserved: `tmdb` and `auto` both mean "scanner /
sync owns this", `manual` means "user owns this".

The metadata subsystem never touches files outside `<app_data>/posters/`.

## Schema

Migration `src-tauri/migrations/0003_metadata_sync.sql`:

```sql
ALTER TABLE shows  ADD COLUMN provider           TEXT;          -- 'tmdb', NULL = unlinked
ALTER TABLE shows  ADD COLUMN provider_id        TEXT;          -- e.g. '1396'
ALTER TABLE shows  ADD COLUMN rating             REAL;          -- 0.0..10.0
ALTER TABLE shows  ADD COLUMN genres             TEXT;          -- JSON array ["Drama","Crime"]
ALTER TABLE shows  ADD COLUMN top_cast           TEXT;          -- JSON [{name,character,order}]
ALTER TABLE shows  ADD COLUMN first_air_date     TEXT;          -- ISO yyyy-mm-dd
ALTER TABLE shows  ADD COLUMN metadata_synced_at INTEGER;
ALTER TABLE shows  ADD COLUMN metadata_locked    INTEGER NOT NULL DEFAULT 0;

ALTER TABLE movies ADD COLUMN provider           TEXT;
ALTER TABLE movies ADD COLUMN provider_id        TEXT;
ALTER TABLE movies ADD COLUMN rating             REAL;
ALTER TABLE movies ADD COLUMN genres             TEXT;
ALTER TABLE movies ADD COLUMN top_cast           TEXT;
ALTER TABLE movies ADD COLUMN runtime_minutes    INTEGER;
ALTER TABLE movies ADD COLUMN metadata_synced_at INTEGER;
ALTER TABLE movies ADD COLUMN metadata_locked    INTEGER NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX idx_shows_provider
    ON shows(provider, provider_id)  WHERE provider IS NOT NULL;
CREATE UNIQUE INDEX idx_movies_provider
    ON movies(provider, provider_id) WHERE provider IS NOT NULL;

CREATE TABLE metadata_jobs (
    kind            TEXT    NOT NULL,                                          -- 'show' | 'movie'
    media_id        INTEGER NOT NULL,
    enqueued_at     INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    attempts        INTEGER NOT NULL DEFAULT 0,
    last_error      TEXT,                                                      -- 'auth_required' sentinel parks the job
    next_attempt_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (kind, media_id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);  -- initial keys: 'tmdb_api_key'
```

### Why these shapes

- **`metadata_status` deliberately omitted.** State derives from
  `(provider, metadata_jobs row?, attempts, last_error)`. Storing it would
  duplicate state and create a drift surface.
- **`provider` + `provider_id`** as two columns rather than one
  `external_id`, so queries like
  `WHERE provider = 'tmdb' AND provider_id = ?` are direct and indexable.
- **Partial UNIQUE indexes** prevent two shows from accidentally linking to
  the same TMDB id, while leaving the unlinked majority of rows free.
- **`top_cast` rather than `cast`** avoids overlap with the `CAST(...)` SQL
  keyword.
- **`metadata_locked` as a single boolean** rather than per-field tracking.
  Once the user edits any text metadata field (title, year, overview,
  etc.), they own the row's text. Simpler and matches the user's
  "don't sprawl" rule.
- **`poster_origin = 'manual'` and `metadata_locked` are independent.**
  Uploading a custom poster protects the poster only — text sync still
  happens. Editing a title locks the text only — a TMDB-supplied poster
  can still arrive on the same item. Two orthogonal user-intent signals.
- **Job queue as desired-state**, not as a log: at most one row per
  `(kind, media_id)`. "Refresh metadata" is
  `INSERT … ON CONFLICT DO UPDATE SET attempts=0, next_attempt_at=now,
  last_error=NULL`.

## Concrete TMDB module

`src-tauri/src/metadata/tmdb.rs` — public surface:

```rust
pub async fn search_movie(client: &Client, key: &str, title: &str, year: Option<i32>)
    -> AppResult<Vec<TmdbMatch>>;

pub async fn search_show(client: &Client, key: &str, title: &str, year: Option<i32>)
    -> AppResult<Vec<TmdbMatch>>;

pub async fn fetch_movie_details(client: &Client, key: &str, tmdb_id: &str)
    -> AppResult<TmdbMovieDetails>;

pub async fn fetch_show_details(client: &Client, key: &str, tmdb_id: &str)
    -> AppResult<TmdbShowDetails>;

pub async fn download_poster(client: &Client, poster_path: &str, dest: &Path)
    -> AppResult<()>;
```

`TmdbMatch` / `TmdbMovieDetails` / `TmdbShowDetails` are local concrete
structs. No neutral wrappers — when a second provider arrives, the shared
shape gets lifted out then.

Endpoints used:

- `GET /search/movie?query=&year=`
- `GET /search/tv?query=&first_air_date_year=`
- `GET /movie/{id}?append_to_response=credits`
- `GET /tv/{id}?append_to_response=credits`

Image base URL hard-coded in this module: `https://image.tmdb.org/t/p/w500`.

One shared `reqwest::Client` constructed at app startup with `timeout(30s)`
and a `User-Agent`. Passed by reference into the provider functions, not
constructed per call.

HTTP errors map to `AppError::Other(...)`. 401 specifically maps to
`AppError::Other("auth_required: ...")` so the worker can park rather than
retry.

New crates in `Cargo.toml`:

- `reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }`
- `unicode-normalization = "0.1"`

`rustls-tls` avoids a system-OpenSSL dependency.

## Matching

`src-tauri/src/metadata/matching.rs`:

```rust
pub fn pick_confident_match<'a>(
    query_title: &str,
    query_year: Option<i32>,
    candidates: &'a [TmdbMatch],
) -> Option<&'a TmdbMatch> {
    // 1. If query_year is known, filter candidates to those within ±1 year.
    //    If query_year is None, keep all candidates.
    // 2. Normalize both sides and keep candidates whose normalized title
    //    equals the normalized query.
    // 3. Return Some(c) only if exactly one survives.
}

fn normalize(raw: &str) -> String {
    // a. NFKD-fold (unicode-normalization)
    // b. Filter to ASCII alphanumeric + whitespace
    // c. Lowercase
    // d. Strip parenthetical disambiguators "(US)", "(UK)", "(2005)" etc.
    // e. Strip trailing/leading article "the", "a", "an"
    // f. Collapse whitespace
}
```

Pure functions, no I/O. Tested directly.

**Bias toward "needs review"** rather than auto-link: any ambiguity returns
`None`. Mismatched cast is annoying; re-linking from the picker is one
click.

## Worker

`src-tauri/src/metadata/worker.rs`. Spawned in `lib.rs::run` setup,
immediately after the DB pool is managed. Exposes its `Arc<Notify>` in
Tauri app state so:

- The scanner can wake it after enqueueing new jobs.
- The settings page can wake it when the user pastes / clears the API key.
- The "Refresh metadata" command can wake it after resetting a job row.

Main loop:

```
loop {
    let key = read_tmdb_key(&db).await?;
    if key.is_none() {
        notify.notified().await;       // wait for a settings change
        continue;
    }

    let Some(job) = next_due_job(&db).await? else {
        notify.notified().await;       // queue drained, wait for a signal
        continue;
    };

    let now = unix_now();
    if job.next_attempt_at > now {
        let wait = Duration::from_secs((job.next_attempt_at - now) as u64);
        tokio::select! {
            _ = sleep(wait) => {},
            _ = notify.notified() => {},
        }
        continue;
    }

    match run_job(&db, &app, &client, &key, &job).await {
        Ok(()) => {}                   // run_job deleted the row + wrote metadata in one tx
        Err(error) => record_failure(&db, &job, error).await?,
    }

    sleep(Duration::from_millis(250)).await;
}
```

### `run_job`

One sqlx transaction:

1. Re-read the row with `SELECT metadata_locked, title, year FROM <table> WHERE id = ?`.
   If `metadata_locked = 1`, delete the job row and return — user took
   ownership while we were queued.
2. Call `tmdb::search_movie` / `tmdb::search_show` with title + year.
3. Call `matching::pick_confident_match` on the results.
4. If `None`, leave the row unlinked, delete the job row, return success.
   This is the "needs review" path — the UI computes its needs-review list
   from `WHERE provider IS NULL AND NOT EXISTS (SELECT 1 FROM metadata_jobs ...)`.
5. If `Some(match)`, call `fetch_*_details` to get full payload.
6. Read current `poster_origin`. If it is not `'manual'`, download poster
   bytes to `<app_data>/posters/{kind}-{id}.<ext>`. If it is `'manual'`,
   skip the poster download — user owns the poster.
7. `UPDATE shows/movies SET provider = 'tmdb', provider_id = ?,
   overview = ?, year = ?, rating = ?, genres = ?, top_cast = ?,
   first_air_date = ? | runtime_minutes = ?, metadata_synced_at = now`.
   If the poster was downloaded in step 6, also set
   `poster_path = ?, poster_origin = 'tmdb'`. Title is **not** overwritten
   — the scanner-derived title is good enough and replacing it with TMDB's
   canonical title surprises users (e.g. "The Office (US)" replacing
   "The Office"). The text fields are overwritten unconditionally because
   `metadata_locked` would have aborted in step 1 if the user had edited
   anything.
8. `DELETE FROM metadata_jobs WHERE kind = ? AND media_id = ?`.
9. Commit.

Crash anywhere in steps 2–8 ⇒ tx rolls back ⇒ row remains in queue ⇒
next start picks it up.

### `record_failure`

- HTTP 401 (`AppError::Other` starting with `"auth_required:"`) ⇒
  `UPDATE metadata_jobs SET last_error = 'auth_required'`. **Do not**
  increment `attempts`. Worker filters parked rows out of `next_due_job`;
  they'll re-enter the rotation only after `notify.notify_one()` and a
  successful key check.
- HTTP 404 on search (badly tokenised query) ⇒ treat as "no match", same
  as step 4 above. Delete the job row.
- Anything else ⇒ exponential backoff:
  `attempts += 1`,
  `next_attempt_at = now + min(60 * 2^attempts, 3600)`,
  `last_error = err.to_string()`. Max 8 attempts. At `attempts = 8` the
  row stays in the queue as a dead letter — revivable only by an explicit
  "Refresh metadata" click that resets `attempts` to 0.

## Scanner integration

A single helper called from the existing `Detected::Movie` and
`Detected::Episode` arms in `src-tauri/src/scanner.rs`, in the same branch
that already increments `report.movies_added` / `report.shows_added`:

```rust
queries::enqueue_metadata_job(pool, "show", show_id).await?;
queries::enqueue_metadata_job(pool, "movie", movie_id).await?;
```

`enqueue_metadata_job` uses `INSERT … ON CONFLICT (kind, media_id) DO NOTHING`,
so re-enqueueing an existing row is a no-op. Combined with PR #17's
rescan-idempotency (`SELECT show_id FROM episodes WHERE path = ?`), this
means a rescan never re-fires sync for items the user has already let the
worker process.

After `scan_libraries` finishes, it calls `notify.notify_one()` once.

## API key & settings

A new settings page at `src/routes/settings/metadata/+page.svelte`:

- `Input` bound to the `tmdb_api_key` value in `app_settings`.
- Save button calls a new Tauri command `set_tmdb_api_key(key: String)`
  that writes to `app_settings` and fires `notify.notify_one()` on the
  worker.
- Small status block: `"X items pending, Y need review, Z failed"`, queried
  on page mount.
- A footer line: `"Metadata powered by [TMDB](https://www.themoviedb.org)"`
  — required by TMDB's terms when displaying their data.

The key is stored as plaintext in `app_settings`. Same threat model as the
rest of the SQLite DB. OS-keychain integration is a future hardening pass.

## UI surfaces

Three small surfaces on top of the existing pages:

1. **"Refresh metadata" + "Unlink" buttons** on
   `src/routes/series/[id]/edit/+page.svelte` and the equivalent movies
   edit page. Refresh resets the job row + clears `metadata_locked`.
   Unlink nulls the metadata columns and re-enqueues.
2. **"Needs review" badge** on the library pages (`/series`, `/films`)
   — a small counter linking to a list view.
3. **Needs-review list** at `src/routes/library/needs-review/+page.svelte`.
   Each row shows `{title, year, "Match…" button}`. Click opens a sheet
   that calls `tmdb::search_*` directly and renders the results; click a
   result to link it (writes provider+provider_id and re-enqueues the job,
   which the worker then finishes off-screen).

The needs-review sheet reuses the same shape as the manual-merge sheet
from PR #15.

## Errors

No new variants on `AppError`. Worker errors are opaque strings;
the worker is the sole consumer and inspects only the `"auth_required:"`
prefix.

User-facing errors surface through the derived UI: the failed-count badge
and a "Last error" line on the per-item edit page. The worker runs
silently otherwise — it never raises toasts or modal errors.

## Testing

Three layers:

1. **`metadata/matching.rs`** — full unit test coverage. Cases:
   - `("The Office", 2005, [US-2005, UK-2001]) → Some(US-2005)`
   - `("The Office", None, [US-2005, UK-2001]) → None` (ambiguous)
   - `("Pokémon", None, [Pokemon-1997]) → Some(Pokemon-1997)` (NFKD)
   - `("Foo", 2010, [Foo-2010, Foo-2011]) → None`
   - `("Foo", 2010, [Foo-2009]) → Some` (year ± 1)
   - `("Foo", 2010, [Foo-2008]) → None` (year > ±1)
   - parenthetical stripping, article stripping, whitespace collapse.
   ~10 cases in one `#[cfg(test)]` module.
2. **Worker state transitions** — light test using an in-memory sqlx pool:
   - Backoff math after N attempts.
   - 401 → parked, no attempts increment.
   - `metadata_locked = 1` → job deleted without work.
   No real HTTP.
3. **TMDB HTTP layer** — skip. Mocked-HTTP tests are high effort, low
   signal for a single-user app. Cover via manual verification.

### Manual verification

- Paste API key in Settings → worker drains queue → posters appear.
- Clear the key → in-flight 401 parks the job (no retry storm).
- Re-paste → parked jobs resume.
- Edit a title inline → that row gets `metadata_locked = 1` → re-run
  scan + worker → row is skipped.
- "Refresh metadata" on a locked row → re-fetches, lock cleared.
- Library with a known-ambiguous title (e.g. "The Office") → ends up in
  Needs review, not auto-linked.
- Kill app mid-fetch → restart → job runs to completion, no duplicate
  posters on disk.

## Rollout

Three sequential PRs. Each compiles and ships independently. Branch names
follow the project's `fix/N-slug` convention.

1. **`fix/21-metadata-scaffolding`**
   - Migration `0003_metadata_sync.sql`.
   - `app_settings` table + `get_tmdb_api_key` / `set_tmdb_api_key`
     Tauri commands.
   - Settings → Metadata page (input + save, no worker yet).
   - `metadata_locked` flip in the existing `update_show_metadata` /
     `update_movie_metadata` commands.
   - No network code. No new crates.

2. **`fix/22-tmdb-fetch`**
   - Adds `reqwest` + `unicode-normalization` deps.
   - `metadata/tmdb.rs`, `metadata/matching.rs`, plus the matcher unit
     tests.
   - `metadata/queries.rs` with `enqueue_metadata_job`, `next_due_job`,
     `record_failure`, `delete_metadata_job`, `apply_show_details`,
     `apply_movie_details`.
   - A temporary `fetch_metadata_now(kind, id)` Tauri command that runs
     the whole pipeline synchronously for one item, plus a "Sync now"
     button on the edit pages. This is the manual-verification harness.

3. **`fix/23-metadata-worker`**
   - `metadata/worker.rs` spawned in `lib.rs::run`.
   - Wired-up `Notify` in app state.
   - Scanner enqueues new items.
   - "Refresh metadata" + "Unlink" buttons replace the temporary "Sync
     now".
   - "Needs review" badge + list view + match sheet.
   - TMDB attribution footer.

PR 2 and PR 3 can each be reverted without affecting the schema in PR 1.

## Risks

- **Wrong auto-link.** A confident-match is occasionally wrong — TMDB has
  multiple records for some titles. Mitigation: "Unlink" + manual pick is
  always available. The matcher biases to `None` in any ambiguity.
- **TMDB API changes.** TMDB v3 is stable and widely used; v4 is opt-in.
  We pin to v3 endpoints. Breakage risk is low.
- **TMDB rate-limit policy change.** Currently no documented limit; we
  pace at 250ms between requests as cheap insurance. If TMDB tightens,
  we add a token bucket — not before.
- **User clears the key mid-sync.** Handled via the `'auth_required'`
  parking path.
- **Long backfills on huge libraries.** Worker is single-threaded and
  paces at 250ms. 1000 items ≈ 4 minutes minimum. Acceptable for a
  background process. If too slow in practice, raise the rate-limit cap.

## Open questions deferred to implementation

- Exactly which fields the worker overwrites vs preserves when re-syncing
  an item that was previously linked. Default plan: re-sync overwrites
  every metadata column except those touched by user edits (which are
  gated by `metadata_locked`). Detailed field list firms up in the
  implementation plan.
- Display order for `top_cast` on the detail page (use TMDB's `order`
  field, cap at 8).
- Whether the "Needs review" badge appears in the global sidebar or per
  library section. Default: per-section count on each list page.
