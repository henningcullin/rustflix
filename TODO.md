# TODO

Working roadmap. Each item becomes one `fix/N-slug` branch + PR. Items get checked off as the corresponding PR is merged into master.

## Active queue

- [ ] **Posters 404 in dev (and likely prod)** — after a metadata sync, `<img src={posterPath}>` resolves relative to the current URL (`/show-52.jpg`, `/series/52/show-52.jpg`). `apply.rs` writes `poster_path` as a bare filename like `show-52.jpg`; manual uploads write a Windows absolute path. Browsers can't load either as `src`. Pick: (a) Tauri asset protocol + `convertFileSrc` on every `posterPath` consumer, or (b) backend command that returns the bytes / a data URL. Affects PosterCard, the edit pages, and the backdrop on the detail pages.

- [ ] **Three Rust dead-code warnings** — clean up:
    - `ReleaseYearNode.end_year` in `metadata/imdb.rs` (parsed but never used — could feed a future "ended in YYYY" UI; for now `#[allow(dead_code)]` or drop the field)
    - `PosterSize::Hero` variant — added for future backdrop support; mark `#[allow(dead_code)]` or remove until needed
    - `queries::default_for` — defined in PR #41 but never called; either wire it into a defaults UI or remove

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
