$ErrorActionPreference = 'Stop'

$SourceRepo = 'henningcullin/rustflix'
$ReleasesRepo = 'henningcullin/rustflix-releases'
$KeyPath = Join-Path $env:USERPROFILE '.tauri/rustflix.key'

function Require-Tool($name, $hint) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Write-Error "Missing required tool: $name. $hint"
    }
}

Write-Host '== Tauri updater setup ==' -ForegroundColor Cyan

Require-Tool 'gh' 'Install GitHub CLI: https://cli.github.com/'
Require-Tool 'pnpm' 'Install pnpm: https://pnpm.io/installation'

# gh auth
gh auth status 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Host 'gh CLI is not authenticated. Running `gh auth login`...' -ForegroundColor Yellow
    gh auth login
}

# Generate (or reuse) the signing keypair
if (Test-Path $KeyPath) {
    Write-Host "Signing key already exists at $KeyPath - reusing." -ForegroundColor Green
} else {
    Write-Host "Generating signing key at $KeyPath ..." -ForegroundColor Cyan
    $keyDir = Split-Path $KeyPath -Parent
    if (-not (Test-Path $keyDir)) {
        New-Item -ItemType Directory -Path $keyDir -Force | Out-Null
    }
    pnpm tauri signer generate -w $KeyPath
}

$privateKey = Get-Content $KeyPath -Raw
$publicKey  = Get-Content "$KeyPath.pub" -Raw

# Capture the password set during key generation
$password = Read-Host 'Enter the password you set on the signing key (empty if none)' -AsSecureString
$plainPassword = [System.Net.NetworkCredential]::new('', $password).Password

# Create the public releases repo if it does not yet exist
$repoExists = $false
gh repo view $ReleasesRepo --json name 2>$null | Out-Null
if ($LASTEXITCODE -eq 0) {
    $repoExists = $true
}

if (-not $repoExists) {
    Write-Host "Creating public repo $ReleasesRepo ..." -ForegroundColor Cyan
    gh repo create $ReleasesRepo --public --description 'Public release artifacts for rustflix (do not push code here)'
} else {
    Write-Host "Releases repo $ReleasesRepo already exists - skipping create." -ForegroundColor Green
}

# Fine-grained PAT for the releases repo
Write-Host ''
Write-Host '== Create a fine-grained Personal Access Token ==' -ForegroundColor Cyan
Write-Host 'Open: https://github.com/settings/personal-access-tokens/new'
Write-Host '  - Resource owner: henningcullin'
Write-Host "  - Repository access: Only select repositories -> $ReleasesRepo"
Write-Host '  - Permissions -> Repository permissions -> Contents: Read and write'
Write-Host '  - Expiration: 1 year (or custom)'
Write-Host ''
$pat = Read-Host 'Paste the generated token' -AsSecureString
$plainPat = [System.Net.NetworkCredential]::new('', $pat).Password

# Set Actions secrets on the source repo
Write-Host 'Setting Actions secrets on the source repo ...' -ForegroundColor Cyan
$plainPrivate = $privateKey.Trim()
$plainPublic  = $publicKey.Trim()

$plainPrivate     | gh secret set TAURI_SIGNING_PRIVATE_KEY          --repo $SourceRepo
$plainPassword    | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo $SourceRepo
$plainPat         | gh secret set RELEASES_REPO_TOKEN                --repo $SourceRepo

Write-Host ''
Write-Host '== Done ==' -ForegroundColor Green
Write-Host ''
Write-Host 'Next steps (manual):' -ForegroundColor Cyan
Write-Host '  1. Open src-tauri/tauri.conf.json'
Write-Host '  2. Replace REPLACE_WITH_OUTPUT_OF_setup-updater with:'
Write-Host ''
Write-Host $plainPublic -ForegroundColor Yellow
Write-Host ''
Write-Host '  3. Commit + push.'
Write-Host "  4. Back up $KeyPath somewhere safe (password manager). If you"
Write-Host '     lose it, you can never ship updates to existing installs.'
