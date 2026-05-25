# IMDB fallback provider — design

Status: approved 2026-05-25 (after three senior-eng reviews), ready for
implementation planning.

## Goal

Add IMDB as a second metadata provider alongside TMDB so users can sync
metadata without signing up for a TMDB developer key. Provider selection
is user-configurable per app: a single `metadata_mode` setting picks one of
`off | tmdb_only | imdb_only | prefer_tmdb | prefer_imdb`. Default is
`prefer_tmdb` (= current behaviour).

## Non-goals

- HTML scraping of `www.imdb.com`. AWS WAF returns a JS challenge for every
  request without browser-evaluated cookies. The v1 scraper at tag v1.0.0
  worked in March 2026 and does not work now. Headless browser bypasses
  are out of scope.
- Concurrent provider racing. Sequential walk with intra-walk pacing.
- Provider trait. Concrete `Provider` enum + match dispatch stays.
- Persistent worker-side setting cache. Worker reads per job.
- Backend struct-per-area for settings. Generic key/value + server-side
  validator + side-effect dispatch on write.
- TMDB-only or IMDB-only deployment modes baked at compile time. Both
  providers always linked; mode selects which run at runtime.
- A general translation/UI-i18n system. Tracked in TODO.md.

## Decisions

1. **Provider abstraction** — concrete `Provider` enum (`Tmdb`, `Imdb`),
   no trait. Dispatched by a `providers_for_mode(mode, has_tmdb_key)`
   helper that returns either `Vec<Provider>` (walk in order) or a typed
   `ParkReason` (park the job).
2. **IMDB access** — two undocumented JSON endpoints, no HTML:
   - Search: `GET https://v3.sg.media-imdb.com/suggestion/<letter>/<slug>.json`
   - Details: `POST https://caching.graphql.imdb.com/`
   - Image: direct CDN `https://m.media-amazon.com/images/M/...@._V1_<size>_.jpg`
3. **Settings storage** — existing `app_settings(key, value)` table.
   New key `metadata_mode`. No schema change.
4. **Settings command shape** — keep generic `get_app_setting` /
   `set_app_setting`. Add server-side `validate(key, value)`. Add
   `on_setting_changed` dispatch for post-write side-effects (wake parked
   jobs / notify worker). Delete the bespoke `set_tmdb_api_key` command —
   the side-effect dispatcher handles it generically.
5. **Sentinel rename** — `metadata_jobs.last_error = 'auth_required'`
   becomes `'tmdb_auth_required'`. A new sentinel `'no_provider_available'`
   replaces the silent empty-walk path.
6. **Fallback semantics** —
   - `prefer_tmdb` and TMDB 401: set a global `tmdb_auth_bad = '1'` flag in
     `app_settings`, surface as a banner, but fall through to IMDB for the
     current job. The "key is broken" notice decouples from "queue
     blocked."
   - Per-provider parse / 5xx error mid-walk: try the next provider in
     the same invocation. `record_failure` runs only when all providers
     in the walk failed.
   - Per-job pacing: 250ms between providers within a walk, in addition
     to 250ms between jobs.
7. **Matcher** — unchanged. `MatchCandidate` gains a `provider: Provider`
   field so the manual match sheet can render mixed results.
8. **Apply layer** — new `apply_imdb_{movie,show}_details` mirror the
   TMDB versions. Same DB columns, `provider = 'tmdb'` or `'imdb'` tag.
   The partial UNIQUE index on `(provider, provider_id)` already exists.
9. **Manual link/refresh/search commands** — every command that takes a
   media id also takes a `provider`. Hardcoded `'tmdb'` strings removed.

## Architecture

A new `metadata::imdb` module sits next to `metadata::tmdb`. Both expose
the same five async functions: `search_movie`, `search_show`,
`fetch_movie_details`, `fetch_show_details`, `download_poster`. The
worker becomes a small dispatcher that walks a provider list and commits
the first confident match.

```
┌── worker (per job) ────────────────────────────────────────┐
│ 1. Read row + metadata_mode + tmdb_api_key (3 SELECTs).    │
│ 2. Build provider list via providers_for_mode.             │
│ 3. If error: park with typed sentinel, return.             │
│ 4. For each provider in list:                              │
│    a. dispatch_provider(provider, &ctx) → Outcome          │
│    b. Outcome::Matched(tx) → commit, delete job, return.   │
│    c. Outcome::NoMatch → try next provider.                │
│    d. Outcome::Err(e) → record per-provider error,         │
│       try next provider.                                   │
│    e. tokio::sleep(250ms) between providers in the walk.   │
│ 5. All providers exhausted without match:                  │
│    a. If any provider erred, record_failure (backoff).     │
│    b. Else delete job (row surfaces in Needs review).      │
└────────────────────────────────────────────────────────────┘
```

`dispatch_provider` is its own helper file or section, ~80 lines per
provider, keeping the worker at ~100 LOC for the loop body. The current
`fetch_movie` / `fetch_show` 50-line helpers move into the dispatcher.

## Schema and migration

No new migration required. Reuse `app_settings`. The `metadata_jobs`
table stays as is; `last_error` values are strings — renaming the
sentinel is a behavioural change in the worker, not a schema change.

A one-shot data fixup runs from `db::dedupe_shows_and_index` on next
startup: `UPDATE metadata_jobs SET last_error = 'tmdb_auth_required'
WHERE last_error = 'auth_required'`. Idempotent. (If users haven't run
0003 yet, the table doesn't exist; the fixup runs after migrations.)

## Settings infrastructure

### Storage

`app_settings(key TEXT PRIMARY KEY, value TEXT)` — already exists.

Known keys today:

| Key | Type / values | Default | Side-effects on write |
|---|---|---|---|
| `tmdb_api_key` | text \| null | `null` | wake_parked + notify_worker |
| `metadata_mode` | `off`/`tmdb_only`/`imdb_only`/`prefer_tmdb`/`prefer_imdb` | `prefer_tmdb` | notify_worker (also wake parked when transitioning *from* `off` or when changing the provider order) |
| `scrape_language` | text (e.g. `en`, `sv-SE`) | `en` | notify_worker |
| `tmdb_auth_bad` | `'1'` flag, present means key is bad | unset | (internal — set by worker, cleared by `set_tmdb_api_key`) |

Future (deferred to TODO):

| `theme` | `system`/`light`/`dark` | `system` | none |
| `ui_language` | text (e.g. `en`, `sv`) | `en` | none |

### Backend

Generic command stays generic, but grows validation + side-effect
dispatch:

```rust
#[tauri::command]
pub async fn set_app_setting(
    app: AppHandle,
    db: State<'_, Db>,
    key: String,
    value: Option<String>,
) -> AppResult<()> {
    validate(&key, value.as_deref())?;
    let previous = queries::get_app_setting(&db, &key).await?;
    match value.as_deref() {
        Some(v) => queries::set_app_setting(&db, &key, v).await?,
        None    => queries::delete_app_setting(&db, &key).await?,
    }
    on_setting_changed(&app, &db, &key, previous.as_deref(), value.as_deref()).await?;
    Ok(())
}

fn validate(key: &str, value: Option<&str>) -> AppResult<()> {
    match key {
        "metadata_mode" => validate_enum(value, &[
            "off", "tmdb_only", "imdb_only", "prefer_tmdb", "prefer_imdb",
        ]),
        "scrape_language" => validate_lang_code(value),
        _ => Ok(()),  // free-form keys
    }
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
            if previous == Some("off") && next != Some("off") {
                crate::metadata::queries::wake_parked(db).await?;
            }
            wake_worker(app);
        }
        "scrape_language" => wake_worker(app),
        _ => {}
    }
    Ok(())
}
```

Backend defaults via `fn default_for(key: &str) -> Option<&'static str>`:

```rust
pub fn default_for(key: &str) -> Option<&'static str> {
    match key {
        "metadata_mode" => Some("prefer_tmdb"),
        "scrape_language" => Some("en"),
        "ui_language" => Some("en"),
        "theme" => Some("system"),
        _ => None,
    }
}
```

Worker / read sites do `get_app_setting(key).await?.or_else(|| default_for(key).map(String::from))`.

The bespoke `set_tmdb_api_key` Tauri command is **removed**. Its
side-effects live in `on_setting_changed`. The frontend wrapper always
calls `set_app_setting('tmdb_api_key', ...)`.

### Frontend wrapper

`src/lib/settings.ts`:

```ts
import { invoke } from '@tauri-apps/api/core';

export type MetadataMode = 'off' | 'tmdb_only' | 'imdb_only' | 'prefer_tmdb' | 'prefer_imdb';

type SettingDef<T> = {
  default: T;
  parse: (raw: string | null) => T;
  encode: (value: T) => string | null;
};

export const SETTINGS = {
  tmdb_api_key: {
    default: null as string | null,
    parse: (raw) => raw,
    encode: (value) => value,
  } satisfies SettingDef<string | null>,
  metadata_mode: {
    default: 'prefer_tmdb' as MetadataMode,
    parse: (raw): MetadataMode => {
      const valid: readonly MetadataMode[] = [
        'off', 'tmdb_only', 'imdb_only', 'prefer_tmdb', 'prefer_imdb',
      ];
      return (valid as readonly string[]).includes(raw ?? '')
        ? (raw as MetadataMode)
        : 'prefer_tmdb';
    },
    encode: (value) => value,
  } satisfies SettingDef<MetadataMode>,
  scrape_language: {
    default: 'en',
    parse: (raw) => raw ?? 'en',
    encode: (value) => value,
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
  const encoded = SETTINGS[key].encode(value);
  await invoke<void>('set_app_setting', { key, value: encoded });
}
```

Per-setting `parse`/`encode` functions sidestep the conditional-type
gymnastics the reviewer warned about; types narrow correctly per key
because of `satisfies`.

## IMDB module

`src-tauri/src/metadata/imdb.rs`. Public surface:

```rust
pub async fn search_movie(client: &Client, title: &str, year: Option<i32>)
    -> AppResult<Vec<MatchCandidate>>;
pub async fn search_show(client: &Client, title: &str, year: Option<i32>)
    -> AppResult<Vec<MatchCandidate>>;
pub async fn fetch_movie_details(client: &Client, imdb_id: &str)
    -> AppResult<ImdbMovieDetails>;
pub async fn fetch_show_details(client: &Client, imdb_id: &str)
    -> AppResult<ImdbShowDetails>;
pub async fn download_poster(client: &Client, image_url: &str, dest: &Path, size: PosterSize)
    -> AppResult<()>;
```

No `language` parameter. The suggestion API returns English titles; the
GraphQL plot is English by default. We don't ask for localised metadata
in v1 — the matcher works against TMDB-English / filename-English titles,
and localised plot from IMDB would degrade match quality elsewhere. The
scrape-language picker on TODO targets TMDB specifically; IMDB stays
English.

### Search — suggestion API

```
GET https://v3.sg.media-imdb.com/suggestion/<first-alnum-char>/<slug>.json
```

Where `<slug>` is `title.to_lowercase()` with non-alphanumerics collapsed
to `_`. The first-alnum-char shard is a cache key, not validation —
`z/the_matrix.json` returns the same data; using the real first letter
improves CloudFront hit rate.

If `year` is known: try `slug + "_" + year` first (e.g.
`dune_2021.json`); if zero results, fall back to plain slug.

Response shape (relevant fields):
```json
{
  "d": [
    {
      "id": "tt0133093",
      "l": "The Matrix",
      "y": 1999,
      "q": "feature",
      "qid": "movie",
      "i": { "imageUrl": "...", "width": 2100, "height": 3156 }
    }
  ]
}
```

Filter by `qid`:
- `search_movie`: keep `qid == "movie"` (TV movies use `tvMovie` — include or skip; v1 skips them).
- `search_show`: keep `qid IN ("tvSeries", "tvMiniSeries")`.

Map each survivor to `MatchCandidate { provider: Provider::Imdb,
provider_id: d.id, title: d.l, year: d.y }`. Top 10 are returned; the
existing `pick_confident_match` does the year-±1 and NFKD-fold filter.

### Details — GraphQL API

```
POST https://caching.graphql.imdb.com/
Content-Type: application/json

{
  "operationName": "TitleDetails",
  "variables": { "id": "<imdb_id>" },
  "query": "<gql below>"
}
```

GraphQL query (one query handles both movies and shows; episode-count
fields aren't requested):

```graphql
query TitleDetails($id: ID!) {
  title(id: $id) {
    id
    titleText { text }
    originalTitleText { text }
    titleType { id text }
    releaseYear { year endYear }
    releaseDate { day month year }
    plot { plotText { plainText } }
    ratingsSummary { aggregateRating voteCount }
    runtime { seconds }
    genres { genres { id text } }
    primaryImage { url width height }
    principalCredits(filter: { categories: ["director","writer","cast"] }) {
      category { text }
      credits {
        name { id nameText { text } primaryImage { url } }
        ... on Cast { characters { name } }
      }
    }
  }
}
```

Local `ImdbMovieDetails` / `ImdbShowDetails` structs deserialise from
the JSON. Both share fields; the only behavioural split is which
`apply_*` function consumes them (movies have `runtime_minutes`, shows
have `first_air_date` and no runtime).

**Cast handling:** the `principalCredits` array contains category
groupings. Filter to `category.text == "Cast"`, take top 10, map to
`{ name, character, order }` JSON shape we already store in `top_cast`.
Top-10 order matches IMDB's billed-cast order.

**Image URL rewriting:** the `primaryImage.url` is an Amazon CDN URL
like `https://m.media-amazon.com/images/M/<asset>@._V1_.jpg`. The
`_V1_.jpg` segment is a transform marker. Two sizes used by us:

```rust
pub enum PosterSize { Small, Hero }

impl PosterSize {
    fn segment(self) -> &'static str {
        match self {
            PosterSize::Small => "_V1_SX500_",        // ~270KB
            PosterSize::Hero  => "_V1_QL90_UX1280_",  // ~1.3MB
        }
    }
}
```

`download_poster` rewrites `_V1_` to the chosen size before fetch.
Worker uses `Small` for the persisted poster row (same display surface
as TMDB `w500`).

### Error mapping

| Response | Maps to |
|---|---|
| HTTP 200 + empty `d` array (search) | `Ok(vec![])` |
| HTTP 200 + GraphQL `errors[]` | `AppError::Other("imdb graphql: <error message>")` |
| HTTP 202 (any endpoint) | `AppError::Other("imdb_waf: Amazon may have blocked these endpoints; see CLAUDE.md")` — distinct from regular errors so we can monitor for drift |
| HTTP 429 / 403 / 5xx | `AppError::Other("imdb_rate_limited: <status>")` → backoff path |
| HTTP 404 | `AppError::Other("imdb not_found: ...")` → treated as no-match by the worker (delete job) |
| JSON parse error | `AppError::Other("imdb parse: <field>")` |

### TOS disclaimer

The GraphQL endpoint includes `extensions.disclaimer` on every response:
"Public, commercial, and/or non-private use of the IMDb data provided by
this API is not allowed."

For a personal Tauri app shipping locally, this is acceptable. If
rustflix ever distributes to third parties as commercial software, the
IMDB path must be disabled and the `tmdb_only` mode must remain
functional. The module's top-of-file comment carries this notice.

### Crates

No new crates needed. `reqwest` (already present) + `serde` + `serde_json`
cover both endpoints. Drop the planned `scraper = "0.20"` dep — no HTML.

## Worker changes

### `providers_for_mode`

```rust
pub enum ParkReason {
    TmdbAuthRequired,
    NoProviderAvailable,
}

pub fn providers_for_mode(
    mode: &str,
    has_tmdb_key: bool,
) -> Result<Vec<Provider>, ParkReason> {
    use Provider::*;
    match mode {
        "off"          => Ok(vec![]),
        "tmdb_only"    => if has_tmdb_key { Ok(vec![Tmdb]) }
                          else            { Err(ParkReason::NoProviderAvailable) },
        "imdb_only"    => Ok(vec![Imdb]),
        "prefer_imdb"  => if has_tmdb_key { Ok(vec![Imdb, Tmdb]) }
                          else            { Ok(vec![Imdb]) },
        _              => if has_tmdb_key { Ok(vec![Tmdb, Imdb]) }
                          else            { Ok(vec![Imdb]) },  // prefer_tmdb falls back to IMDB
    }
}
```

When `mode == "off"` the empty walk means "feature disabled" — the worker
deletes the job (not parked). The Needs review surface doesn't reflect
disabled-mode rows; they're just unlinked. UI shows a banner when mode is
off.

### Worker loop

```rust
loop {
    let mode = read_setting("metadata_mode").or_default();
    let has_key = read_setting("tmdb_api_key").is_some();

    let job = next_due_job().await?;

    let providers = match providers_for_mode(&mode, has_key) {
        Ok(list) if list.is_empty() => { delete_job(&job).await?; continue; }
        Ok(list) => list,
        Err(reason) => { park_with_reason(&job, reason).await?; continue; }
    };

    let mut last_err: Option<AppError> = None;
    let mut saw_tmdb_auth = false;
    let mut matched = false;

    for (i, provider) in providers.iter().enumerate() {
        if i > 0 { tokio::time::sleep(Duration::from_millis(250)).await; }
        match dispatch_provider(*provider, &job, &ctx).await {
            Ok(Outcome::Matched) => { matched = true; break; }
            Ok(Outcome::NoMatch) => continue,
            Err(error) => {
                if error.to_string().starts_with("tmdb_auth_required") {
                    saw_tmdb_auth = true;
                    set_app_setting(&db, "tmdb_auth_bad", "1").await?;
                }
                last_err = Some(error);
                continue;
            }
        }
    }

    if matched {
        // dispatch_provider already committed and deleted the job in one tx.
    } else if saw_tmdb_auth {
        // At least one provider in the walk hit a TMDB auth error. Park the
        // row — backoff would just retry the broken key forever. Wakes on
        // the next set_app_setting('tmdb_api_key', …) via on_setting_changed.
        park_with_reason(&job, ParkReason::TmdbAuthRequired).await?;
    } else if let Some(error) = last_err {
        // Real transient failure (5xx, parse, rate limit). Backoff.
        record_failure(&job, &error).await?;
    } else {
        // No errors — providers said no-match. Row surfaces in Needs review.
        delete_job(&job).await?;
    }

    tokio::time::sleep(Duration::from_millis(250)).await;
}
```

### `dispatch_provider`

Self-contained per-provider function. Reads the row, searches via the
provider, matches, fetches details, applies, downloads poster, deletes
the job — all in a single transaction guarded by a re-check of
`metadata_locked` (existing pattern preserved). Returns `Outcome::Matched`
or `Outcome::NoMatch` or `Err(AppError)`.

`dispatch_provider` for IMDB doesn't need an API key (it gets passed
`&()` or nothing for that argument). For TMDB it takes the key.

### Park / wake with new sentinel

- `park_with_reason(job, ParkReason::TmdbAuthRequired)` writes
  `last_error = 'tmdb_auth_required'`.
- `park_with_reason(job, ParkReason::NoProviderAvailable)` writes
  `last_error = 'no_provider_available'`.
- `wake_parked` clears either sentinel and is called from
  `on_setting_changed` whenever the TMDB key is set or the mode changes.
- `next_due` excludes any row whose `last_error` matches a parking
  sentinel.

### Status counts

`metadata_status_counts` grows two columns: `tmdb_auth_required` and
`no_provider_available`. The frontend `MetadataStatusCounts` interface
mirrors. The metadata settings page renders both as distinct cards.

## Manual override commands

All four commands grow a `provider` parameter where they don't have one:

```rust
metadata_search(kind: String, query: String, year: Option<i32>, provider: String)
    -> Vec<MatchCandidate>
link_metadata(kind: String, media_id: i64, provider: String, provider_id: String)
    -> ()
refresh_metadata(kind: String, id: i64)  // already provider-agnostic; no change
unlink_metadata(kind: String, id: i64)   // already provider-agnostic; no change
```

`link_metadata` SQL changes from `SET provider = 'tmdb'` to
`SET provider = ?2`. `force_enqueue` follows. After link, the worker on
its next pass takes a fast path: if the row already has provider +
provider_id, skip search and go straight to fetch_details for that
provider. This makes manual picks deterministic regardless of mode.

`metadata_search` validates `provider IN ("tmdb", "imdb")` and dispatches
to the right module. For TMDB it requires a key (returns
`AppError::Other("tmdb_auth_required: ...")` when missing); for IMDB it
needs no key.

`MatchCandidate` adds a `provider: Provider` field, serialized as
`"tmdb"` / `"imdb"` over the Tauri bridge. The match sheet uses this to
render the right candidate-source label.

## Frontend changes

### Settings → Metadata page

Adds a Mode select above the existing TMDB key input:

```svelte
<Select.Root type="single" bind:value={metadataMode} onValueChange={onModeChange}>
  <Select.Trigger>{labelFor(metadataMode)}</Select.Trigger>
  <Select.Content>
    <Select.Item value="off">Off (no metadata sync)</Select.Item>
    <Select.Item value="tmdb_only">TMDB only</Select.Item>
    <Select.Item value="imdb_only">IMDB only</Select.Item>
    <Select.Item value="prefer_tmdb">Prefer TMDB, fall back to IMDB</Select.Item>
    <Select.Item value="prefer_imdb">Prefer IMDB, fall back to TMDB</Select.Item>
  </Select.Content>
</Select.Root>
```

- When mode is `off`: the rest of the page (key input, status counts)
  hides behind a "Metadata sync is disabled" banner.
- When mode is `imdb_only`: API key input stays visible but disabled
  with note "TMDB key not used in IMDB-only mode."
- When `tmdb_auth_bad = '1'`: a red banner above the key input —
  "Your TMDB key was rejected. Update it to resume sync." Setting a new
  key clears the flag (`on_setting_changed`).

### Needs-review match sheet

Two-tab UI at the top of the sheet: "TMDB" and "IMDB". The default tab
follows the active mode (TMDB for `prefer_tmdb`/`tmdb_only`, IMDB
otherwise). The TMDB tab is disabled when no key is set; clicking shows
a tooltip "Add a TMDB key under Settings → Metadata to enable."

Switching tabs re-runs `metadata_search(kind, query, year, provider)`
through the selected provider. Picking a candidate calls
`link_metadata(kind, id, provider, provider_id)`.

## Errors

No new `AppError` variants. Worker continues to inspect error strings
for sentinel prefixes (`tmdb_auth_required:`, `imdb_waf:`,
`tmdb_rate_limited:`, `imdb_rate_limited:`, `imdb not_found:`).

The frontend error banner shows raw strings; users see what went wrong.

## Testing

Two new test fixtures:

- `src-tauri/tests/fixtures/imdb-suggestion-movie.json` — sample suggestion
  response for "the matrix".
- `src-tauri/tests/fixtures/imdb-suggestion-show.json` — sample for
  "breaking bad".
- `src-tauri/tests/fixtures/imdb-graphql-movie.json` — full GraphQL
  response for a movie.
- `src-tauri/tests/fixtures/imdb-graphql-show.json` — full response for a
  show.
- `src-tauri/tests/fixtures/imdb-graphql-edge.json` — missing
  ratingsSummary, runtime, primaryImage.

Unit tests:

- `metadata/imdb.rs` — parse the five fixtures into the right structs.
  Assert ≥10 fields per fixture, including the missing-field edge case.
- `metadata/queries.rs` — already has 6 tests; add 2 for the new
  sentinel rename (existing `auth_required` rows get rewritten to
  `tmdb_auth_required`).
- `metadata/dispatch.rs` (new file housing `providers_for_mode`) —
  ~10 tests covering every mode × key-present combination. Pure
  function, no DB.
- `metadata/matching.rs` — unchanged; the 10 existing tests cover the
  `MatchCandidate.provider` field passthrough.

Live HTTP tests are skipped, same policy as TMDB.

### Manual verification (per PR)

After all three PRs land:
- Set mode `off` → worker stops dispatching.
- Set mode `tmdb_only` with no key → all newly-scanned rows park with
  `no_provider_available`.
- Set TMDB key while in that state → parked jobs wake and complete.
- Set mode `prefer_tmdb` with key, scan an obscure non-English title
  → TMDB miss → IMDB fills in.
- Set mode `imdb_only` with no key → IMDB takes everything; no parked rows.
- Set mode `prefer_imdb` → IMDB tried first; TMDB called only on IMDB miss.
- Break the TMDB key (paste garbage) → `tmdb_auth_bad` flag appears,
  banner shows, jobs continue via IMDB fallback.
- Open Needs review, switch provider tab on the match sheet → both
  providers searched; pick from either lands on the right
  `(provider, provider_id)` pair.

## Risks

- **IMDB JSON endpoints undocumented** — they could change without
  notice. Cinemagoer, Stremio, JustWatch all depend on them; a sudden
  break would be visible everywhere. Mitigation: log
  `imdb_waf:` errors distinctly so we can spot the drift; users always
  have TMDB as fallback.
- **IMDB TOS clause** — non-commercial use only. Personal apps fine;
  redistributing rustflix would require disabling IMDB. Mitigation:
  module docstring carries the disclaimer; `tmdb_only` mode is
  always functional.
- **Suggestion API geo/edge caching** — CloudFront serves cached
  responses; year-filter variants slug `dune_2021.json` are slow to
  populate after a new title release. Acceptable.
- **GraphQL query stability** — the `principalCredits` filter syntax
  could change. Mitigation: fixture tests fail loudly when the shape
  drifts.
- **Two providers can disagree** — same title gets different cast
  ordering / genre names between TMDB and IMDB. Whichever provider
  wins records its data; user can Unlink and re-link to the other.
  No metadata merging.

## Rollout

Three PRs. Each ships independently. Numbers reflect the next-available
sequence (current max: fix/41).

1. **fix/42 — settings + mode wiring + sentinel rename.**
   - Server-side `validate` + `on_setting_changed` in
     `set_app_setting`.
   - Remove `set_tmdb_api_key` command; consumers route through generic.
   - `default_for(key)` helper + one test per key.
   - `src/lib/settings.ts` frontend wrapper.
   - `metadata_mode` Select on `/settings/metadata` page.
   - `providers_for_mode` helper + ~10 tests.
   - Sentinel rename: `auth_required` → `tmdb_auth_required`; add
     `no_provider_available`. One-shot data fixup in
     `dedupe_shows_and_index`.
   - `metadata_status_counts` grows two new columns; UI renders both.
   - Worker loop refactor: `providers_for_mode` → empty-walk path → park.
   - At end of this PR, behaviour is unchanged for the default mode;
     `imdb_only` and `prefer_imdb` log "not implemented" since the IMDB
     module doesn't exist yet.

2. **fix/43 — IMDB module + dispatch.**
   - `metadata/imdb.rs` with suggestion + GraphQL clients.
   - Fixture-based unit tests for the JSON parsers.
   - `metadata/apply.rs` grows `apply_imdb_*` functions.
   - `dispatch_provider` extraction (worker shrinks).
   - Intra-walk 250ms pacing.
   - `metadata_search`, `link_metadata` take `provider`. Hardcoded
     `'tmdb'` strings removed.
   - `MatchCandidate` gains `provider` field.
   - `imdb_only`, `prefer_imdb`, and `prefer_tmdb` fallback all work.
   - TMDB 401 → set `tmdb_auth_bad` flag, fall through.

3. **fix/44 — match sheet provider toggle + auth-bad banner.**
   - Needs-review match sheet grows TMDB / IMDB tabs. Defaults to the
     primary provider in the active mode.
   - Disabled-tab tooltip when TMDB key missing.
   - `tmdb_auth_bad` banner on `/settings/metadata`.
   - Mode-off banner on `/settings/metadata`.
   - Frontend-only PR aside from the banner state.

## Open questions deferred to implementation

- Should `tmdb_auth_bad` persist across restarts or be cleared on
  startup? Default: persists. User-visible flag; clearing on every
  startup would hide a real config problem until the next sync attempt.
- Should the match sheet retain the user's last-picked provider as a
  per-app default for subsequent matches in the same session? Default:
  no, follow the active mode every time. Revisit if the constant
  re-clicking annoys.
- Worker setting cache. Currently each job re-reads three settings.
  Negligible cost; revisit if real-world profiling says it matters.
- `scrape_language` for TMDB. Currently always `en` in calls; the
  scrape-language picker (in TODO) will plumb the setting through
  without a refactor since the parameter exists in the TMDB module
  signatures already.
