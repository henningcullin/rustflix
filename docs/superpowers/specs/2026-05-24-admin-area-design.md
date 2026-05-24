# Admin / registervård area — design

Status: approved 2026-05-24, ready for implementation planning.

## Goal

Add a parallel "admin" surface that lets the user — who is the only user —
do registry-style maintenance on every persistent table in the SQLite DB.
Grid + drawer layout, inline edits for short fields, drawer for long ones,
foreign-key click-through navigation. Coexists with the polished viewing
UI; the viewing UI keeps the quick inline edits it already has.

## Non-goals

- Multi-user / authorization. Single-user desktop app.
- Raw SQL prompt. The user said "not raw" — every interaction goes through
  the typed config and the three generic Tauri commands.
- Audit log / undo. Last-write-wins. The user owns the DB.
- Schema introspection / auto-discovery of new tables. The seven tables we
  know about are enumerated in code; future tables get one config entry.
- Server-side pagination. Client-side search/sort over the full row set is
  good enough for personal-library scale.
- Virtualization. Same reason.
- Keyboard navigation polish (arrow keys, Tab between cells, etc.).
  Deferred to a future polish PR.
- Replacing the existing inline editors. PR #13/#14/#18's editors stay;
  the admin is additive.

## Decisions

1. **Relationship to viewing UI** — coexist. Inline editors on viewing pages
   stay; the admin adds full-table CRUD on top for everything they don't
   reach.
2. **Layout** — grid with inline edits for short fields, drawer for long
   ones (overview, JSON columns) and multi-field changes.
3. **Tables** — all seven persistent tables: `libraries`, `shows`,
   `movies`, `episodes`, `watch_history`, `metadata_jobs`, `app_settings`.
4. **Per-table config** — lean. One `readonly` flag, optional `kind`
   renderer hint, optional `fkTable`/`fkLabel` for foreign keys, optional
   `hideInGrid` for big text fields. Sensible defaults beat opinionated
   config.
5. **Cross-table navigation** — click-through on foreign keys via URL.
   Each FK chip is a link to `/admin/{table}/{id}`. Browser-back works
   naturally. No custom drawer stack.
6. **Discoverability** — a third card in `/settings` (next to Libraries
   and Metadata). Not in the top nav. Two clicks from the home screen.

## Architecture

A new top-level route `/admin/*`:

- `/admin` — table picker. Lists the seven tables with row counts.
- `/admin/[table]` — working surface for one table. Sort, search, and
  selection live in URL query state (`?q=foo&sort=title&dir=desc`).
- `/admin/[table]/[id]` — same surface with the drawer auto-open on that
  row. FK click-throughs land here.

The implementation is **one generic `<DataGrid>` + `<RowDrawer>` pair**,
driven by a per-table config in TypeScript. Adding a new table later is a
config entry plus (if it has fancy columns) one or two custom cell
renderers. No hand-rolled table page per entity.

Backend: **three generic Tauri commands** —
`admin_list_rows`, `admin_update_row`, `admin_delete_rows` — gated by a
Rust `Table` enum that enumerates the seven valid table names. The frontend
deserialises into the enum or fails; that's the SQL-injection guardrail.

## Per-table config

```ts
// src/lib/admin/tables.ts

export interface ColumnConfig {
  key: string;
  label?: string;          // defaults to humanised key
  readonly?: boolean;      // id, added_at, computed counts, fingerprint
  hideInGrid?: boolean;    // long-text fields live in the drawer only
  kind?: 'text' | 'json' | 'datetime' | 'boolean';
  fkTable?: TableId;
  fkLabel?: string;
}

export interface TableConfig {
  id: TableId;
  label: string;
  primaryKey: string;
  defaultSort: { column: string; direction: 'asc' | 'desc' };
  columns: ColumnConfig[];
}

export type TableId =
  | 'libraries' | 'shows' | 'movies' | 'episodes'
  | 'watch_history' | 'metadata_jobs' | 'app_settings';

export const TABLES: Record<TableId, TableConfig> = {
  shows: {
    id: 'shows',
    label: 'Shows',
    primaryKey: 'id',
    defaultSort: { column: 'title', direction: 'asc' },
    columns: [
      { key: 'id',              readonly: true },
      { key: 'title' },
      { key: 'year' },
      { key: 'library_id',      fkTable: 'libraries', fkLabel: 'path' },
      { key: 'provider' },
      { key: 'rating',          readonly: true },
      { key: 'metadata_locked', kind: 'boolean' },
      { key: 'genres',          kind: 'json',     hideInGrid: true },
      { key: 'top_cast',        kind: 'json',     hideInGrid: true },
      { key: 'overview',        kind: 'text',     hideInGrid: true },
      { key: 'folder_path' },
      { key: 'fingerprint',     readonly: true },
      { key: 'added_at',        kind: 'datetime', readonly: true },
    ],
  },
  // libraries, movies, episodes, watch_history, metadata_jobs,
  // app_settings — same shape, ~10 lines each.
};
```

Notes:

- **`readonly` is a UI hint, not a server-side gate.** The backend has no
  per-column readonly check. The UI just doesn't expose readonly columns
  as editable. Hand-firing the Tauri command from devtools bypasses this —
  intentional, the user owns their DB.
- **Foreign-key columns** render as chips showing the related row's
  `fkLabel` value. Clicking the chip navigates to that row in the
  admin.
- **`kind` is optional** — defaults to `text`. Set only when the renderer
  needs to behave differently (boolean checkbox, datetime relative-time
  display, json textarea).
- **JSON columns** are stored as TEXT. The drawer shows them in a textarea.
  Parse-on-save isn't enforced — invalid JSON gets stored as a string,
  and the frontend handles a parse failure by displaying the raw text
  with a "not valid JSON" warning.

## Backend

`src-tauri/src/admin/mod.rs`:

```rust
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Table {
    Libraries, Shows, Movies, Episodes,
    WatchHistory, MetadataJobs, AppSettings,
}

impl Table {
    fn name(&self) -> &'static str { /* "shows", … */ }
    fn primary_key(&self) -> &'static [&'static str] {
        // Vec because watch_history has a composite key.
    }
    fn columns(&self) -> &'static [&'static str] { /* allowlist */ }
}

// admin_list_rows(table, sort_column, direction, search?, limit?, offset?)
//   → Vec<serde_json::Map<String, Value>>

// admin_update_row(table, primary_key_values: Vec<Value>, patch: HashMap<String, Value>)
//   → ()

// admin_delete_rows(table, primary_key_values: Vec<Vec<Value>>)
//   → DeleteOutcome { deleted: usize, cascaded: HashMap<String, usize> }
//   The cascaded map is only populated for `libraries` deletes; for other
//   tables it's empty.
```

### Value binding strategy

Patches arrive as `HashMap<String, serde_json::Value>`. Each value is
matched on its variant and bound:

```rust
match value {
    Value::Bool(b)            => query.bind(if b { 1i64 } else { 0i64 }),
    Value::Number(n) if n.is_i64() => query.bind(n.as_i64().unwrap()),
    Value::Number(n)          => query.bind(n.as_f64().unwrap_or(0.0)),
    Value::String(s)          => query.bind(s),
    Value::Null               => query.bind(Option::<i64>::None),
    Value::Object(_) | Value::Array(_) => unreachable!("frontend serialises JSON columns to String"),
}
```

The frontend always serialises JSON columns to a string before sending
them, so `Object`/`Array` never appear in a patch.

### Reading rows back

`SqliteRow` doesn't have a "to map" API. Walk `row.columns()`, for each
read `try_get_raw(idx)` → `SqliteValueRef` → match `type_info().name()`:

```rust
let json_value = match value_ref.type_info().name() {
    "NULL"    => Value::Null,
    "INTEGER" => Value::from(row.try_get::<i64, _>(idx)?),
    "REAL"    => Value::from(row.try_get::<f64, _>(idx)?),
    "BLOB"    => Value::String(format!("<{} bytes>", …)),  // not used today
    _         => Value::from(row.try_get::<Option<String>, _>(idx)?),
};
```

~20 lines in one helper.

### Cascade semantics

- `DELETE FROM libraries WHERE id = ?` cascades to `shows` and `movies`
  (FK with `ON DELETE CASCADE`). Episodes cascade through shows. The
  `cascaded` count in `DeleteOutcome` is built from a pre-flight query.
- `DELETE FROM shows WHERE id = ?` cascades to episodes via FK.
- **`watch_history` orphans** remain after a cascade — that table has no
  FK to anything. Known limitation, same as the existing
  `queries::delete_show` quirk. The admin documents this rather than
  hand-cleaning per table.
- `metadata_jobs` and `app_settings` deletes affect only their own rows.

### SQL identifier safety

`format!` is used for the table name and column names because sqlx can't
bind identifiers. Both are gated:

- Table names come from the `Table` enum (deserialised, not raw strings).
- Column names in update patches are checked against `Table::columns()`
  before any `format!` runs.

No SQL injection surface despite the `format!`.

## Frontend

Three new generic components plus six small cell sub-components, all
under `src/lib/admin/`:

- **`<DataGrid config={...} rows={...}>`** — owns sort, search, selection.
  Header row built from `config.columns.filter(c => !c.hideInGrid)`. Body
  row delegates to `<Cell>` per column.
- **`<RowDrawer config={...} row={...} onSave onDelete>`** — full edit
  form. Renders every column (including `hideInGrid`). Cell in drawer
  mode: multi-line textarea for text, full JSON textarea for json, etc.
- **`<Cell column={c} value={v} mode="grid"|"drawer" onSave>`** —
  dispatches by `column.kind` to:
  - `<TextCell>` — `<input type="text">`. Enter saves, Escape cancels,
    blur saves. Existing idiom from `HeroBanner` / `EpisodeTitleEditor`.
  - `<IntegerCell>` — `<input type="text" inputmode="numeric">`. **Never**
    `type="number"` — Svelte 5's `bind:value` coerces to `number | undefined`
    and breaks `.trim()`. The cross-cutting review caught this trap
    already; we don't repeat it.
  - `<BooleanCell>` — checkbox.
  - `<DatetimeCell>` — relative-time display ("3 days ago"), ISO on
    hover. Editable as `<input type="text">` accepting either format.
  - `<JsonCell>` — textarea. Parse-on-render so an invalid value still
    displays (as raw text with a warning).
  - `<FkChip table={c.fkTable} id={value} label={...}>` — anchor to
    `/admin/{table}/{id}`. Label fetched lazily via a small
    `admin_fk_label(table, id)` command, cached in a module-level
    `Map<string, string>` for the session.

### Sort, search, selection

- **Sort**: click column header. Direction indicator. Sort state in URL
  query string.
- **Search**: single input above the grid. Filters client-side via
  `JSON.stringify(row).toLowerCase().includes(query.toLowerCase())`. Query
  in URL query string. Works up to ~10k rows.
- **Selection**: checkbox column. Click checks. Range-select with
  Shift-click is deferred.

### Bulk delete

- Counts always shown in confirm dialog ("Delete N rows from shows?").
- For `libraries` deletes, the confirm dialog also shows the cascade
  breakdown ("…cascading to M shows, K movies, J episodes") and requires
  typing the library's path to confirm — typed-confirmation matches the
  delete-the-database idiom.

### Optimistic edits with rollback

Cell save commits locally first, then calls `admin_update_row`. On error
the old value is restored and an error toast appears. Same pattern as
`toggleEpisode` in `series/[id]/+page.svelte`.

### Refresh

A small refresh icon in the grid header re-runs `admin_list_rows`. No
auto-polling. Background processes (e.g., the metadata worker writing
`metadata_synced_at`) only show up after an explicit refresh — accepted
cost for not burning cycles polling.

### Interaction with the metadata worker

Editing `shows.metadata_locked` (or the equivalent on movies) takes effect
on the next worker pass — the worker re-reads the row inside its
transaction. No coordination needed; the worker already treats the DB as
the source of truth. Deleting a row from `metadata_jobs` while a job is
in flight is benign: the worker's commit re-inserts nothing (the row was
already deleted as part of its own transaction). Force-clearing a parked
auth-required job from the admin works the same as the "Refresh metadata"
button: clear the sentinel + reset `next_attempt_at`, and the worker picks
it up on the next notify.

### Discoverability

A third card in the existing `/settings` index page (alongside Libraries
and Metadata), labelled "Admin / Database". Two clicks from the home
screen.

## Errors

No new variants on `AppError`. The three commands surface errors as the
existing opaque `AppError::Other` strings. The frontend renders them in
the page-level error banner that the rest of the app uses.

The known orphan-`watch_history` quirk on cascade delete is documented in
the spec rather than fixed, matching how the rest of the codebase
handles it today.

## Testing

Two layers:

- **Backend binding helpers** — pure functions, real unit tests. Bind
  every `Value` variant + null + nested JSON. Round-trip a row: insert
  via the command, read it back, assert the JSON shape matches. In-memory
  sqlx pool. ~8 cases in `admin/mod.rs`'s `#[cfg(test)]` module.
- **End-to-end manual** — load admin, open each table, edit a row,
  refresh, confirm persistence. Edit a FK column, confirm the chip
  updates. Bulk-delete a few rows. Type-confirm a library delete on a
  scratch library.

Frontend component tests skipped — Svelte 5 component testing is awkward
and the cells are small enough to verify by eye.

## Risks

- **Generic grid is the most ambitious UI in the codebase to date.**
  Existing components are page-level or single-purpose widgets. A
  reusable grid with sort/search/selection/inline-edit is meaningfully
  larger. Single-table MVP in fix/27 is partly to feel the shape before
  building seven of them.
- **Cascade delete on `libraries` is destructive.** The typed-confirmation
  is the only guardrail and it lands in fix/29. Don't ship bulk delete
  before that.
- **JSON column round-trip data loss.** Pasting invalid JSON and saving
  puts the column in a string state. The frontend handles this by
  rendering raw text + a "not valid JSON" warning rather than crashing.
- **No compile-time type safety on row payloads.** `Map<string, unknown>`
  means a typo in a config column key fails silently (cell renders
  empty). Mitigation: dev-mode console warning that compares
  `Object.keys(row)` against `config.columns.map(c => c.key)` on first
  render.
- **Dependency on metadata-sync tables.** The configs for `metadata_jobs`
  and `app_settings` reference tables that only exist after PR #23. Admin
  rollout is gated behind metadata-sync.

## Rollout

Four PRs, each shippable on its own. This spec is fix/26; implementation
follows the metadata-sync PRs (#23–25). Branches:

1. **fix/27 — admin backend.** `src-tauri/src/admin/mod.rs` with `Table`
   enum + the three commands + binding helpers + row-to-Map helper.
   `#[cfg(test)]` tests on the binding helpers. Registered in `lib.rs`.
   No frontend.
2. **fix/28 — admin MVP UI.** `/admin` table picker + `/admin/shows` grid
   (one full config). Inline cell editing. No drawer yet, no FK chips,
   no search. First usable surface.
3. **fix/29 — drawer + remaining tables + FK.** `<RowDrawer>`. Configs for
   the other six tables. FK chips with the label cache. URL routing for
   `/admin/[table]/[id]`. End-to-end edit flow.
4. **fix/30 — search, sort, selection, bulk delete.** Client-side search
   input, sortable headers, checkbox selection, bulk delete button.
   Typed-confirmation for `libraries`.

After fix/28 there's already *something usable* for shows. Stop after
fix/29 and you have a functional admin, just clunkier. fix/30 is polish.

## Open questions deferred to implementation

- Whether the FK label cache needs an LRU. Default: unbounded `Map` per
  session; revisit if memory bloat shows up (it won't, the cache is
  bounded by table sizes).
- Exact width hints per column in the grid. Defaults derived from `kind`;
  refine during fix/27 once the shows grid feels right.
- Whether the `metadata_jobs` row should be deletable from the admin at
  all, or only "Refresh" / "Park" / "Wake". Default: full delete because
  the goal is letting the user nuke stuck state. Revisit if accidents
  happen.
