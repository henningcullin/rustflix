# Metadata Sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an automatic metadata sync subsystem that enriches scanned shows and movies with poster, overview, year, genres, rating, and top cast pulled from TMDB v3.

**Architecture:** Three sequential PRs. PR A (fix/22) lays schema + settings UI + the `metadata_locked` toggle but no network code. PR B (fix/23) adds the concrete TMDB client and a pure matcher plus a synchronous `fetch_metadata_now` command for one-item verification. PR C (fix/24) wraps the async worker around the queue, hooks the scanner up, and adds the user-facing "Refresh / Unlink / Needs review" surfaces. Each PR compiles and ships independently.

**Tech Stack:** Rust (Tauri 2, sqlx 0.8 with bundled SQLite, reqwest 0.12 with rustls-tls, unicode-normalization 0.1). SvelteKit 5 (Svelte 5 runes) + shadcn-svelte (bits-ui). All work follows the project's `fix/N-slug` branch convention (one branch = one PR = one logical change); spec at `docs/superpowers/specs/2026-05-24-metadata-sync-design.md`.

> Branch numbering note: the spec text mentions fix/21–23 for the three PRs, but fix/21 was used for the spec doc itself. Actual branches in this plan are **fix/22**, **fix/23**, **fix/24**.

---

## File Structure

### PR A — fix/22-metadata-scaffolding

- **Create**
  - `src-tauri/migrations/0003_metadata_sync.sql` — adds the new columns on `shows`/`movies`, the `metadata_jobs` queue table, the `app_settings` key/value table, and partial UNIQUE indexes on `(provider, provider_id)`.
  - `src/routes/settings/metadata/+page.svelte` — Settings page with the TMDB API-key input and a per-status counts panel (populated from a single command).
- **Modify**
  - `src-tauri/src/queries.rs` — add `get_app_setting`, `set_app_setting`, `count_metadata_jobs_by_status`. Flip `metadata_locked = 1` inside `update_show_metadata` and `update_movie_metadata`.
  - `src-tauri/src/commands.rs` — add `get_tmdb_api_key`, `set_tmdb_api_key`, `metadata_status_counts` Tauri commands.
  - `src-tauri/src/lib.rs` — register the three new commands.
  - `src-tauri/src/models.rs` — extend `Show`/`Movie` structs with the new nullable columns; add `MetadataStatusCounts` struct.
  - `src-tauri/src/queries.rs` — update `MOVIE_SELECT` / `SHOW_SELECT` constants to include the new columns.
  - `src/lib/api.ts` — extend `Show`/`Movie` types; add `getTmdbApiKey`, `setTmdbApiKey`, `metadataStatusCounts` to the `api` object; add `MetadataStatusCounts` interface.

### PR B — fix/23-tmdb-fetch

- **Create**
  - `src-tauri/src/metadata/mod.rs` — module root, re-exports.
  - `src-tauri/src/metadata/tmdb.rs` — concrete TMDB client (`search_movie`, `search_show`, `fetch_movie_details`, `fetch_show_details`, `download_poster`).
  - `src-tauri/src/metadata/matching.rs` — `pick_confident_match`, `normalize`, plus full unit-test coverage.
  - `src-tauri/src/metadata/apply.rs` — `apply_show_details`, `apply_movie_details` (DB write helpers).
- **Modify**
  - `src-tauri/Cargo.toml` — add `reqwest = "0.12"` (rustls) and `unicode-normalization = "0.1"`.
  - `src-tauri/src/lib.rs` — add `mod metadata;`, build a shared `reqwest::Client` at startup, register the new `fetch_metadata_now` command.
  - `src-tauri/src/commands.rs` — add `fetch_metadata_now(kind, id)` command (synchronous, single-item, for manual verification).
  - `src/lib/api.ts` — add `fetchMetadataNow`.
  - `src/routes/series/[id]/edit/+page.svelte` and `src/routes/films/[id]/edit/+page.svelte` (or movies equivalent — verify path) — add a temporary "Sync now" button calling `api.fetchMetadataNow`.

### PR C — fix/24-metadata-worker

- **Create**
  - `src-tauri/src/metadata/worker.rs` — async loop, `Notify`-driven, drains `metadata_jobs`.
  - `src-tauri/src/metadata/queries.rs` — `enqueue_metadata_job`, `next_due_job`, `record_failure`, `delete_metadata_job`.
  - `src/routes/library/needs-review/+page.svelte` — the unlinked-items list view.
  - `src/lib/components/MetadataMatchSheet.svelte` — sheet that fires TMDB search and lets the user pick a result.
- **Modify**
  - `src-tauri/src/lib.rs` — spawn the worker; expose `Arc<Notify>` via Tauri state.
  - `src-tauri/src/scanner.rs` — enqueue on new `show_id` / `movie_id` insertions; signal the notify after the scan loop.
  - `src-tauri/src/commands.rs` — add `refresh_metadata`, `unlink_metadata`, `metadata_search`, `link_metadata` commands. Drop the temporary `fetch_metadata_now`.
  - `src-tauri/src/lib.rs` — register the four new commands; deregister `fetch_metadata_now`.
  - `src/routes/settings/metadata/+page.svelte` — replace any "Sync now" stub references; the status counts panel is real now.
  - `src/routes/series/[id]/edit/+page.svelte` and movies edit page — replace the temporary "Sync now" with "Refresh metadata" + "Unlink".
  - `src/routes/series/+page.svelte` and `src/routes/films/+page.svelte` — add a "Needs review" badge linking to the list view.
  - `src/lib/api.ts` — replace `fetchMetadataNow` with the four new methods.

---

## PR A — fix/22-metadata-scaffolding

Self-contained: schema + Settings page + the `metadata_locked` toggle. No network code, no new crates.

### Task A.0: Create branch from latest master

**Files:** none

- [ ] **Step 1: Sync master**

Run:
```bash
git checkout master && git pull --ff-only && git status --porcelain
```
Expected: clean working tree, on master.

- [ ] **Step 2: Create the branch**

Run:
```bash
git checkout -b fix/22-metadata-scaffolding
```
Expected: branch created.

---

### Task A.1: Add migration `0003_metadata_sync.sql`

**Files:**
- Create: `src-tauri/migrations/0003_metadata_sync.sql`

- [ ] **Step 1: Write the migration file**

Create `src-tauri/migrations/0003_metadata_sync.sql` with this exact content:

```sql
-- Adds the metadata sync columns on shows and movies plus the job-queue and
-- app-settings tables. Pure ALTER TABLE ADD COLUMN + CREATE TABLE / INDEX,
-- so no table rebuild is needed.

ALTER TABLE shows  ADD COLUMN provider           TEXT;
ALTER TABLE shows  ADD COLUMN provider_id        TEXT;
ALTER TABLE shows  ADD COLUMN rating             REAL;
ALTER TABLE shows  ADD COLUMN genres             TEXT;
ALTER TABLE shows  ADD COLUMN top_cast           TEXT;
ALTER TABLE shows  ADD COLUMN first_air_date     TEXT;
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
    kind            TEXT    NOT NULL,
    media_id        INTEGER NOT NULL,
    enqueued_at     INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    attempts        INTEGER NOT NULL DEFAULT 0,
    last_error      TEXT,
    next_attempt_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    PRIMARY KEY (kind, media_id)
);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

- [ ] **Step 2: Verify the migration syntax compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: `Finished \`dev\` profile [unoptimized + debuginfo] target(s)` — `sqlx::migrate!` is a compile-time macro that parses every `.sql` in the migrations dir, so a syntax error fails the build here.

(The `touch`/`rm` dance stubs the Linux mpv binary that the Tauri build script expects but that the WSL dev environment doesn't ship. The stub never gets committed because `src-tauri/binaries/*-linux-gnu` isn't tracked.)

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/migrations/0003_metadata_sync.sql
git commit -m "feat(db): add metadata sync schema (columns, queue, settings)"
```

---

### Task A.2: Extend `Show` / `Movie` Rust models

**Files:**
- Modify: `src-tauri/src/models.rs`

- [ ] **Step 1: Add the new fields to `Movie`**

In `src-tauri/src/models.rs`, replace the `Movie` struct with:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Movie {
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub path: String,
    pub poster_path: Option<String>,
    pub poster_origin: Option<String>,
    pub overview: Option<String>,
    pub duration_seconds: Option<i64>,
    pub progress_seconds: i64,
    pub watched: bool,
    pub added_at: i64,
    pub provider: Option<String>,
    pub provider_id: Option<String>,
    pub rating: Option<f64>,
    pub genres: Option<String>,
    pub top_cast: Option<String>,
    pub runtime_minutes: Option<i64>,
    pub metadata_synced_at: Option<i64>,
    pub metadata_locked: i64,
}
```

- [ ] **Step 2: Add the new fields to `Show`**

Replace the `Show` struct with:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct Show {
    pub id: i64,
    pub library_id: i64,
    pub title: String,
    pub year: Option<i32>,
    pub folder_path: String,
    pub fingerprint: String,
    pub poster_path: Option<String>,
    pub poster_origin: Option<String>,
    pub overview: Option<String>,
    pub episode_count: i64,
    pub watched_count: i64,
    pub added_at: i64,
    pub provider: Option<String>,
    pub provider_id: Option<String>,
    pub rating: Option<f64>,
    pub genres: Option<String>,
    pub top_cast: Option<String>,
    pub first_air_date: Option<String>,
    pub metadata_synced_at: Option<i64>,
    pub metadata_locked: i64,
}
```

- [ ] **Step 3: Add `MetadataStatusCounts` struct**

Append to the end of `src-tauri/src/models.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct MetadataStatusCounts {
    pub pending: i64,        // queued, attempts == 0, not parked
    pub failed: i64,         // queued, attempts > 0, not parked, not dead-letter
    pub auth_required: i64,  // parked on 401
    pub dead_letter: i64,    // attempts >= 8
    pub needs_review: i64,   // provider IS NULL AND no row in metadata_jobs
}
```

- [ ] **Step 4: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -20 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build. Note: `sqlx::FromRow` derive needs every column in `SELECT *` paths — but our queries use explicit column lists (`MOVIE_SELECT`, `SHOW_SELECT`), so we'll update those in the next task.

---

### Task A.3: Update `MOVIE_SELECT` / `SHOW_SELECT` to include new columns

**Files:**
- Modify: `src-tauri/src/queries.rs`

- [ ] **Step 1: Update `MOVIE_SELECT`**

Replace the `MOVIE_SELECT` constant near the top of `src-tauri/src/queries.rs` with:

```rust
const MOVIE_SELECT: &str = "
    SELECT m.id, m.title, m.year, m.path, m.poster_path, m.poster_origin, m.overview,
           m.duration_seconds,
           COALESCE(w.progress_seconds, 0) AS progress_seconds,
           COALESCE(w.watched, 0) AS watched,
           m.added_at,
           m.provider, m.provider_id, m.rating, m.genres, m.top_cast,
           m.runtime_minutes, m.metadata_synced_at, m.metadata_locked
    FROM movies m
    LEFT JOIN watch_history w
      ON w.media_kind = 'movie' AND w.media_id = m.id
";
```

- [ ] **Step 2: Update `SHOW_SELECT`**

Replace the `SHOW_SELECT` constant with:

```rust
const SHOW_SELECT: &str = "
    SELECT s.id, s.library_id, s.title, s.year, s.folder_path, s.fingerprint,
           s.poster_path, s.poster_origin, s.overview,
           (SELECT COUNT(*) FROM episodes e WHERE e.show_id = s.id) AS episode_count,
           (SELECT COUNT(*) FROM episodes e
              LEFT JOIN watch_history w
                ON w.media_kind = 'episode' AND w.media_id = e.id
             WHERE e.show_id = s.id AND COALESCE(w.watched, 0) = 1) AS watched_count,
           s.added_at,
           s.provider, s.provider_id, s.rating, s.genres, s.top_cast,
           s.first_air_date, s.metadata_synced_at, s.metadata_locked
    FROM shows s
";
```

- [ ] **Step 3: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 4: Commit Tasks A.2 + A.3 together**

Run:
```bash
git add src-tauri/src/models.rs src-tauri/src/queries.rs
git commit -m "feat(models): extend Show/Movie with metadata sync columns"
```

---

### Task A.4: Flip `metadata_locked = 1` inside the text-edit mutations

**Files:**
- Modify: `src-tauri/src/queries.rs:167-230` (the two `update_*_metadata` functions)

- [ ] **Step 1: Find the current `update_show_metadata`**

Run:
```bash
grep -n "pub async fn update_show_metadata\|pub async fn update_movie_metadata" src-tauri/src/queries.rs
```
Expected: two line numbers around 167–200.

- [ ] **Step 2: Update `update_show_metadata` to set the lock**

Inside the existing `update_show_metadata` function in `src-tauri/src/queries.rs`, change the dynamically-built `UPDATE shows SET ...` query to also set `metadata_locked = 1` whenever any field is being updated. The existing helper builds the SET clause from `Option` fields; locate the line where the helper returns (or the place the UPDATE is executed) and append `metadata_locked = 1` to the SET clause. The function signature stays the same.

Concretely: the existing code uses `update_metadata_row(pool, "shows", id, title, year, overview)`. Find `update_metadata_row` (also in `queries.rs`) and modify its built SQL string so that after appending all dynamic fields it appends `, metadata_locked = 1`. The SET clause always has at least one comma at this point (because the helper only runs when at least one field is provided), so always appending `, metadata_locked = 1` is safe — but verify by reading the helper.

If the helper instead early-returns on "no fields to update," ensure that early-return path also runs an `UPDATE shows SET metadata_locked = 1 WHERE id = ?` so that a no-op text edit still records the user's intent to own the row. Actually — re-think: a no-op edit (user opened the form and saved without changing anything) probably shouldn't lock. So leave the early-return path untouched, only adding the `metadata_locked = 1` to the real-update path.

Pseudo-diff of the helper:
```rust
// Before
let sql = format!("UPDATE {table} SET {set_clause} WHERE id = ?{n}");

// After
let sql = format!("UPDATE {table} SET {set_clause}, metadata_locked = 1 WHERE id = ?{n}");
```

`{set_clause}` already contains at least one assignment, so the extra `,` is always valid.

- [ ] **Step 3: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 4: Commit**

Run:
```bash
git add src-tauri/src/queries.rs
git commit -m "feat(metadata): lock items from sync when user edits text fields"
```

---

### Task A.5: Add `app_settings` helpers in `queries.rs`

**Files:**
- Modify: `src-tauri/src/queries.rs`

- [ ] **Step 1: Append the helpers to the bottom of `queries.rs`**

Add at the end of `src-tauri/src/queries.rs`:

```rust
pub async fn get_app_setting(pool: &SqlitePool, key: &str) -> AppResult<Option<String>> {
    let value: Option<String> =
        sqlx::query_scalar("SELECT value FROM app_settings WHERE key = ?1")
            .bind(key)
            .fetch_optional(pool)
            .await?;

    Ok(value)
}

pub async fn set_app_setting(pool: &SqlitePool, key: &str, value: &str) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_app_setting(pool: &SqlitePool, key: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM app_settings WHERE key = ?1")
        .bind(key)
        .execute(pool)
        .await?;

    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

---

### Task A.6: Add `count_metadata_jobs_by_status` query

**Files:**
- Modify: `src-tauri/src/queries.rs`

- [ ] **Step 1: Add the count function**

Append to `src-tauri/src/queries.rs`:

```rust
pub async fn metadata_status_counts(
    pool: &SqlitePool,
) -> AppResult<crate::models::MetadataStatusCounts> {
    let pending: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs
         WHERE attempts = 0 AND COALESCE(last_error, '') <> 'auth_required'",
    )
    .fetch_one(pool)
    .await?;

    let failed: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs
         WHERE attempts > 0 AND attempts < 8
           AND COALESCE(last_error, '') <> 'auth_required'",
    )
    .fetch_one(pool)
    .await?;

    let auth_required: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE last_error = 'auth_required'",
    )
    .fetch_one(pool)
    .await?;

    let dead_letter: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM metadata_jobs WHERE attempts >= 8",
    )
    .fetch_one(pool)
    .await?;

    // "Needs review" = item that finished its sync attempt without finding a
    // match. We can't distinguish that today (the job row gets deleted on
    // both success and no-match). In PR C we'll add a marker; for now keep
    // this as 0 so the UI compiles.
    let needs_review: i64 = 0;

    Ok(crate::models::MetadataStatusCounts {
        pending,
        failed,
        auth_required,
        dead_letter,
        needs_review,
    })
}
```

- [ ] **Step 2: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 3: Commit Tasks A.5 + A.6 together**

Run:
```bash
git add src-tauri/src/queries.rs
git commit -m "feat(queries): app_settings helpers + metadata status counts"
```

---

### Task A.7: Add `get_tmdb_api_key` / `set_tmdb_api_key` / `metadata_status_counts` Tauri commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add commands to `commands.rs`**

Append to `src-tauri/src/commands.rs` (just before the existing `copy_poster` helper at the bottom):

```rust
#[tauri::command]
pub async fn get_tmdb_api_key(db: State<'_, Db>) -> AppResult<Option<String>> {
    queries::get_app_setting(&db, "tmdb_api_key").await
}

#[tauri::command]
pub async fn set_tmdb_api_key(db: State<'_, Db>, key: String) -> AppResult<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        queries::delete_app_setting(&db, "tmdb_api_key").await
    } else {
        queries::set_app_setting(&db, "tmdb_api_key", trimmed).await
    }
}

#[tauri::command]
pub async fn metadata_status_counts(
    db: State<'_, Db>,
) -> AppResult<crate::models::MetadataStatusCounts> {
    queries::metadata_status_counts(&db).await
}
```

Also add the import at the top if not already present:
```rust
use crate::models::MetadataStatusCounts;  // only if not already imported transitively
```

- [ ] **Step 2: Register the three commands in `lib.rs`**

Open `src-tauri/src/lib.rs` and add three lines to the `generate_handler![...]` macro list, right after `commands::reset_movie_poster`:

```rust
commands::get_tmdb_api_key,
commands::set_tmdb_api_key,
commands::metadata_status_counts,
```

- [ ] **Step 3: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 4: Commit**

Run:
```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): TMDB key + metadata status count commands"
```

---

### Task A.8: Extend the frontend API

**Files:**
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Add types**

In `src/lib/api.ts`, after the existing `MetadataPatch` interface (around line 80), add:

```ts
export interface MetadataStatusCounts {
  pending: number;
  failed: number;
  auth_required: number;
  dead_letter: number;
  needs_review: number;
}
```

- [ ] **Step 2: Extend `Show` interface**

Add to the `Show` interface (after `added_at: number;`):

```ts
  provider: string | null;
  provider_id: string | null;
  rating: number | null;
  genres: string | null;
  top_cast: string | null;
  first_air_date: string | null;
  metadata_synced_at: number | null;
  metadata_locked: number;
```

- [ ] **Step 3: Extend `Movie` interface**

Add to the `Movie` interface (after `added_at: number;`):

```ts
  provider: string | null;
  provider_id: string | null;
  rating: number | null;
  genres: string | null;
  top_cast: string | null;
  runtime_minutes: number | null;
  metadata_synced_at: number | null;
  metadata_locked: number;
```

- [ ] **Step 4: Add API methods**

Add to the `api` object literal (alongside the other show/movie mutations):

```ts
  getTmdbApiKey: () => invoke<string | null>('get_tmdb_api_key'),
  setTmdbApiKey: (key: string) => invoke<void>('set_tmdb_api_key', { key }),
  metadataStatusCounts: () =>
    invoke<MetadataStatusCounts>('metadata_status_counts'),
```

- [ ] **Step 5: Verify the project still type-checks**

Run from repo root:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error in `src/routes/series/[id]/+page.svelte` (the `MergeShowSheet` `onMerged` type issue from prior PRs). No new errors.

- [ ] **Step 6: Commit**

Run:
```bash
git add src/lib/api.ts
git commit -m "feat(api): metadata sync types and command bindings"
```

---

### Task A.9: Build the Settings → Metadata page

**Files:**
- Create: `src/routes/settings/metadata/+page.svelte`

- [ ] **Step 1: Write the page**

Create `src/routes/settings/metadata/+page.svelte` with:

```svelte
<script lang="ts">
  import { api, type MetadataStatusCounts } from '$lib/api';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { Input } from '$lib/components/ui/input';

  let keyDraft = $state('');
  let savedKey = $state<string | null>(null);
  let saving = $state(false);
  let counts = $state<MetadataStatusCounts | null>(null);
  let error = $state<string | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      [savedKey, counts] = await Promise.all([
        api.getTmdbApiKey(),
        api.metadataStatusCounts(),
      ]);
      keyDraft = savedKey ?? '';
    } catch (caught) {
      error = String(caught);
    }
  }

  async function saveKey() {
    saving = true;
    error = null;
    try {
      await api.setTmdbApiKey(keyDraft);
      savedKey = keyDraft.trim().length === 0 ? null : keyDraft.trim();
    } catch (caught) {
      error = String(caught);
    } finally {
      saving = false;
    }
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <header class="mb-6">
    <h1 class="text-3xl font-bold tracking-tight">Metadata</h1>
    <p class="text-sm text-muted-foreground">
      Rustflix can fetch posters, overviews, genres, ratings, and cast from TMDB.
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
        <CardTitle>TMDB API key</CardTitle>
        <CardDescription>
          Sign up at <a class="underline" href="https://www.themoviedb.org/settings/api">themoviedb.org</a>
          and paste your v3 API key here. Without a key, metadata sync is paused.
        </CardDescription>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <Input
          bind:value={keyDraft}
          placeholder="Paste your TMDB v3 API key"
          type="password"
        />
        <div class="flex items-center gap-3">
          <Button onclick={saveKey} disabled={saving}>
            {saving ? 'Saving…' : savedKey ? 'Update key' : 'Save key'}
          </Button>
          {#if savedKey}
            <span class="text-xs text-muted-foreground">
              A key is currently stored.
            </span>
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
          <ul class="grid grid-cols-2 gap-3 text-sm sm:grid-cols-5">
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-muted-foreground text-xs uppercase tracking-wide">Pending</div>
              <div class="text-lg font-semibold">{counts.pending}</div>
            </li>
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-muted-foreground text-xs uppercase tracking-wide">Failed</div>
              <div class="text-lg font-semibold">{counts.failed}</div>
            </li>
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-muted-foreground text-xs uppercase tracking-wide">Auth-paused</div>
              <div class="text-lg font-semibold">{counts.auth_required}</div>
            </li>
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-muted-foreground text-xs uppercase tracking-wide">Dead-letter</div>
              <div class="text-lg font-semibold">{counts.dead_letter}</div>
            </li>
            <li class="rounded-md border border-border bg-background px-3 py-2">
              <div class="text-muted-foreground text-xs uppercase tracking-wide">Needs review</div>
              <div class="text-lg font-semibold">{counts.needs_review}</div>
            </li>
          </ul>
        {:else}
          <p class="text-muted-foreground text-sm">Loading…</p>
        {/if}
      </CardContent>
    </Card>

    <p class="text-xs text-muted-foreground">
      Metadata powered by <a class="underline" href="https://www.themoviedb.org">TMDB</a>.
    </p>
  </div>
</div>
```

- [ ] **Step 2: Verify it type-checks**

Run from repo root:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error, no new ones.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/routes/settings/metadata/+page.svelte
git commit -m "feat(settings): Metadata settings page (API key + status counts)"
```

---

### Task A.10: Wire the new page into the Settings nav

**Files:**
- Modify: `src/routes/settings/+layout.svelte` (or the file that holds the Settings sidebar / nav)

- [ ] **Step 1: Locate the Settings nav**

Run:
```bash
ls src/routes/settings/ && grep -rn "settings/libraries" src/routes/settings/ 2>/dev/null | head -5
```
Identify the file that contains the link to `/settings/libraries`. If `/settings` has a `+layout.svelte` with a sidebar, add the new link there. If each sub-page is freestanding (no layout), add a link from `/settings/libraries` to `/settings/metadata` for quick access, or add `/settings/+page.svelte` if it doesn't exist that lists both.

- [ ] **Step 2: Add a "Metadata" entry next to "Libraries"**

Wherever the "Libraries" link lives, add a sibling link to `/settings/metadata`. If there is no nav at all (i.e., `/settings/libraries` is the only sub-page and the user reaches it directly), create `src/routes/settings/+page.svelte` as an index listing both sub-pages.

If creating `src/routes/settings/+page.svelte`:

```svelte
<script lang="ts">
  import { ChevronRight } from '$lib/lucide';
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <h1 class="mb-6 text-3xl font-bold tracking-tight">Settings</h1>
  <ul class="flex flex-col gap-2">
    <li>
      <a
        href="/settings/libraries"
        class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3 transition-colors hover:bg-accent"
      >
        <div>
          <div class="font-medium">Libraries</div>
          <div class="text-sm text-muted-foreground">
            Folders Rustflix scans for movies and series.
          </div>
        </div>
        <ChevronRight class="size-4 text-muted-foreground" />
      </a>
    </li>
    <li>
      <a
        href="/settings/metadata"
        class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3 transition-colors hover:bg-accent"
      >
        <div>
          <div class="font-medium">Metadata</div>
          <div class="text-sm text-muted-foreground">
            TMDB integration for posters, cast, and overviews.
          </div>
        </div>
        <ChevronRight class="size-4 text-muted-foreground" />
      </a>
    </li>
  </ul>
</div>
```

- [ ] **Step 3: Verify it type-checks**

Run from repo root:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/routes/settings/
git commit -m "feat(settings): expose Metadata page in the settings index"
```

---

### Task A.11: Push, open PR, merge

**Files:** none

- [ ] **Step 1: Push the branch**

Run:
```bash
git push -u origin fix/22-metadata-scaffolding
```

- [ ] **Step 2: Open the PR**

Run:
```bash
gh pr create --title "Metadata sync scaffolding: schema, settings page, edit-lock" --body "$(cat <<'EOF'
## Summary
- Add migration 0003_metadata_sync.sql introducing the metadata columns on shows/movies, the metadata_jobs queue table, the app_settings key/value table, and partial UNIQUE indexes on (provider, provider_id).
- Extend the Show/Movie Rust models and SHOW_SELECT/MOVIE_SELECT to include the new columns.
- Flip metadata_locked = 1 inside update_show_metadata / update_movie_metadata so user text edits protect the row from later TMDB sync.
- Add get_tmdb_api_key / set_tmdb_api_key / metadata_status_counts Tauri commands.
- Add Settings → Metadata page with TMDB API key input and a counts panel.
- Wire the new page into the Settings index.

No network code, no new crates, no worker. Spec at docs/superpowers/specs/2026-05-24-metadata-sync-design.md.

## Test plan
- [ ] Migration applies cleanly on a fresh DB and on a DB previously at migration 0002.
- [ ] Visit /settings/metadata, save a key, reload — key persists.
- [ ] Edit a show title inline → SELECT metadata_locked FROM shows WHERE id = ? returns 1.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Merge**

Run:
```bash
gh pr merge <PR-NUMBER> --merge --delete-branch && git checkout master && git pull --ff-only
```

---

## PR B — fix/23-tmdb-fetch

TMDB client + matcher + a synchronous `fetch_metadata_now` command for one-item manual verification. Heavy TDD on `matching.rs`.

### Task B.0: Create branch

**Files:** none

- [ ] **Step 1: Branch from latest master**

Run:
```bash
git checkout master && git pull --ff-only && git checkout -b fix/23-tmdb-fetch
```

---

### Task B.1: Add `reqwest` and `unicode-normalization` to `Cargo.toml`

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add the dependencies**

In `src-tauri/Cargo.toml`, under `[dependencies]`, add:

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "stream"] }
unicode-normalization = "0.1"
```

- [ ] **Step 2: Build to fetch deps**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo build 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: dependencies download, build succeeds. First build may take several minutes (`reqwest` + `rustls` is a chunky tree).

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "build: add reqwest and unicode-normalization"
```

---

### Task B.2: Create the `metadata` module shell

**Files:**
- Create: `src-tauri/src/metadata/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create `mod.rs`**

Create `src-tauri/src/metadata/mod.rs`:

```rust
//! Metadata sync subsystem. See
//! docs/superpowers/specs/2026-05-24-metadata-sync-design.md.

pub mod apply;
pub mod matching;
pub mod tmdb;
```

- [ ] **Step 2: Declare the module in `lib.rs`**

In `src-tauri/src/lib.rs`, add `mod metadata;` near the top with the other `mod` declarations.

- [ ] **Step 3: Verify (will fail until apply/matching/tmdb files exist; that's fine)**

We'll add the three files in the next tasks. No build step here.

---

### Task B.3 (TDD): `metadata/matching.rs` — write failing tests first

**Files:**
- Create: `src-tauri/src/metadata/matching.rs`

- [ ] **Step 1: Write the module with stubs + tests**

Create `src-tauri/src/metadata/matching.rs`:

```rust
//! Pure matching logic. Given a scanner-derived title + year and a list of
//! provider search results, decide whether one is a confident match.

use unicode_normalization::UnicodeNormalization;

/// One TMDB-ish search result. Provider-neutral on purpose so the matcher
/// stays unit-testable without touching the TMDB module.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    let folded: String = raw.nfkd().filter(|c| c.is_ascii()).collect();
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
    for ch in input.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ if depth == 0 => output.push(ch),
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

    fn c(id: &str, title: &str, year: Option<i32>) -> MatchCandidate {
        MatchCandidate {
            provider_id: id.to_string(),
            title: title.to_string(),
            year,
        }
    }

    #[test]
    fn unique_match_returns_it() {
        let candidates = vec![c("1", "Breaking Bad", Some(2008))];
        let picked = pick_confident_match("Breaking Bad", Some(2008), &candidates);
        assert_eq!(picked.map(|m| m.provider_id.as_str()), Some("1"));
    }

    #[test]
    fn the_office_us_vs_uk_resolved_by_year() {
        let candidates = vec![
            c("us", "The Office (US)", Some(2005)),
            c("uk", "The Office", Some(2001)),
        ];
        let picked = pick_confident_match("The Office", Some(2005), &candidates);
        assert_eq!(picked.map(|m| m.provider_id.as_str()), Some("us"));
    }

    #[test]
    fn ambiguous_without_year_returns_none() {
        let candidates = vec![
            c("us", "The Office (US)", Some(2005)),
            c("uk", "The Office", Some(2001)),
        ];
        let picked = pick_confident_match("The Office", None, &candidates);
        assert!(picked.is_none());
    }

    #[test]
    fn nfkd_fold_pokemon() {
        let candidates = vec![c("1", "Pokemon", Some(1997))];
        let picked = pick_confident_match("Pokémon", None, &candidates);
        assert_eq!(picked.map(|m| m.provider_id.as_str()), Some("1"));
    }

    #[test]
    fn year_plus_minus_one_accepted() {
        let candidates = vec![c("1", "Foo", Some(2009))];
        let picked = pick_confident_match("Foo", Some(2010), &candidates);
        assert!(picked.is_some());
    }

    #[test]
    fn year_more_than_one_off_rejected() {
        let candidates = vec![c("1", "Foo", Some(2008))];
        let picked = pick_confident_match("Foo", Some(2010), &candidates);
        assert!(picked.is_none());
    }

    #[test]
    fn two_matches_in_same_year_window_returns_none() {
        let candidates = vec![
            c("1", "Foo", Some(2010)),
            c("2", "Foo", Some(2011)),
        ];
        let picked = pick_confident_match("Foo", Some(2010), &candidates);
        assert!(picked.is_none());
    }

    #[test]
    fn leading_article_stripped() {
        let candidates = vec![c("1", "The Matrix", Some(1999))];
        let picked = pick_confident_match("Matrix", Some(1999), &candidates);
        assert!(picked.is_some());
    }

    #[test]
    fn parenthetical_year_stripped() {
        let candidates = vec![c("1", "Dune (2021)", Some(2021))];
        let picked = pick_confident_match("Dune", Some(2021), &candidates);
        assert!(picked.is_some());
    }

    #[test]
    fn whitespace_collapsed() {
        let candidates = vec![c("1", "Foo   Bar", Some(2010))];
        let picked = pick_confident_match("Foo Bar", Some(2010), &candidates);
        assert!(picked.is_some());
    }
}
```

- [ ] **Step 2: Run the tests — expect them to PASS (the implementation is already there)**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo test metadata::matching 2>&1 | tail -20 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 10 passing tests. If any fail, fix the relevant helper (`strip_parenthetical`, `strip_leading_article`, etc.) and re-run.

Note: this isn't strict TDD — we wrote the implementation in the same file as the tests because the matcher is one tightly-knit unit. The TDD spirit is preserved: the tests are written, runnable, and the implementation must satisfy them. If you prefer red-then-green, stub each function to `unimplemented!()` first, watch the tests fail, then paste in the implementation.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/metadata/mod.rs src-tauri/src/metadata/matching.rs
git commit -m "feat(metadata): pure matcher with normalize + tests"
```

---

### Task B.4: TMDB client — search

**Files:**
- Create: `src-tauri/src/metadata/tmdb.rs`

- [ ] **Step 1: Create the file with search functions**

Create `src-tauri/src/metadata/tmdb.rs`:

```rust
//! TMDB v3 client. Stays narrow: only the calls the worker needs.
//! Returns concrete TMDB structs; the matcher converts them to its own
//! provider-neutral shape.

use std::path::Path;

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;

use crate::error::{AppError, AppResult};
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
    pub poster_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TmdbShowResult {
    pub id: i64,
    pub name: String,
    pub first_air_date: Option<String>,
    pub poster_path: Option<String>,
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
        request = request.query(&[("year", year_value.to_string().as_str())]);
    }

    let response = request.send().await.map_err(http_err)?;
    let envelope: SearchEnvelope<TmdbMovieResult> =
        parse_response(response, "search/movie").await?;

    Ok(envelope
        .results
        .into_iter()
        .map(|raw| MatchCandidate {
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
        request = request.query(&[("first_air_date_year", year_value.to_string().as_str())]);
    }

    let response = request.send().await.map_err(http_err)?;
    let envelope: SearchEnvelope<TmdbShowResult> =
        parse_response(response, "search/tv").await?;

    Ok(envelope
        .results
        .into_iter()
        .map(|raw| MatchCandidate {
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
/// `dest`. Streams the response body to disk to keep memory bounded.
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
    date.and_then(|d| d.get(0..4)).and_then(|y| y.parse().ok())
}

/// Public so the apply layer can build the local poster filename extension.
pub fn poster_extension(poster_path: &str) -> &str {
    poster_path
        .rsplit_once('.')
        .map(|(_, ext)| ext)
        .unwrap_or("jpg")
}
```

- [ ] **Step 2: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build (apply.rs is still missing; if `metadata/mod.rs` references it, comment that line out for now and re-add after Task B.5).

Workaround if `cargo check` errors on missing `apply`: temporarily edit `src-tauri/src/metadata/mod.rs` to remove `pub mod apply;` for this step, then restore it in Task B.5.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/metadata/tmdb.rs src-tauri/src/metadata/mod.rs
git commit -m "feat(metadata): TMDB v3 client (search, details, poster download)"
```

---

### Task B.5: `metadata/apply.rs` — write fetched details to the DB

**Files:**
- Create: `src-tauri/src/metadata/apply.rs`

- [ ] **Step 1: Create the file**

Create `src-tauri/src/metadata/apply.rs`:

```rust
//! Pure DB writes for a fetched-and-matched TMDB payload. Called inside
//! the worker's per-job transaction. Never overwrites a manual poster;
//! never runs when metadata_locked = 1 (caller's responsibility to check).

use sqlx::SqliteConnection;

use crate::error::AppResult;
use crate::metadata::tmdb::{TmdbCastMember, TmdbMovieDetails, TmdbShowDetails};

/// Apply a fetched movie payload onto an existing `movies` row. Returns the
/// optional poster filename (extension) the caller should download to, or
/// None if the current row already has a manual poster.
pub async fn apply_movie_details(
    conn: &mut SqliteConnection,
    movie_id: i64,
    details: &TmdbMovieDetails,
) -> AppResult<Option<String>> {
    let current_poster_origin: Option<String> = sqlx::query_scalar(
        "SELECT poster_origin FROM movies WHERE id = ?1",
    )
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

    let new_poster_clause = if download_extension.is_some() {
        ", poster_path = ?9, poster_origin = 'tmdb'"
    } else {
        ""
    };

    let sql = format!(
        "UPDATE movies SET
             provider = 'tmdb',
             provider_id = ?1,
             overview = ?2,
             year = ?3,
             rating = ?4,
             genres = ?5,
             top_cast = ?6,
             runtime_minutes = ?7,
             metadata_synced_at = strftime('%s','now')
             {new_poster_clause}
         WHERE id = ?8"
    );

    let mut query = sqlx::query(&sql)
        .bind(details.id.to_string())
        .bind(details.overview.as_deref())
        .bind(year)
        .bind(details.vote_average)
        .bind(genres_json)
        .bind(cast_json)
        .bind(details.runtime)
        .bind(movie_id);

    if let Some(extension) = download_extension.as_ref() {
        let local_path = format!("movie-{movie_id}.{extension}");
        query = query.bind(local_path);
    }

    query.execute(&mut *conn).await?;

    Ok(download_extension)
}

pub async fn apply_show_details(
    conn: &mut SqliteConnection,
    show_id: i64,
    details: &TmdbShowDetails,
) -> AppResult<Option<String>> {
    let current_poster_origin: Option<String> = sqlx::query_scalar(
        "SELECT poster_origin FROM shows WHERE id = ?1",
    )
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

    let new_poster_clause = if download_extension.is_some() {
        ", poster_path = ?9, poster_origin = 'tmdb'"
    } else {
        ""
    };

    let sql = format!(
        "UPDATE shows SET
             provider = 'tmdb',
             provider_id = ?1,
             overview = ?2,
             year = ?3,
             rating = ?4,
             genres = ?5,
             top_cast = ?6,
             first_air_date = ?7,
             metadata_synced_at = strftime('%s','now')
             {new_poster_clause}
         WHERE id = ?8"
    );

    let mut query = sqlx::query(&sql)
        .bind(details.id.to_string())
        .bind(details.overview.as_deref())
        .bind(year)
        .bind(details.vote_average)
        .bind(genres_json)
        .bind(cast_json)
        .bind(details.first_air_date.as_deref())
        .bind(show_id);

    if let Some(extension) = download_extension.as_ref() {
        let local_path = format!("show-{show_id}.{extension}");
        query = query.bind(local_path);
    }

    query.execute(&mut *conn).await?;

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
    date.and_then(|d| d.get(0..4)).and_then(|y| y.parse().ok())
}
```

- [ ] **Step 2: Verify the whole metadata module compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -15 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/metadata/apply.rs
git commit -m "feat(metadata): DB write helpers for fetched show/movie details"
```

---

### Task B.6: Build a shared `reqwest::Client` at startup

**Files:**
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add a managed `HttpClient` type**

Open `src-tauri/src/lib.rs`. In the `setup` closure, after `app.manage(pool);`, build and manage a `reqwest::Client`:

```rust
let http_client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .user_agent(concat!("rustflix/", env!("CARGO_PKG_VERSION")))
    .build()
    .expect("failed to build reqwest client");
app.manage(http_client);
```

- [ ] **Step 2: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/lib.rs
git commit -m "feat(metadata): manage a shared reqwest client in app state"
```

---

### Task B.7: `fetch_metadata_now` command — the manual one-shot

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the command**

Append to `src-tauri/src/commands.rs` (before `copy_poster`):

```rust
#[tauri::command]
pub async fn fetch_metadata_now(
    app: AppHandle,
    db: State<'_, Db>,
    http: State<'_, reqwest::Client>,
    kind: String,
    id: i64,
) -> AppResult<()> {
    let api_key = queries::get_app_setting(&db, "tmdb_api_key")
        .await?
        .ok_or_else(|| AppError::Other("auth_required: no TMDB key configured".to_string()))?;

    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?;
    let posters_dir = app_data_dir.join("posters");

    let mut tx = db.begin().await?;

    match kind.as_str() {
        "movie" => fetch_one_movie(&mut tx, &http, &api_key, &posters_dir, id).await?,
        "show" => fetch_one_show(&mut tx, &http, &api_key, &posters_dir, id).await?,
        other => {
            return Err(AppError::Other(format!("unknown kind: {other}")));
        }
    };

    tx.commit().await?;
    Ok(())
}

async fn fetch_one_movie(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    http: &reqwest::Client,
    api_key: &str,
    posters_dir: &std::path::Path,
    movie_id: i64,
) -> AppResult<()> {
    let locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut **tx)
    .await?;
    if locked != 0 {
        return Ok(());
    }

    let (title, year): (String, Option<i32>) =
        sqlx::query_as("SELECT title, year FROM movies WHERE id = ?1")
            .bind(movie_id)
            .fetch_one(&mut **tx)
            .await?;

    let candidates = crate::metadata::tmdb::search_movie(http, api_key, &title, year).await?;
    let Some(pick) =
        crate::metadata::matching::pick_confident_match(&title, year, &candidates)
    else {
        return Ok(());
    };

    let details =
        crate::metadata::tmdb::fetch_movie_details(http, api_key, &pick.provider_id).await?;
    let download_ext =
        crate::metadata::apply::apply_movie_details(&mut **tx, movie_id, &details).await?;

    if let (Some(extension), Some(poster_path)) = (download_ext, details.poster_path.as_deref()) {
        let dest = posters_dir.join(format!("movie-{movie_id}.{extension}"));
        crate::metadata::tmdb::download_poster(http, poster_path, &dest).await?;
    }

    Ok(())
}

async fn fetch_one_show(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    http: &reqwest::Client,
    api_key: &str,
    posters_dir: &std::path::Path,
    show_id: i64,
) -> AppResult<()> {
    let locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_one(&mut **tx)
    .await?;
    if locked != 0 {
        return Ok(());
    }

    let (title, year): (String, Option<i32>) =
        sqlx::query_as("SELECT title, year FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(&mut **tx)
            .await?;

    let candidates = crate::metadata::tmdb::search_show(http, api_key, &title, year).await?;
    let Some(pick) =
        crate::metadata::matching::pick_confident_match(&title, year, &candidates)
    else {
        return Ok(());
    };

    let details =
        crate::metadata::tmdb::fetch_show_details(http, api_key, &pick.provider_id).await?;
    let download_ext =
        crate::metadata::apply::apply_show_details(&mut **tx, show_id, &details).await?;

    if let (Some(extension), Some(poster_path)) = (download_ext, details.poster_path.as_deref()) {
        let dest = posters_dir.join(format!("show-{show_id}.{extension}"));
        crate::metadata::tmdb::download_poster(http, poster_path, &dest).await?;
    }

    Ok(())
}
```

- [ ] **Step 2: Register the command in `lib.rs`**

In `src-tauri/src/lib.rs`, add to the `generate_handler!` macro list:

```rust
commands::fetch_metadata_now,
```

- [ ] **Step 3: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -15 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 4: Commit**

Run:
```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(commands): synchronous fetch_metadata_now (manual verification)"
```

---

### Task B.8: Frontend binding for `fetch_metadata_now`

**Files:**
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Add the method**

In `src/lib/api.ts`, add to the `api` object:

```ts
  fetchMetadataNow: (kind: 'show' | 'movie', id: number) =>
    invoke<void>('fetch_metadata_now', { kind, id }),
```

- [ ] **Step 2: Verify it type-checks**

Run from repo root:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error, no new ones.

- [ ] **Step 3: Commit**

Run:
```bash
git add src/lib/api.ts
git commit -m "feat(api): fetchMetadataNow binding"
```

---

### Task B.9: Add a temporary "Sync now" button to series + movies edit pages

**Files:**
- Modify: `src/routes/series/[id]/edit/+page.svelte`
- Modify: `src/routes/films/[id]/edit/+page.svelte` (verify the path; could be `src/routes/movies/[id]/edit/+page.svelte`)

- [ ] **Step 1: Locate the movies edit page**

Run:
```bash
ls src/routes/films/\[id\]/edit/ 2>/dev/null && echo OR && ls src/routes/movies/\[id\]/edit/ 2>/dev/null
```
Use whichever exists in subsequent steps. (Project history shows `films/` is the route.)

- [ ] **Step 2: Add "Sync now" button to the series edit page**

In `src/routes/series/[id]/edit/+page.svelte`, near the existing `<Button onclick={save}>` (or alongside the other actions), add:

```svelte
<Button
  variant="secondary"
  disabled={!show || syncingMetadata}
  onclick={async () => {
    if (!show) return;
    syncingMetadata = true;
    try {
      await api.fetchMetadataNow('show', show.id);
      await load(show.id);
    } catch (caught) {
      error = String(caught);
    } finally {
      syncingMetadata = false;
    }
  }}
>
  {syncingMetadata ? 'Syncing…' : 'Sync now (temp)'}
</Button>
```

Add `let syncingMetadata = $state(false);` near the other `$state` declarations.

- [ ] **Step 3: Same change in `src/routes/films/[id]/edit/+page.svelte`**

Identical pattern, with `api.fetchMetadataNow('movie', movie.id)` and `await load(movie.id)`.

- [ ] **Step 4: Verify it type-checks**

Run from repo root:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error.

- [ ] **Step 5: Commit**

Run:
```bash
git add src/routes/series/\[id\]/edit/+page.svelte src/routes/films/\[id\]/edit/+page.svelte
git commit -m "feat(ui): temporary Sync-now button on edit pages (verification harness)"
```

---

### Task B.10: Push, PR, merge

- [ ] **Step 1: Push**

Run:
```bash
git push -u origin fix/23-tmdb-fetch
```

- [ ] **Step 2: Open PR**

Run:
```bash
gh pr create --title "TMDB client, matcher, and synchronous fetch_metadata_now" --body "$(cat <<'EOF'
## Summary
- Add the metadata/tmdb.rs concrete TMDB v3 client (search, details, download_poster).
- Add the metadata/matching.rs pure matcher with full unit-test coverage (NFKD-fold, year ± 1, ambiguity ⇒ None).
- Add the metadata/apply.rs DB write helpers that respect manual posters.
- Add a synchronous fetch_metadata_now Tauri command for manual single-item verification.
- Add a temporary "Sync now" button to the series and movies edit pages.

No background worker yet; that lands in PR C. Spec at docs/superpowers/specs/2026-05-24-metadata-sync-design.md.

## Test plan
- [ ] Paste a real TMDB key in /settings/metadata.
- [ ] Open a freshly-scanned movie, click "Sync now (temp)". Verify overview/year/genres/poster populate.
- [ ] Same for a series.
- [ ] Edit a series title inline → metadata_locked = 1 → Sync now is a no-op (rows are not overwritten).
- [ ] Upload a manual poster → Sync now updates text fields but leaves the poster.
- [ ] cargo test metadata::matching passes (10 cases).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Merge**

Run:
```bash
gh pr merge <PR-NUMBER> --merge --delete-branch && git checkout master && git pull --ff-only
```

---

## PR C — fix/24-metadata-worker

Background worker + scanner integration + user-facing UI surfaces. Drops the temporary `fetch_metadata_now`.

### Task C.0: Branch

- [ ] **Step 1: Branch**

Run:
```bash
git checkout master && git pull --ff-only && git checkout -b fix/24-metadata-worker
```

---

### Task C.1: `metadata/queries.rs` — queue CRUD

**Files:**
- Create: `src-tauri/src/metadata/queries.rs`
- Modify: `src-tauri/src/metadata/mod.rs`

- [ ] **Step 1: Create the queries module**

Create `src-tauri/src/metadata/queries.rs`:

```rust
//! Direct SQL helpers for `metadata_jobs`. Keep the SQL strings here so the
//! worker and the scanner stay short.

use sqlx::SqlitePool;

use crate::error::AppResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetadataJob {
    pub kind: String,
    pub media_id: i64,
    pub attempts: i64,
    pub last_error: Option<String>,
    pub next_attempt_at: i64,
}

pub async fn enqueue(pool: &SqlitePool, kind: &str, media_id: i64) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO metadata_jobs (kind, media_id) VALUES (?1, ?2)
         ON CONFLICT(kind, media_id) DO NOTHING",
    )
    .bind(kind)
    .bind(media_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Resets an existing job (or inserts a new one) so the worker treats it
/// as fresh. Used by "Refresh metadata" and "Unlink".
pub async fn force_enqueue(pool: &SqlitePool, kind: &str, media_id: i64) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO metadata_jobs (kind, media_id) VALUES (?1, ?2)
         ON CONFLICT(kind, media_id) DO UPDATE SET
             attempts = 0,
             next_attempt_at = strftime('%s','now'),
             last_error = NULL",
    )
    .bind(kind)
    .bind(media_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Returns the next job whose next_attempt_at <= now and that isn't parked
/// on 'auth_required'. None ⇒ queue is empty / fully parked.
pub async fn next_due(pool: &SqlitePool) -> AppResult<Option<MetadataJob>> {
    let job: Option<MetadataJob> = sqlx::query_as(
        "SELECT kind, media_id, attempts, last_error, next_attempt_at
         FROM metadata_jobs
         WHERE COALESCE(last_error, '') <> 'auth_required'
           AND next_attempt_at <= strftime('%s','now')
         ORDER BY next_attempt_at ASC
         LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    Ok(job)
}

/// Park on 401 ⇒ don't increment attempts. Will only re-enter rotation
/// after a settings change clears the sentinel via `wake_parked`.
pub async fn park_auth(pool: &SqlitePool, kind: &str, media_id: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET last_error = 'auth_required'
         WHERE kind = ?1 AND media_id = ?2",
    )
    .bind(kind)
    .bind(media_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Clears the auth_required sentinel from every parked row so the worker
/// can pick them up. Called when the user saves a new TMDB key.
pub async fn wake_parked(pool: &SqlitePool) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET last_error = NULL, next_attempt_at = strftime('%s','now')
         WHERE last_error = 'auth_required'",
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Exponential backoff: attempts++; next_attempt_at = now + min(60·2^attempts, 3600);
/// last_error = message. At attempts = 8 the row stays in the queue but
/// `next_due` will eventually surface it again only after backoff. The dead
/// letter check happens by reading attempts directly in the worker.
pub async fn record_failure(
    pool: &SqlitePool,
    kind: &str,
    media_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE metadata_jobs SET
             attempts = attempts + 1,
             next_attempt_at = strftime('%s','now') +
                 CASE WHEN (60 * (1 << MIN(attempts + 1, 6))) > 3600
                      THEN 3600
                      ELSE 60 * (1 << MIN(attempts + 1, 6))
                 END,
             last_error = ?3
         WHERE kind = ?1 AND media_id = ?2",
    )
    .bind(kind)
    .bind(media_id)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete(pool: &SqlitePool, kind: &str, media_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM metadata_jobs WHERE kind = ?1 AND media_id = ?2")
        .bind(kind)
        .bind(media_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_in_tx(
    conn: &mut sqlx::SqliteConnection,
    kind: &str,
    media_id: i64,
) -> AppResult<()> {
    sqlx::query("DELETE FROM metadata_jobs WHERE kind = ?1 AND media_id = ?2")
        .bind(kind)
        .bind(media_id)
        .execute(conn)
        .await?;

    Ok(())
}
```

- [ ] **Step 2: Register the new submodule**

Update `src-tauri/src/metadata/mod.rs`:

```rust
pub mod apply;
pub mod matching;
pub mod queries;
pub mod tmdb;
pub mod worker;
```

(`worker` will be created in Task C.3.)

- [ ] **Step 3: Verify it compiles (will fail on `worker` until C.3)**

Optional: temporarily comment out `pub mod worker;`. We'll uncomment in Task C.3.

- [ ] **Step 4: Commit**

Run:
```bash
git add src-tauri/src/metadata/queries.rs src-tauri/src/metadata/mod.rs
git commit -m "feat(metadata): SQL helpers for the metadata_jobs queue"
```

---

### Task C.2: Update `metadata_status_counts` to subtract from `shows`/`movies` for needs-review

**Files:**
- Modify: `src-tauri/src/queries.rs`

- [ ] **Step 1: Replace the `needs_review = 0` stub**

In `src-tauri/src/queries.rs`, replace the `let needs_review: i64 = 0;` line inside `metadata_status_counts` with:

```rust
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
```

- [ ] **Step 2: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/queries.rs
git commit -m "feat(metadata): compute real needs-review count"
```

---

### Task C.3: `metadata/worker.rs` — the async loop

**Files:**
- Create: `src-tauri/src/metadata/worker.rs`

- [ ] **Step 1: Write the worker**

Create `src-tauri/src/metadata/worker.rs`:

```rust
//! Background worker that drains `metadata_jobs`. One task, sequential,
//! 250ms pacing between requests. Waits on a Notify when the queue is
//! empty, when there's no API key, or when every job is parked on
//! auth_required.

use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tauri::{AppHandle, Manager};
use tokio::sync::Notify;
use tokio::time::sleep;

use crate::error::{AppError, AppResult};
use crate::metadata::{apply, matching, queries, tmdb};
use crate::queries as app_queries;

const MAX_ATTEMPTS: i64 = 8;
const PACING_MS: u64 = 250;

pub fn spawn(pool: SqlitePool, http: reqwest::Client, app: AppHandle) -> Arc<Notify> {
    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        if let Err(error) = run(pool, http, app, notify_clone).await {
            eprintln!("metadata worker exited with error: {error}");
        }
    });

    notify
}

async fn run(
    pool: SqlitePool,
    http: reqwest::Client,
    app: AppHandle,
    notify: Arc<Notify>,
) -> AppResult<()> {
    loop {
        let api_key = app_queries::get_app_setting(&pool, "tmdb_api_key").await?;
        let Some(api_key) = api_key else {
            notify.notified().await;
            continue;
        };

        let Some(job) = queries::next_due(&pool).await? else {
            notify.notified().await;
            continue;
        };

        if job.attempts >= MAX_ATTEMPTS {
            // Dead-letter: leave it in the queue but skip it. Sleep briefly
            // so we don't spin-loop on dead rows.
            sleep(Duration::from_secs(60)).await;
            continue;
        }

        let now = chrono_now_unix();
        if job.next_attempt_at > now {
            let wait = Duration::from_secs((job.next_attempt_at - now) as u64);
            tokio::select! {
                _ = sleep(wait) => {},
                _ = notify.notified() => {},
            }
            continue;
        }

        match run_job(&pool, &http, &app, &api_key, &job).await {
            Ok(()) => {}
            Err(error) => handle_failure(&pool, &job, error).await?,
        }

        sleep(Duration::from_millis(PACING_MS)).await;
    }
}

async fn run_job(
    pool: &SqlitePool,
    http: &reqwest::Client,
    app: &AppHandle,
    api_key: &str,
    job: &queries::MetadataJob,
) -> AppResult<()> {
    let posters_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| AppError::Other(format!("app_data_dir: {error}")))?
        .join("posters");

    let mut tx = pool.begin().await?;

    let outcome = match job.kind.as_str() {
        "movie" => fetch_movie(&mut tx, http, api_key, &job.media_id).await?,
        "show" => fetch_show(&mut tx, http, api_key, &job.media_id).await?,
        other => {
            return Err(AppError::Other(format!("unknown job kind: {other}")));
        }
    };

    queries::delete_in_tx(&mut *tx, &job.kind, job.media_id).await?;
    tx.commit().await?;

    if let Some((poster_url, dest_filename)) = outcome {
        let dest = posters_dir.join(dest_filename);
        // Best-effort: a failed poster download doesn't invalidate the
        // already-committed text metadata.
        if let Err(error) = tmdb::download_poster(http, &poster_url, &dest).await {
            eprintln!("poster download failed for {dest:?}: {error}");
        }
    }

    Ok(())
}

async fn fetch_movie(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    http: &reqwest::Client,
    api_key: &str,
    movie_id: &i64,
) -> AppResult<Option<(String, String)>> {
    let locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM movies WHERE id = ?1",
    )
    .bind(movie_id)
    .fetch_one(&mut **tx)
    .await?;
    if locked != 0 {
        return Ok(None);
    }

    let (title, year): (String, Option<i32>) =
        sqlx::query_as("SELECT title, year FROM movies WHERE id = ?1")
            .bind(movie_id)
            .fetch_one(&mut **tx)
            .await?;

    let candidates = tmdb::search_movie(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(None);
    };

    let details = tmdb::fetch_movie_details(http, api_key, &pick.provider_id).await?;
    let download_ext = apply::apply_movie_details(&mut **tx, *movie_id, &details).await?;

    Ok(match (download_ext, details.poster_path) {
        (Some(extension), Some(poster_path)) => Some((
            poster_path,
            format!("movie-{movie_id}.{extension}"),
        )),
        _ => None,
    })
}

async fn fetch_show(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    http: &reqwest::Client,
    api_key: &str,
    show_id: &i64,
) -> AppResult<Option<(String, String)>> {
    let locked: i64 = sqlx::query_scalar(
        "SELECT metadata_locked FROM shows WHERE id = ?1",
    )
    .bind(show_id)
    .fetch_one(&mut **tx)
    .await?;
    if locked != 0 {
        return Ok(None);
    }

    let (title, year): (String, Option<i32>) =
        sqlx::query_as("SELECT title, year FROM shows WHERE id = ?1")
            .bind(show_id)
            .fetch_one(&mut **tx)
            .await?;

    let candidates = tmdb::search_show(http, api_key, &title, year).await?;
    let Some(pick) = matching::pick_confident_match(&title, year, &candidates) else {
        return Ok(None);
    };

    let details = tmdb::fetch_show_details(http, api_key, &pick.provider_id).await?;
    let download_ext = apply::apply_show_details(&mut **tx, *show_id, &details).await?;

    Ok(match (download_ext, details.poster_path) {
        (Some(extension), Some(poster_path)) => Some((
            poster_path,
            format!("show-{show_id}.{extension}"),
        )),
        _ => None,
    })
}

async fn handle_failure(
    pool: &SqlitePool,
    job: &queries::MetadataJob,
    error: AppError,
) -> AppResult<()> {
    let message = error.to_string();

    if message.starts_with("auth_required") {
        queries::park_auth(pool, &job.kind, job.media_id).await?;
    } else {
        queries::record_failure(pool, &job.kind, job.media_id, &message).await?;
    }

    Ok(())
}

fn chrono_now_unix() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
```

- [ ] **Step 2: Uncomment `pub mod worker;` in `metadata/mod.rs` if you commented it earlier.**

- [ ] **Step 3: Spawn the worker in `lib.rs::run`**

In `src-tauri/src/lib.rs`, inside `setup`, after both `app.manage(pool);` and the `app.manage(http_client);` line, add:

```rust
let pool_handle = app.state::<Db>().inner().clone();
let http_handle = app.state::<reqwest::Client>().inner().clone();
let notify = crate::metadata::worker::spawn(pool_handle, http_handle, app.handle().clone());
app.manage(notify);
```

(`Notify` lands in app state so other commands can `notify.notify_one()`.)

- [ ] **Step 4: Verify the whole crate compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -15 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 5: Commit**

Run:
```bash
git add src-tauri/src/metadata/worker.rs src-tauri/src/metadata/mod.rs src-tauri/src/lib.rs
git commit -m "feat(metadata): background worker drains the metadata_jobs queue"
```

---

### Task C.4: Scanner integration — enqueue on new inserts

**Files:**
- Modify: `src-tauri/src/scanner.rs`
- Modify: `src-tauri/src/commands.rs` (the `scan_libraries` wake-the-worker piece)

- [ ] **Step 1: Enqueue in the movie branch**

In `src-tauri/src/scanner.rs`, inside the `Detected::Movie` arm, immediately after the line that increments `report.movies_added += 1;`, insert:

```rust
crate::metadata::queries::enqueue(pool, "movie", new_id).await?;
```

- [ ] **Step 2: Enqueue in the episode branch (when a NEW show was created)**

In the `Detected::Episode` arm, inside the `Created` branch where `created_new_show = true`, immediately after the `INSERT INTO shows ... RETURNING id` query, add:

```rust
crate::metadata::queries::enqueue(pool, "show", id).await?;
```

Be careful to enqueue only when `created_new_show` ends up true. The cleanest place is right after the existing `if created_new_show { report.shows_added += 1; }` block (move the enqueue inside, gated by the same flag).

Re-read the existing scanner code in `src-tauri/src/scanner.rs` between the `INSERT … shows … RETURNING id` and the `INSERT OR IGNORE INTO episodes` calls and place the enqueue call precisely where the show id is known and we've confirmed it's a brand-new row.

- [ ] **Step 3: Wake the worker at end of scan**

In `src-tauri/src/commands.rs`, locate `scan_libraries`. After the existing for loop that scans each library, add:

```rust
if let Some(notify) = app.try_state::<std::sync::Arc<tokio::sync::Notify>>() {
    notify.notify_one();
}
```

This requires `scan_libraries` to take `app: AppHandle` (it already takes `db: State<...>`). If `app` isn't a parameter yet, add it. Update the command body to match. Register changes need no `lib.rs` edits — the command is already registered.

- [ ] **Step 4: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -15 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 5: Commit**

Run:
```bash
git add src-tauri/src/scanner.rs src-tauri/src/commands.rs
git commit -m "feat(scanner): enqueue metadata jobs for new shows/movies and wake worker"
```

---

### Task C.5: Wake parked jobs when the user saves a new TMDB key

**Files:**
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Update `set_tmdb_api_key`**

Replace the `set_tmdb_api_key` command body with:

```rust
#[tauri::command]
pub async fn set_tmdb_api_key(
    app: AppHandle,
    db: State<'_, Db>,
    key: String,
) -> AppResult<()> {
    let trimmed = key.trim();
    if trimmed.is_empty() {
        queries::delete_app_setting(&db, "tmdb_api_key").await?;
    } else {
        queries::set_app_setting(&db, "tmdb_api_key", trimmed).await?;
        crate::metadata::queries::wake_parked(&db).await?;
    }

    if let Some(notify) = app.try_state::<std::sync::Arc<tokio::sync::Notify>>() {
        notify.notify_one();
    }

    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/commands.rs
git commit -m "feat(metadata): saving a key wakes parked jobs and the worker"
```

---

### Task C.6: `refresh_metadata` + `unlink_metadata` commands

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the commands**

Append to `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn refresh_metadata(
    app: AppHandle,
    db: State<'_, Db>,
    kind: String,
    id: i64,
) -> AppResult<()> {
    let table = match kind.as_str() {
        "show" => "shows",
        "movie" => "movies",
        other => return Err(AppError::Other(format!("unknown kind: {other}"))),
    };

    sqlx::query(&format!("UPDATE {table} SET metadata_locked = 0 WHERE id = ?1"))
        .bind(id)
        .execute(&*db)
        .await?;

    crate::metadata::queries::force_enqueue(&db, &kind, id).await?;

    if let Some(notify) = app.try_state::<std::sync::Arc<tokio::sync::Notify>>() {
        notify.notify_one();
    }

    Ok(())
}

#[tauri::command]
pub async fn unlink_metadata(
    app: AppHandle,
    db: State<'_, Db>,
    kind: String,
    id: i64,
) -> AppResult<()> {
    let (table, extras) = match kind.as_str() {
        "show" => ("shows", "first_air_date = NULL, "),
        "movie" => ("movies", "runtime_minutes = NULL, "),
        other => return Err(AppError::Other(format!("unknown kind: {other}"))),
    };

    let sql = format!(
        "UPDATE {table} SET
             provider = NULL,
             provider_id = NULL,
             rating = NULL,
             genres = NULL,
             top_cast = NULL,
             {extras}
             metadata_synced_at = NULL,
             metadata_locked = 0
         WHERE id = ?1"
    );

    sqlx::query(&sql).bind(id).execute(&*db).await?;
    crate::metadata::queries::force_enqueue(&db, &kind, id).await?;

    if let Some(notify) = app.try_state::<std::sync::Arc<tokio::sync::Notify>>() {
        notify.notify_one();
    }

    Ok(())
}
```

- [ ] **Step 2: Register the two commands in `lib.rs`**

Add to the `generate_handler!` list:

```rust
commands::refresh_metadata,
commands::unlink_metadata,
```

Remove `commands::fetch_metadata_now` and delete the function body in `commands.rs` (along with `fetch_one_movie` / `fetch_one_show` helpers). The user-facing path is now Refresh / Unlink; the temporary command served its purpose in PR B.

- [ ] **Step 3: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 4: Commit**

Run:
```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat(metadata): refresh + unlink commands (drop temporary fetch_metadata_now)"
```

---

### Task C.7: `metadata_search` + `link_metadata` commands for the Needs-review picker

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add the commands**

Append to `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn metadata_search(
    db: State<'_, Db>,
    http: State<'_, reqwest::Client>,
    kind: String,
    query: String,
    year: Option<i32>,
) -> AppResult<Vec<crate::metadata::matching::MatchCandidate>> {
    let api_key = queries::get_app_setting(&db, "tmdb_api_key")
        .await?
        .ok_or_else(|| AppError::Other("no TMDB key configured".to_string()))?;

    match kind.as_str() {
        "movie" => crate::metadata::tmdb::search_movie(&http, &api_key, &query, year).await,
        "show" => crate::metadata::tmdb::search_show(&http, &api_key, &query, year).await,
        other => Err(AppError::Other(format!("unknown kind: {other}"))),
    }
}

#[tauri::command]
pub async fn link_metadata(
    app: AppHandle,
    db: State<'_, Db>,
    kind: String,
    media_id: i64,
    provider_id: String,
) -> AppResult<()> {
    let table = match kind.as_str() {
        "show" => "shows",
        "movie" => "movies",
        other => return Err(AppError::Other(format!("unknown kind: {other}"))),
    };

    sqlx::query(&format!(
        "UPDATE {table} SET provider = 'tmdb', provider_id = ?2, metadata_locked = 0
         WHERE id = ?1"
    ))
    .bind(media_id)
    .bind(&provider_id)
    .execute(&*db)
    .await?;

    crate::metadata::queries::force_enqueue(&db, &kind, media_id).await?;

    if let Some(notify) = app.try_state::<std::sync::Arc<tokio::sync::Notify>>() {
        notify.notify_one();
    }

    Ok(())
}
```

Add `serde::Serialize` to the `MatchCandidate` derive in `metadata/matching.rs` so it crosses the Tauri boundary:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MatchCandidate {
    pub provider_id: String,
    pub title: String,
    pub year: Option<i32>,
}
```

- [ ] **Step 2: Register the two commands in `lib.rs`**

```rust
commands::metadata_search,
commands::link_metadata,
```

- [ ] **Step 3: Verify it compiles**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo check 2>&1 | tail -10 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: clean build.

- [ ] **Step 4: Commit**

Run:
```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs src-tauri/src/metadata/matching.rs
git commit -m "feat(metadata): manual search + link commands for the picker UI"
```

---

### Task C.8: Update the frontend API surface

**Files:**
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Add types**

In `src/lib/api.ts`, add:

```ts
export interface MatchCandidate {
  provider_id: string;
  title: string;
  year: number | null;
}
```

- [ ] **Step 2: Replace `fetchMetadataNow` with the four new methods**

Remove the `fetchMetadataNow` entry. Add:

```ts
  refreshMetadata: (kind: 'show' | 'movie', id: number) =>
    invoke<void>('refresh_metadata', { kind, id }),
  unlinkMetadata: (kind: 'show' | 'movie', id: number) =>
    invoke<void>('unlink_metadata', { kind, id }),
  metadataSearch: (kind: 'show' | 'movie', query: string, year: number | null) =>
    invoke<MatchCandidate[]>('metadata_search', { kind, query, year }),
  linkMetadata: (kind: 'show' | 'movie', mediaId: number, providerId: string) =>
    invoke<void>('link_metadata', { kind, mediaId, providerId }),
```

- [ ] **Step 3: Verify it type-checks**

Run:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/lib/api.ts
git commit -m "feat(api): refreshMetadata / unlinkMetadata / metadataSearch / linkMetadata"
```

---

### Task C.9: Replace the temporary "Sync now" buttons with "Refresh" + "Unlink"

**Files:**
- Modify: `src/routes/series/[id]/edit/+page.svelte`
- Modify: `src/routes/films/[id]/edit/+page.svelte`

- [ ] **Step 1: Edit the series page**

In `src/routes/series/[id]/edit/+page.svelte`, replace the "Sync now (temp)" button with:

```svelte
<div class="flex gap-2">
  <Button
    variant="secondary"
    disabled={!show || syncingMetadata}
    onclick={async () => {
      if (!show) return;
      syncingMetadata = true;
      try {
        await api.refreshMetadata('show', show.id);
      } catch (caught) {
        error = String(caught);
      } finally {
        syncingMetadata = false;
      }
    }}
  >
    {syncingMetadata ? 'Refreshing…' : 'Refresh metadata'}
  </Button>
  {#if show?.provider}
    <Button
      variant="ghost"
      disabled={!show || syncingMetadata}
      onclick={async () => {
        if (!show) return;
        syncingMetadata = true;
        try {
          await api.unlinkMetadata('show', show.id);
          await load(show.id);
        } catch (caught) {
          error = String(caught);
        } finally {
          syncingMetadata = false;
        }
      }}
    >
      Unlink
    </Button>
  {/if}
</div>
```

- [ ] **Step 2: Same change on the movies edit page**

Mirror in `src/routes/films/[id]/edit/+page.svelte` using `'movie'` and `movie.id`.

- [ ] **Step 3: Verify**

Run:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/routes/series/\[id\]/edit/+page.svelte src/routes/films/\[id\]/edit/+page.svelte
git commit -m "feat(ui): Refresh metadata + Unlink buttons on edit pages"
```

---

### Task C.10: Needs-review list view

**Files:**
- Create: `src/routes/library/needs-review/+page.svelte`
- Create: `src/lib/components/MetadataMatchSheet.svelte`
- Modify: `src-tauri/src/queries.rs` (add helper to list unlinked items)
- Modify: `src-tauri/src/commands.rs` + `lib.rs` (expose it)
- Modify: `src/lib/api.ts`

- [ ] **Step 1: Backend helper to list unlinked items**

Append to `src-tauri/src/queries.rs`:

```rust
#[derive(Debug, sqlx::FromRow, serde::Serialize)]
pub struct NeedsReviewItem {
    pub kind: String,
    pub id: i64,
    pub title: String,
    pub year: Option<i32>,
}

pub async fn list_needs_review(pool: &SqlitePool) -> AppResult<Vec<NeedsReviewItem>> {
    let items: Vec<NeedsReviewItem> = sqlx::query_as(
        "SELECT 'show' AS kind, id, title, year FROM shows
            WHERE provider IS NULL
              AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                              WHERE j.kind = 'show' AND j.media_id = shows.id)
         UNION ALL
         SELECT 'movie' AS kind, id, title, year FROM movies
            WHERE provider IS NULL
              AND NOT EXISTS (SELECT 1 FROM metadata_jobs j
                              WHERE j.kind = 'movie' AND j.media_id = movies.id)
         ORDER BY title COLLATE NOCASE",
    )
    .fetch_all(pool)
    .await?;

    Ok(items)
}
```

- [ ] **Step 2: Wire the command**

Add to `src-tauri/src/commands.rs`:

```rust
#[tauri::command]
pub async fn list_needs_review(
    db: State<'_, Db>,
) -> AppResult<Vec<crate::queries::NeedsReviewItem>> {
    queries::list_needs_review(&db).await
}
```

Register in `lib.rs`:

```rust
commands::list_needs_review,
```

- [ ] **Step 3: API binding**

In `src/lib/api.ts`:

```ts
export interface NeedsReviewItem {
  kind: 'show' | 'movie';
  id: number;
  title: string;
  year: number | null;
}

// in api object:
  listNeedsReview: () => invoke<NeedsReviewItem[]>('list_needs_review'),
```

- [ ] **Step 4: Build `MetadataMatchSheet.svelte`**

Create `src/lib/components/MetadataMatchSheet.svelte`:

```svelte
<script lang="ts">
  import {
    api,
    type MatchCandidate,
    type NeedsReviewItem,
  } from '$lib/api';
  import * as Sheet from '$lib/components/ui/sheet';
  import { Button } from '$lib/components/ui/button';

  type Props = {
    open: boolean;
    item: NeedsReviewItem | null;
    onClose: () => void;
    onLinked: () => void;
  };

  let { open = $bindable(), item, onClose, onLinked }: Props = $props();

  let candidates = $state<MatchCandidate[]>([]);
  let searching = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (open && item) {
      void runSearch();
    }
  });

  async function runSearch() {
    if (!item) {
      return;
    }
    searching = true;
    error = null;
    try {
      candidates = await api.metadataSearch(item.kind, item.title, item.year);
    } catch (caught) {
      error = String(caught);
    } finally {
      searching = false;
    }
  }

  async function pick(candidate: MatchCandidate) {
    if (!item) {
      return;
    }
    try {
      await api.linkMetadata(item.kind, item.id, candidate.provider_id);
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

    {#if error}
      <p class="mt-3 text-sm text-destructive-foreground">{error}</p>
    {/if}

    {#if searching}
      <p class="mt-3 text-sm text-muted-foreground">Searching…</p>
    {:else}
      <ul class="mt-4 flex flex-col gap-2">
        {#each candidates as candidate (candidate.provider_id)}
          <li>
            <button
              type="button"
              onclick={() => pick(candidate)}
              class="w-full rounded-md border border-border bg-background px-3 py-2 text-left text-sm transition-colors hover:bg-accent"
            >
              <div class="font-medium">{candidate.title}</div>
              {#if candidate.year}
                <div class="text-xs text-muted-foreground">{candidate.year}</div>
              {/if}
            </button>
          </li>
        {/each}
        {#if candidates.length === 0}
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

- [ ] **Step 5: Build the list page**

Create `src/routes/library/needs-review/+page.svelte`:

```svelte
<script lang="ts">
  import { api, type NeedsReviewItem } from '$lib/api';
  import MetadataMatchSheet from '$lib/components/MetadataMatchSheet.svelte';
  import { Button } from '$lib/components/ui/button';

  let items = $state<NeedsReviewItem[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let active = $state<NeedsReviewItem | null>(null);
  let sheetOpen = $state(false);

  $effect(() => {
    void load();
  });

  async function load() {
    loading = true;
    try {
      items = await api.listNeedsReview();
    } catch (caught) {
      error = String(caught);
    } finally {
      loading = false;
    }
  }

  function openSheet(item: NeedsReviewItem) {
    active = item;
    sheetOpen = true;
  }
</script>

<div class="mx-auto max-w-3xl px-6 py-8">
  <h1 class="mb-6 text-3xl font-bold tracking-tight">Needs review</h1>
  {#if error}
    <p class="mb-4 text-sm text-destructive-foreground">{error}</p>
  {/if}

  {#if loading}
    <p class="text-sm text-muted-foreground">Loading…</p>
  {:else if items.length === 0}
    <p class="text-sm text-muted-foreground">Everything is matched. Nothing to review.</p>
  {:else}
    <ul class="flex flex-col gap-2">
      {#each items as item (item.kind + ':' + item.id)}
        <li
          class="flex items-center justify-between rounded-md border border-border bg-card px-4 py-3"
        >
          <div>
            <div class="font-medium">{item.title}</div>
            <div class="text-xs uppercase tracking-wide text-muted-foreground">
              {item.kind}{item.year ? ` · ${item.year}` : ''}
            </div>
          </div>
          <Button onclick={() => openSheet(item)}>Match…</Button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<MetadataMatchSheet
  bind:open={sheetOpen}
  item={active}
  onClose={() => (sheetOpen = false)}
  onLinked={() => load()}
/>
```

- [ ] **Step 6: Verify**

Run:
```bash
touch src-tauri/binaries/mpv-x86_64-unknown-linux-gnu && cd src-tauri && cargo check 2>&1 | tail -15 ; cd .. ; rm -f src-tauri/binaries/mpv-x86_64-unknown-linux-gnu
CI=true pnpm check 2>&1 | tail -10
```
Expected: both clean (one pre-existing TS error remains).

- [ ] **Step 7: Commit**

Run:
```bash
git add src-tauri/src/queries.rs src-tauri/src/commands.rs src-tauri/src/lib.rs src/lib/api.ts src/lib/components/MetadataMatchSheet.svelte src/routes/library/needs-review/+page.svelte
git commit -m "feat(metadata): Needs-review list view + match sheet"
```

---

### Task C.11: Needs-review badges on the library pages

**Files:**
- Modify: `src/routes/series/+page.svelte`
- Modify: `src/routes/films/+page.svelte`

- [ ] **Step 1: Load the count on each library page**

In both files, add to the `<script>` block:

```ts
import { api, type MetadataStatusCounts } from '$lib/api';

let metadataCounts = $state<MetadataStatusCounts | null>(null);

$effect(() => {
  void api.metadataStatusCounts().then((counts) => {
    metadataCounts = counts;
  }).catch(() => {});
});
```

- [ ] **Step 2: Show a small badge if `needs_review > 0`**

Below the page header (or wherever feels natural), add:

```svelte
{#if metadataCounts && metadataCounts.needs_review > 0}
  <a
    href="/library/needs-review"
    class="mb-4 inline-flex items-center gap-2 rounded-md border border-yellow-500/30 bg-yellow-500/10 px-3 py-1.5 text-xs font-medium text-yellow-200"
  >
    {metadataCounts.needs_review} item{metadataCounts.needs_review === 1 ? '' : 's'} need a match
  </a>
{/if}
```

- [ ] **Step 3: Verify**

Run:
```bash
CI=true pnpm check 2>&1 | tail -10
```
Expected: same single pre-existing error.

- [ ] **Step 4: Commit**

Run:
```bash
git add src/routes/series/+page.svelte src/routes/films/+page.svelte
git commit -m "feat(ui): needs-review badge on library pages"
```

---

### Task C.12: Tests for the queue SQL helpers

**Files:**
- Modify: `src-tauri/src/metadata/queries.rs` (add `#[cfg(test)]` module)

The spec calls for "light test using an in-memory sqlx pool: backoff math after N attempts, 401 → parked, metadata_locked → skip." `metadata_locked → skip` is exercised by `run_job` which is HTTP-coupled; the backoff math and the auth-park logic are pure SQL and easy to test.

- [ ] **Step 1: Append the test module to `metadata/queries.rs`**

Add at the bottom of `src-tauri/src/metadata/queries.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .expect("memory pool");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("migrations");
        pool
    }

    async fn seed_show(pool: &SqlitePool, library_id: i64) -> i64 {
        sqlx::query("INSERT INTO libraries (id, path, kind) VALUES (?1, '/tmp', 'series')")
            .bind(library_id)
            .execute(pool)
            .await
            .expect("library");

        sqlx::query(
            "INSERT INTO shows (library_id, title, folder_path, fingerprint)
             VALUES (?1, 'Test', '/tmp/test', 'test')",
        )
        .bind(library_id)
        .execute(pool)
        .await
        .expect("show");

        sqlx::query_scalar::<_, i64>("SELECT last_insert_rowid()")
            .fetch_one(pool)
            .await
            .expect("show id")
    }

    #[tokio::test]
    async fn enqueue_then_next_due_returns_the_job() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;

        enqueue(&pool, "show", show_id).await.unwrap();

        let job = next_due(&pool).await.unwrap().expect("a job");
        assert_eq!(job.kind, "show");
        assert_eq!(job.media_id, show_id);
        assert_eq!(job.attempts, 0);
    }

    #[tokio::test]
    async fn enqueue_twice_is_a_noop() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;

        enqueue(&pool, "show", show_id).await.unwrap();
        enqueue(&pool, "show", show_id).await.unwrap();

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM metadata_jobs WHERE kind = 'show' AND media_id = ?1",
        )
        .bind(show_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn park_auth_excludes_job_from_next_due() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;
        enqueue(&pool, "show", show_id).await.unwrap();

        park_auth(&pool, "show", show_id).await.unwrap();

        let job = next_due(&pool).await.unwrap();
        assert!(job.is_none(), "parked job should be excluded");
    }

    #[tokio::test]
    async fn wake_parked_clears_sentinel() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;
        enqueue(&pool, "show", show_id).await.unwrap();
        park_auth(&pool, "show", show_id).await.unwrap();

        wake_parked(&pool).await.unwrap();

        let job = next_due(&pool).await.unwrap().expect("should be due again");
        assert!(job.last_error.is_none());
    }

    #[tokio::test]
    async fn record_failure_increments_attempts_and_backs_off() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;
        enqueue(&pool, "show", show_id).await.unwrap();

        record_failure(&pool, "show", show_id, "boom").await.unwrap();

        let (attempts, next_attempt_at, last_error): (i64, i64, Option<String>) = sqlx::query_as(
            "SELECT attempts, next_attempt_at, last_error FROM metadata_jobs WHERE kind = 'show' AND media_id = ?1",
        )
        .bind(show_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(attempts, 1);
        assert_eq!(last_error.as_deref(), Some("boom"));

        let now: i64 = sqlx::query_scalar("SELECT strftime('%s','now')")
            .fetch_one(&pool)
            .await
            .unwrap();
        // After 1 failure: 60 * 2^1 = 120s.
        assert!(next_attempt_at >= now + 60 && next_attempt_at <= now + 200);
    }

    #[tokio::test]
    async fn backoff_caps_at_one_hour() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;
        enqueue(&pool, "show", show_id).await.unwrap();

        // Force attempts up to 8 to test the cap.
        sqlx::query("UPDATE metadata_jobs SET attempts = 7 WHERE kind = 'show' AND media_id = ?1")
            .bind(show_id)
            .execute(&pool)
            .await
            .unwrap();

        record_failure(&pool, "show", show_id, "still broken").await.unwrap();

        let next_attempt_at: i64 = sqlx::query_scalar(
            "SELECT next_attempt_at FROM metadata_jobs WHERE kind = 'show' AND media_id = ?1",
        )
        .bind(show_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let now: i64 = sqlx::query_scalar("SELECT strftime('%s','now')")
            .fetch_one(&pool)
            .await
            .unwrap();

        // Cap is 3600s.
        assert!(next_attempt_at <= now + 3700);
        assert!(next_attempt_at >= now + 3500);
    }

    #[tokio::test]
    async fn force_enqueue_resets_existing_job() {
        let pool = fresh_pool().await;
        let show_id = seed_show(&pool, 1).await;
        enqueue(&pool, "show", show_id).await.unwrap();
        record_failure(&pool, "show", show_id, "boom").await.unwrap();
        record_failure(&pool, "show", show_id, "boom").await.unwrap();

        force_enqueue(&pool, "show", show_id).await.unwrap();

        let job = next_due(&pool).await.unwrap().expect("a job");
        assert_eq!(job.attempts, 0);
        assert!(job.last_error.is_none());
    }
}
```

- [ ] **Step 2: Run the tests**

Run from `src-tauri/`:
```bash
touch binaries/mpv-x86_64-unknown-linux-gnu && cargo test metadata::queries 2>&1 | tail -20 ; rm -f binaries/mpv-x86_64-unknown-linux-gnu
```
Expected: 7 passing tests. If any fail, the most likely culprit is the SQLite `CASE` arithmetic in `record_failure`'s exponential backoff — adjust the SQL to match the test's expectations.

- [ ] **Step 3: Commit**

Run:
```bash
git add src-tauri/src/metadata/queries.rs
git commit -m "test(metadata): queue SQL helpers (enqueue, park, wake, backoff)"
```

---

### Task C.13: Push, PR, merge

- [ ] **Step 1: Push**

Run:
```bash
git push -u origin fix/24-metadata-worker
```

- [ ] **Step 2: Open PR**

Run:
```bash
gh pr create --title "Metadata sync worker, scanner hookup, and review UI" --body "$(cat <<'EOF'
## Summary
- Background async worker drains metadata_jobs at a 250ms cadence with exponential backoff. Parks on 401, wakes when the user saves a new key.
- Scanner now enqueues a metadata job whenever it inserts a new show / movie row and signals the worker after the scan loop.
- Refresh + Unlink buttons on the series/movies edit pages; the temporary fetch_metadata_now is removed.
- Needs-review list view at /library/needs-review with a TMDB search + link sheet.
- Needs-review badge on the library pages when the count is non-zero.

Closes out the design from docs/superpowers/specs/2026-05-24-metadata-sync-design.md.

## Test plan
- [ ] Scan a fresh library with a TMDB key set → worker drains jobs → posters and metadata populate.
- [ ] Clear the TMDB key → in-flight job parks (last_error = 'auth_required'); no retry storm in logs.
- [ ] Re-paste the key → parked jobs resume (verify metadata_jobs rows fall to zero over time).
- [ ] Edit a series title inline → next worker pass skips it (metadata_locked = 1).
- [ ] Click Unlink → row's metadata cleared, re-fetched, lock reset.
- [ ] Add a show with an ambiguous title ("The Office") → ends up in /library/needs-review.
- [ ] Pick a candidate in the match sheet → row links, sync completes.
- [ ] Kill the app mid-fetch → restart → the in-flight job completes on next worker pass (job row still in DB).

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

- [ ] **Step 3: Merge**

Run:
```bash
gh pr merge <PR-NUMBER> --merge --delete-branch && git checkout master && git pull --ff-only
```

---

## Post-merge: full manual verification

Once all three PRs are merged, walk the test plan from the spec end-to-end against a real TMDB key and a real library. Anything that fails goes into a follow-up `fix/N-…` branch, not into amendments to these three PRs (the iterate pattern: one branch = one logical change).

The matcher unit tests are the only automated coverage — run them via `cd src-tauri && cargo test metadata` on every change touching `metadata/`.
