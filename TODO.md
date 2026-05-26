# TODO

Working roadmap. Each item becomes one `fix/N-slug` branch + PR. Items get checked off as the corresponding PR is merged into master.

## Active queue

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
