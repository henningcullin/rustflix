# One-command release on Windows. See scripts/release.sh for the bash
# equivalent — this script mirrors it step-for-step so behaviour stays
# in lockstep across the OS split. Usage:
#   .\scripts\release.ps1 patch
#   .\scripts\release.ps1 minor
#   .\scripts\release.ps1 major
#   .\scripts\release.ps1 0.2.0          (explicit, leading "v" optional)

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Target
)

$ErrorActionPreference = 'Stop'

# All text rewrites below go through this encoding so output files don't
# get a UTF-8 BOM and don't get CRLF endings on Windows. The repo's
# .gitattributes file already locks Cargo.lock + .json + .toml to LF,
# but writing them correctly on first pass avoids a noisy `git status`
# in between the script's steps.
$Utf8NoBom = New-Object System.Text.UTF8Encoding $false

function Write-LfText {
    param(
        [string]$Path,
        [string]$Content
    )
    $normalized = $Content -replace "`r`n", "`n"
    [System.IO.File]::WriteAllText($Path, $normalized, $Utf8NoBom)
}

function Show-Usage {
    Write-Host @'
Usage: .\scripts\release.ps1 patch|minor|major|<version>
  patch         bumps Z in X.Y.Z
  minor         bumps Y and zeroes Z
  major         bumps X and zeroes Y + Z
  <version>     explicit semver (e.g. 0.2.0 or v0.2.0-rc.1)
'@
    exit 64
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $repoRoot

$packageJson = 'package.json'
$cargoToml = 'src-tauri/Cargo.toml'
$tauriConf = 'src-tauri/tauri.conf.json'

function Require-CleanTree {
    $status = git status --porcelain
    if ($status) {
        Write-Error "working tree is not clean; commit or stash first`n$status"
    }
}

function Require-OnMaster {
    $branch = (git rev-parse --abbrev-ref HEAD).Trim()
    if ($branch -ne 'master') {
        Write-Error "must be on master to release, currently on '$branch'"
    }
}

function Require-UpToDate {
    git fetch --quiet origin master | Out-Null
    $localSha = (git rev-parse '@').Trim()
    $remoteSha = (git rev-parse 'origin/master').Trim()
    if ($localSha -ne $remoteSha) {
        Write-Error "local master diverges from origin/master; sync first`n  local:  $localSha`n  remote: $remoteSha"
    }
}

function Get-CurrentVersion {
    $pkg = (Get-Content $packageJson -Raw | ConvertFrom-Json).version
    $tauri = (Get-Content $tauriConf -Raw | ConvertFrom-Json).version

    $cargoLine = Select-String -Path $cargoToml -Pattern '^version = "([^"]+)"' | Select-Object -First 1
    if (-not $cargoLine) {
        Write-Error "could not find a version line in $cargoToml"
    }
    $cargo = $cargoLine.Matches[0].Groups[1].Value

    if ($pkg -ne $cargo -or $pkg -ne $tauri) {
        Write-Error "versions drifted between manifests:`n  $packageJson`t$pkg`n  $cargoToml`t$cargo`n  $tauriConf`t$tauri"
    }
    return $pkg
}

function Bump-Part {
    param(
        [string]$Current,
        [string]$Part
    )
    if ($Current -notmatch '^(\d+)\.(\d+)\.(\d+)') {
        Write-Error "current version '$Current' is not semver-shaped"
    }
    [int]$majorPart = $Matches[1]
    [int]$minorPart = $Matches[2]
    [int]$patchPart = $Matches[3]

    switch ($Part) {
        'patch' { $patchPart++ }
        'minor' { $minorPart++; $patchPart = 0 }
        'major' { $majorPart++; $minorPart = 0; $patchPart = 0 }
        default { Write-Error "unknown bump '$Part'" }
    }
    return "$majorPart.$minorPart.$patchPart"
}

function Resolve-Target {
    param([string]$Arg)
    switch -Regex ($Arg) {
        '^(patch|minor|major)$' { return (Bump-Part -Current (Get-CurrentVersion) -Part $Arg) }
        '^[vV](.+)$' { return $Matches[1] }
        '^\d+\.\d+\.\d+([.-].+)?$' { return $Arg }
        default { Show-Usage }
    }
}

function Rewrite-JsonVersion {
    param(
        [string]$Path,
        [string]$NextVersion
    )
    # Delegate to node — Windows PowerShell 5.1's ConvertFrom-Json doesn't
    # preserve key order (it materializes to a PSCustomObject whose
    # properties end up alphabetized), which on the first run rewrote
    # package.json with every key in a new spot and produced a 100-line
    # diff for a one-line version bump. node's JSON.parse preserves
    # insertion order, matching the bash twin exactly.
    # Slice the last two args off process.argv regardless of node's
    # version-dependent layout for `-e`. Pre-Node-20 put '[eval]' at
    # argv[1] and user args at argv[2]+; Node 22 dropped the placeholder
    # and user args start at argv[1]. Pulling from the end sidesteps
    # both.
    $script = @"
const fs = require('fs');
const [path, next] = process.argv.slice(-2);
const obj = JSON.parse(fs.readFileSync(path, 'utf8'));
obj.version = next;
fs.writeFileSync(path, JSON.stringify(obj, null, 2) + '\n');
"@
    node -e $script $Path $NextVersion
    if ($LASTEXITCODE -ne 0) {
        Write-Error "node JSON rewrite failed for $Path"
    }
}

function Rewrite-CargoToml {
    param([string]$NextVersion)
    $lines = Get-Content $cargoToml
    $done = $false
    $rewritten = $lines | ForEach-Object {
        if (-not $done -and $_ -match '^version = "') {
            $done = $true
            "version = `"$NextVersion`""
        } else {
            $_
        }
    }
    Write-LfText -Path $cargoToml -Content (($rewritten -join "`n") + "`n")
}

function Normalize-Lf {
    param([string]$Path)
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -eq 0) { return }
    $text = [System.Text.Encoding]::UTF8.GetString($bytes)
    Write-LfText -Path $Path -Content $text
}

function Regen-CargoLock {
    Push-Location 'src-tauri'
    try {
        cargo update --workspace --quiet
        if ($LASTEXITCODE -ne 0) {
            Write-Error "cargo update failed"
        }
    } finally {
        Pop-Location
    }
    # cargo on Windows writes Cargo.lock with native CRLF terminators
    # even when the repo's .gitattributes says eol=lf — the smudge
    # filter only kicks in on `git checkout`, not on tool writes.
    # Normalize back to LF in-place so the diff stays small.
    Normalize-Lf -Path 'src-tauri/Cargo.lock'
}

function Run-Checks {
    Write-Host '==> pnpm check'
    pnpm check
    if ($LASTEXITCODE -ne 0) { Write-Error "pnpm check failed" }

    Write-Host '==> cargo check (Tauri)'
    Push-Location 'src-tauri'
    try {
        cargo check --quiet
        if ($LASTEXITCODE -ne 0) { Write-Error "cargo check failed" }
    } finally {
        Pop-Location
    }
}

$current = Get-CurrentVersion
$target = Resolve-Target $Target
$tag = "v$target"

if ($target -eq $current) {
    Write-Error "target version ($target) equals current ($current); nothing to bump"
}

Require-CleanTree
Require-OnMaster
Require-UpToDate

$existingTag = git rev-parse --verify --quiet "refs/tags/$tag" 2>$null
if ($LASTEXITCODE -eq 0 -and $existingTag) {
    Write-Error "tag $tag already exists locally"
}

Write-Host "Bumping $current -> $target"

Rewrite-JsonVersion -Path $packageJson -NextVersion $target
Rewrite-JsonVersion -Path $tauriConf -NextVersion $target
Rewrite-CargoToml -NextVersion $target
Regen-CargoLock

Run-Checks

git add $packageJson $cargoToml $tauriConf 'src-tauri/Cargo.lock'
git commit -m $tag
if ($LASTEXITCODE -ne 0) { Write-Error "git commit failed" }

git tag $tag
if ($LASTEXITCODE -ne 0) { Write-Error "git tag failed" }

Write-Host "==> pushing master + tag $tag"
git push origin master
if ($LASTEXITCODE -ne 0) { Write-Error "git push master failed" }
git push origin $tag
if ($LASTEXITCODE -ne 0) { Write-Error "git push tag failed" }

Write-Host ''
Write-Host "Released $tag."
Write-Host 'The Release workflow will pick up the tag and build the bundles.'
