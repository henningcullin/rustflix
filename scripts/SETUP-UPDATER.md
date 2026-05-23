# Updater setup

One-time setup for the Tauri auto-updater. Two equivalent scripts ship — pick whichever matches where your `gh` CLI lives:

**WSL / Linux / macOS:**
```bash
bash scripts/setup-updater.sh
```

**Windows PowerShell:**
```powershell
powershell -ExecutionPolicy Bypass -File scripts/setup-updater.ps1
```

## What it does

1. Verifies `gh` (GitHub CLI) and `pnpm` are installed.
2. Authenticates `gh` if needed.
3. Generates an Ed25519 signing keypair at `~/.tauri/rustflix.key` (+ `.pub`).
4. Creates the public `henningcullin/rustflix-releases` repo (skipped if it already exists).
5. Walks you through creating a fine-grained PAT scoped to that repo.
6. Sets three Actions secrets on the source repo:
   - `TAURI_SIGNING_PRIVATE_KEY`
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   - `RELEASES_REPO_TOKEN`
7. Prints the public key — paste it into `src-tauri/tauri.conf.json` under `plugins.updater.pubkey`.

## After running

- Replace the `REPLACE_WITH_OUTPUT_OF_setup-updater` placeholder in `tauri.conf.json` with the printed pubkey, commit, push.
- Back up `~/.tauri/rustflix.key` to a password manager. Losing it means losing the ability to ship updates to existing installs forever.

## Releasing

Every release after setup runs through one command:

**WSL / Linux / macOS:**
```bash
pnpm release patch       # or minor / major / 0.2.0
```

**Windows PowerShell:**
```powershell
pnpm release:ps patch    # or minor / major / 0.2.0
```

The script bumps `package.json` + `src-tauri/Cargo.toml` + `src-tauri/tauri.conf.json` in lockstep, regenerates `Cargo.lock`, runs `pnpm check` + `cargo check`, makes a single `vX.Y.Z` commit on master, tags it, and pushes both. See `scripts/release.sh` for the source of truth.

The Release workflow then runs the build matrix (currently Windows only — uncomment the macOS/Linux blocks in `.github/workflows/release.yml` to expand), signs each bundle with the same key, and uploads them alongside a unified `latest.json` whose `platforms` map carries one entry per OS. The app picks up the new version on next launch via `tauri-plugin-updater`.
