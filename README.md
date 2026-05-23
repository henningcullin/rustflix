# Rustflix

A Netflix-style local-library media app. Point it at folders of movies and series
on your disk; Rustflix indexes them, tracks watched progress, and plays them
through a bundled `mpv`.

Built with Tauri 2, SvelteKit (Svelte 5 runes), Tailwind, and `sqlx` over SQLite.

## Requirements

- **Node.js** 20+
- **Rust** stable, MSVC toolchain on Windows (`rustup default stable-x86_64-pc-windows-msvc`)
- **Microsoft C++ Build Tools** ("Desktop development with C++" workload) on Windows
- **WebView2** runtime (preinstalled on modern Windows)
- **mpv binary** — see _Bundling mpv_ below; only needed once at build time

## Run / build

```sh
pnpm install
pnpm tauri:dev         # dev (hot reload)
pnpm tauri:build       # release installer
```

## Bundling mpv

Rustflix ships `mpv` as a Tauri sidecar so end users don't need to install it.
You drop the binary into `src-tauri/binaries/` once; Tauri then bakes it into the
installer.

The expected name encodes the target triple, e.g. on Windows x86_64:

```
src-tauri/binaries/mpv-x86_64-pc-windows-msvc.exe
```

Either run the helper script:

```powershell
pwsh ./scripts/fetch-mpv.ps1
```

…or follow the manual instructions in `src-tauri/binaries/README.md`.

## Releases

Releases are tag-driven. `pnpm release patch` (or `release:ps` on Windows) bumps
the version across `package.json` + `Cargo.toml` + `tauri.conf.json`,
regenerates `Cargo.lock`, sanity-checks both builds, commits, tags, and pushes.
The `.github/workflows/release.yml` workflow then builds, signs with the
updater key, and publishes installers + `latest.json` to a separate
`henningcullin/rustflix-releases` repo.

First-time setup (generates the signing key, creates the releases repo, sets
GitHub Actions secrets) is in `scripts/SETUP-UPDATER.md`.

## Conventions

- Coding rules: `CLAUDE.md`
- Branch / PR / merge workflow for iterating on a list of changes:
  `.claude/commands/iterate.md` (invoke with `/iterate`)
- PR template: `.github/pull_request_template.md`

## Project layout

```
src/                       SvelteKit frontend
  lib/api.ts               Typed wrapper around invoke()
  lib/components/          Hero, MediaRow, PosterCard, TopNav
  routes/                  Home, /films, /series, settings
src-tauri/
  src/                     Rust backend (commands, queries, scanner, player)
  migrations/              sqlx schema (applied at startup)
  binaries/                mpv sidecar lives here
scripts/                   Build-time + release helpers
.github/workflows/         CI/CD (release.yml)
.claude/commands/          Project slash commands (/iterate)
```
