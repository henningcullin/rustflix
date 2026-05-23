#!/usr/bin/env pwsh
#
# Fetches an mpv binary into src-tauri/binaries/ with the correct
# target-triple suffix so Tauri can bundle it as a sidecar.
#
# Strategy (in order):
#   1. If mpv.exe already exists on PATH, copy it.
#   2. Else, run `winget install mpv` and copy from a winget install dir.
#
# Run from anywhere; paths resolve relative to the repo root.

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$binDir = Join-Path $repoRoot "src-tauri/binaries"
$targetTriple = "x86_64-pc-windows-msvc"
$destName = "mpv-$targetTriple.exe"
$dest = Join-Path $binDir $destName

if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Force -Path $binDir | Out-Null
}

function Copy-Mpv($source) {
    Write-Host "Copying $source -> $dest"
    Copy-Item -Force $source $dest
    Write-Host "Done. Sidecar at $dest"
}

# Step 1: existing mpv on PATH
$existing = Get-Command mpv -ErrorAction SilentlyContinue
if ($existing) {
    Copy-Mpv $existing.Source
    exit 0
}

# Step 2: try winget
$winget = Get-Command winget -ErrorAction SilentlyContinue
if (-not $winget) {
    Write-Error @"
No mpv.exe on PATH and winget is not available.
Download mpv manually from https://mpv.io/installation/ and place mpv.exe at:
  $dest
"@
    exit 1
}

Write-Host "Installing mpv via winget…"
# Prefer the official MSVC CI build — it's a per-user zip install (no UAC).
# Fall back to the shinchiro community installer, which triggers UAC because
# it writes to Program Files.
& winget install --id=mpv-player.mpv-CI.MSVC -e --silent --accept-source-agreements --accept-package-agreements
if ($LASTEXITCODE -ne 0) {
    & winget install --id=shinchiro.mpv -e --silent --accept-source-agreements --accept-package-agreements
}

# Re-resolve after install
$existing = Get-Command mpv -ErrorAction SilentlyContinue
if ($existing) {
    Copy-Mpv $existing.Source
    exit 0
}

# Last-ditch: look in common install dirs (PATH may not be refreshed in this session)
$candidates = @(
    "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\mpv-player.mpv-CI.MSVC_*\mpv.exe",
    "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\shinchiro.mpv_*\mpv\mpv.exe",
    "$env:LOCALAPPDATA\Programs\mpv\mpv.exe",
    "C:\Program Files\mpv\mpv.exe",
    "C:\Program Files (x86)\mpv\mpv.exe"
)
foreach ($pattern in $candidates) {
    $found = Get-Item $pattern -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) {
        Copy-Mpv $found.FullName
        exit 0
    }
}

Write-Error @"
Installed mpv but couldn't locate mpv.exe. Find it manually and copy it to:
  $dest
"@
exit 1
