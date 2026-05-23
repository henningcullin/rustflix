# Sidecar binaries

Tauri bundles every file referenced in `tauri.conf.json -> bundle.externalBin`
next to the main `rustflix` executable, and at runtime they sit beside the
running app binary. The Rust side resolves them via `current_exe()` + the
target-triple suffix.

## mpv

Drop the `mpv` executable here with the host's target triple suffix:

| Target                          | Filename                             |
| ------------------------------- | ------------------------------------ |
| Windows x86_64                  | `mpv-x86_64-pc-windows-msvc.exe`     |
| Windows aarch64                 | `mpv-aarch64-pc-windows-msvc.exe`    |
| macOS Intel                     | `mpv-x86_64-apple-darwin`            |
| macOS Apple Silicon             | `mpv-aarch64-apple-darwin`           |
| Linux x86_64 (GNU)              | `mpv-x86_64-unknown-linux-gnu`       |

### Easiest path (Windows)

1. `winget install mpv` — installs mpv globally
2. Locate `mpv.exe` (usually under
   `%LOCALAPPDATA%\Microsoft\WinGet\Packages\mpv.net_...\mpv\` or
   `C:\Program Files\mpv\`)
3. Copy it here and rename it to `mpv-x86_64-pc-windows-msvc.exe`

Or run `scripts/fetch-mpv.ps1` from the repo root to do it automatically.

### Manual download

Download a recent stable build from [mpv.io](https://mpv.io/installation/) or
the [shinchiro builds on GitHub](https://github.com/shinchiro/mpv-winbuild-cmake/releases),
extract `mpv.exe`, and rename it as above.
