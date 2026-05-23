#!/usr/bin/env bash
set -euo pipefail

SOURCE_REPO="henningcullin/rustflix"
RELEASES_REPO="henningcullin/rustflix-releases"
KEY_PATH="$HOME/.tauri/rustflix.key"

require_tool() {
    local name="$1"
    local hint="$2"
    if ! command -v "$name" >/dev/null 2>&1; then
        echo "ERROR: Missing required tool: $name. $hint" >&2
        exit 1
    fi
}

printf '\033[36m== Tauri updater setup ==\033[0m\n'

require_tool gh   'Install GitHub CLI: https://cli.github.com/'
require_tool pnpm 'Install pnpm: https://pnpm.io/installation'

# gh auth
if ! gh auth status >/dev/null 2>&1; then
    printf '\033[33mgh CLI is not authenticated. Running `gh auth login`...\033[0m\n'
    gh auth login
fi

# Generate (or reuse) the signing keypair
if [[ -f "$KEY_PATH" ]]; then
    printf '\033[32mSigning key already exists at %s - reusing.\033[0m\n' "$KEY_PATH"
else
    printf '\033[36mGenerating signing key at %s ...\033[0m\n' "$KEY_PATH"
    mkdir -p "$(dirname "$KEY_PATH")"
    # pnpm dlx fetches a fresh Tauri CLI for the host platform instead of
    # using the project's node_modules, which on this repo gets installed
    # from PowerShell and therefore only carries the Windows native binding.
    pnpm dlx "@tauri-apps/cli@^2" signer generate -w "$KEY_PATH"
fi

PRIVATE_KEY="$(cat "$KEY_PATH")"
PUBLIC_KEY="$(cat "${KEY_PATH}.pub")"

# Capture the password set during key generation
echo
read -r -s -p 'Enter the password you set on the signing key (empty if none): ' KEY_PASSWORD
echo

# Create the public releases repo if it does not yet exist
if gh repo view "$RELEASES_REPO" --json name >/dev/null 2>&1; then
    printf '\033[32mReleases repo %s already exists - skipping create.\033[0m\n' "$RELEASES_REPO"
else
    printf '\033[36mCreating public repo %s ...\033[0m\n' "$RELEASES_REPO"
    gh repo create "$RELEASES_REPO" \
        --public \
        --description 'Public release artifacts for rustflix (do not push code here)'
fi

# Fine-grained PAT for the releases repo
echo
printf '\033[36m== Create a fine-grained Personal Access Token ==\033[0m\n'
echo 'Open: https://github.com/settings/personal-access-tokens/new'
echo '  - Resource owner: henningcullin'
echo "  - Repository access: Only select repositories -> $RELEASES_REPO"
echo '  - Permissions -> Repository permissions -> Contents: Read and write'
echo '  - Expiration: 1 year (or custom)'
echo
read -r -s -p 'Paste the generated token: ' PAT
echo

# Set Actions secrets on the source repo
printf '\033[36mSetting Actions secrets on the source repo ...\033[0m\n'
printf '%s' "$PRIVATE_KEY"  | gh secret set TAURI_SIGNING_PRIVATE_KEY          --repo "$SOURCE_REPO"
printf '%s' "$KEY_PASSWORD" | gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --repo "$SOURCE_REPO"
printf '%s' "$PAT"          | gh secret set RELEASES_REPO_TOKEN                --repo "$SOURCE_REPO"

echo
printf '\033[32m== Done ==\033[0m\n'
echo
printf '\033[36mNext steps (manual):\033[0m\n'
echo '  1. Open src-tauri/tauri.conf.json'
echo '  2. Replace REPLACE_WITH_OUTPUT_OF_setup-updater with:'
echo
printf '\033[33m%s\033[0m\n' "$PUBLIC_KEY"
echo
echo '  3. Commit + push.'
echo "  4. Back up $KEY_PATH (and ${KEY_PATH}.pub) somewhere safe (password"
echo '     manager). If you lose them, you can never ship updates to existing'
echo '     installs.'
