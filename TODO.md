# TODO

Working roadmap. Each item becomes one `fix/N-slug` branch + PR. Items get checked off as the corresponding PR is merged into master.

## Active queue

- [ ] **Render cast + genres on detail pages** — DB columns `genres` (JSON array of strings) and `top_cast` (JSON array of `{name, character, order}`) are populated by both TMDB (`apply_movie_details`) and IMDB (`apply_imdb_movie_details`). The detail pages `src/routes/films/[id]/+page.svelte` and `src/routes/series/[id]/+page.svelte` don't render either. Plan: chip row for genres under the title; small cast grid (name + character) below the overview. Movies + shows both need it; ship movies first if shows need a layout rethink.

- [ ] **Episode titles from metadata sync** — `src-tauri/src/metadata/apply.rs` only writes to `movies` and `shows`. The `episodes` table is never touched by sync, so episode names remain whatever the scanner pulled from the file path (affects TMDB sync too — pre-existing gap). Plan: add an "episodes pass" to the worker after the show-level apply — fetch each season, match by `(season_number, episode_number)`, UPDATE `episodes.title` only when the existing value looks scanner-generated (e.g. matches the file basename, so user-edited titles survive). Sources: IMDB GraphQL — add `episodes(first, after) { edges { node { titleText releaseYear } } }` to the query in `metadata/imdb.rs`; TMDB — `/tv/{id}/season/{n}` returns `name` per episode.

- [ ] **Hero-size posters / backdrops on detail pages** — `PosterSize::Hero` at `src-tauri/src/metadata/imdb.rs:380` already exists (URL segment `_V1_QL90_UX1280_`) but nothing constructs it. Plan: add a `backdrop_path` column to `movies` and `shows` (migration), have `apply.rs` download both Small (poster) and Hero (backdrop) variants — extra ~300 KB per title — and have `src/lib/components/HeroBanner.svelte` prefer `backdrop_path` and fall back to `poster_path`. TMDB has a separate `backdrop_path` field on the details payload that we currently discard; the TMDB hero URL is `https://image.tmdb.org/t/p/w1280<backdrop_path>`. Lights up TMDB too.

- [ ] **Netflix-style playback UX** — bundle of interactions. Foundation already exists: `progress_seconds` on Movie/Episode, `play_movie(id, resume?)` and `play_episode(id, resume?)` accept an optional offset, `continueWatching()` resolves next-up. Sub-items:
    1. **Split card click** — `src/lib/components/PosterCard.svelte` wraps the whole card in `<a href>`, so the play-icon overlay just navigates. Give the overlay its own click handler with `event.stopPropagation()` that calls `playMovie` / `playEpisode` directly; card body click → detail page (unchanged).
    2. **"Current episode" resolver for shows** — clicking play on a series card needs to pick the right episode without bouncing through the detail page. Logic: most recent episode with `progress_seconds > 0` (resume there) → else next unwatched after the last completed → else S1E1. New Tauri command `get_current_episode(show_id)` returning the chosen episode id + resume offset.
    3. **Resume vs Start-over toggle on detail pages** — small button next to the big Play CTA on `/films/[id]` and `/series/[id]`. Default = resume from `progress_seconds`; Start over passes `resume=0`. Trivial once the button is in place.
    4. **Verify mpv → progress reporting** — open question: does `src-tauri/src/player.rs` write `progress_seconds` continuously while mpv is running, or only at session close? Inspect before relying on accurate resume points — session-close-only means the resume offset is stale by the length of the last playback. Fix if needed (mpv IPC has `time-pos` events).

- [ ] **Matcher tuning** — `matching::pick_confident_match` in `src-tauri/src/metadata/matching.rs` rejects some IMDB-mode movies the user expects to match. Needs concrete failing titles to tune against; ask the user for 3-5 examples that ended up in Needs-review, walk each through `metadata_search` on both providers (DevTools console), inspect the candidate list, then adjust scoring weights. Blind quick win worth trying first: relax the year-mismatch penalty so off-by-one (release year vs scan year) doesn't kill a strong title match.

- [ ] **IMDB-fallback post-merge verification** — walk these against a real library now that PRs #41 / #42 / #43 / #45 / #46 are on master:
    - Pick each of the five `metadata_mode` values (`off`, `tmdb_only`, `imdb_only`, `prefer_tmdb`, `prefer_imdb`) and confirm the worker behaves per spec.
    - Add a known-obscure non-English title to the library; verify TMDB miss → IMDB fallback in `prefer_tmdb` mode.
    - Hand-link a row to the *other* provider via the match sheet; verify the worker's fast path uses that provider on the next pass regardless of mode.
    - Break the TMDB key (paste garbage, save), force a sync, observe the yellow banner on Settings → Metadata. Save a real key → banner clears.
    - Switch modes between IMDB-touching and TMDB-only states and watch parked jobs wake (no orphaned `no_provider_available` rows).
    - Refresh a row via the detail page; verify `metadata_locked` clears and the worker re-fetches via the linked provider.

- [ ] **Dependabot — `cookie` low-severity alert open on master** — https://github.com/henningcullin/rustflix/security/dependabot/64. Out-of-bounds chars accepted in cookie name/path/domain. Transitive via SvelteKit. Take the patched range when Dependabot opens the auto-PR or bump manually if the auto-PR doesn't appear.

- [ ] **Stranger Things seasons fail to merge** — investigate. Probably either an episode-conflict path or a (season, episode) collision the merge sheet doesn't surface. Reproduce, then file a `fix/N` once we know the cause.

- [ ] **UI language picker (en / sv)** — pick the language the app's own UI renders in. Requires an i18n library (probably `svelte-i18n` or `paraglide`). Settings → Appearance area when it lands. Default `en`. Out of scope for the IMDB fallback work; surface in `/settings/appearance` (new card) when built. Will need a translation pass on every page; budget accordingly.

- [ ] **Scrape language picker (en / sv)** — separate from the UI language. Controls which locale we ask TMDB / IMDB for. Today the IMDB module pins `lc-main=en_US`; the future setting overrides that. TMDB takes a `language` query param. Stored as `scrape_language` in `app_settings`, default `en`. The IMDB / TMDB scraper signatures should take a `language: &str` parameter from day one (per the metadata-mode spec) so this lands without a refactor.

- [ ] **i18n infrastructure** — pick a library, wire it into the SvelteKit build, extract a string-catalog convention. This blocks the UI language picker. Worth its own design spec when it's time.

- [ ] **Settings → Appearance area** — empty card today, but the home for the UI language picker, the dark/light/system theme picker, and any future presentation settings.

- [ ] **Theme picker (system / light / dark)** — separate from language. Same area (Appearance). Stored as `theme` in `app_settings`. Trivial once the area exists; just a Select wired to a CSS class on `<html>`.

## Done

- [x] **fix/16 — Delete series entry from library** (PR #16)
- [x] **fix/17 — Skip already-imported files on rescan** (PR #17)
- [x] **fix/18 — Edit episode title names** (PR #18)
- [x] **fix/19 — Fix recently-added card hover border clipping** (PR #19)
- [x] **fix/20 — Replace settings auto-detect Select with shadcn-svelte Select** (PR #20)
- [x] **fix/44 — Settings infrastructure, 5-mode picker, sentinel rename** (PR #41) — generic `get_app_setting` / `set_app_setting`, `metadata_mode` enum, `tmdb_auth_required` + `no_provider_available` sentinels, 6-card status panel
- [x] **fix/45 — IMDB module + worker dispatch + hand-link fast path** (PR #42) — suggestion API + GraphQL client, `apply_imdb_*`, real `providers_for_mode` walks, `dispatch_imdb`, hand-linked fast path, `provider` param on `metadata_search` / `link_metadata`
- [x] **fix/46 — Match-sheet provider toggle + auth-bad banner** (PR #43) — TMDB / IMDB tabs on the Needs-review sheet, `tmdb_auth_bad` banner on Settings → Metadata
- [x] **fix/47 — Track TODO.md + log poster-404 / dead-code items** (PR #44) — TODO.md was locally ignored via `.git/info/exclude`; brought into the repo
- [x] **fix/48 — Posters via Tauri asset protocol** (PR #45) — backend normalises bare filenames to `$APPDATA/posters/...`, frontend wraps with `convertFileSrc` via new `posterUrl()` helper
- [x] **fix/49 — Clear three Rust dead-code warnings** (PR #46) — `#[allow(dead_code)]` on `ReleaseYearNode.end_year`, `PosterSize::Hero`, `queries::default_for`
