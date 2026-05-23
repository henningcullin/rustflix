#!/usr/bin/env bash
# One-command release: bump version in package.json + Cargo.toml +
# tauri.conf.json, regenerate Cargo.lock, sanity-check builds, commit,
# tag, push. Usage:
#   scripts/release.sh patch
#   scripts/release.sh minor
#   scripts/release.sh major
#   scripts/release.sh 0.2.0          (explicit, leading "v" optional)
#
# Assumes master is the release branch and the working tree is clean.

set -euo pipefail

usage() {
    cat >&2 <<EOF
Usage: $0 patch|minor|major|<version>
  patch         bumps Z in X.Y.Z
  minor         bumps Y and zeroes Z
  major         bumps X and zeroes Y + Z
  <version>     explicit semver (e.g. 0.2.0 or v0.2.0-rc.1)
EOF
    exit 64
}

[ $# -eq 1 ] || usage

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo_root"

package_json="package.json"
cargo_toml="src-tauri/Cargo.toml"
tauri_conf="src-tauri/tauri.conf.json"

require_clean_tree() {
    if [ -n "$(git status --porcelain)" ]; then
        echo "ERROR: working tree is not clean; commit or stash first" >&2
        git status --short >&2
        exit 1
    fi
}

require_on_master() {
    local branch
    branch="$(git rev-parse --abbrev-ref HEAD)"
    if [ "$branch" != "master" ]; then
        echo "ERROR: must be on master to release, currently on '$branch'" >&2
        exit 1
    fi
}

require_up_to_date() {
    git fetch --quiet origin master
    local local_sha remote_sha
    local_sha="$(git rev-parse @)"
    remote_sha="$(git rev-parse @{u} 2>/dev/null || git rev-parse origin/master)"
    if [ "$local_sha" != "$remote_sha" ]; then
        echo "ERROR: local master diverges from origin/master; sync first" >&2
        echo "  local:  $local_sha" >&2
        echo "  remote: $remote_sha" >&2
        exit 1
    fi
}

current_version() {
    # Read directly from package.json (it's the source of truth for what
    # the panel ships as). The three files must already be in lockstep
    # going in — if they aren't, abort rather than silently picking one.
    local pkg cargo tauri
    pkg="$(node -p "require('./$package_json').version")"
    cargo="$(grep -m1 -E '^version = ' "$cargo_toml" | sed -E 's/^version = "(.*)"$/\1/')"
    tauri="$(node -p "require('./$tauri_conf').version")"
    if [ "$pkg" != "$cargo" ] || [ "$pkg" != "$tauri" ]; then
        echo "ERROR: versions drifted between manifests" >&2
        printf '  %-30s %s\n' "$package_json" "$pkg" >&2
        printf '  %-30s %s\n' "$cargo_toml" "$cargo" >&2
        printf '  %-30s %s\n' "$tauri_conf" "$tauri" >&2
        exit 1
    fi
    echo "$pkg"
}

bump_part() {
    local current="$1"
    local part="$2"
    local core
    core="$(echo "$current" | sed -E 's/^([0-9]+\.[0-9]+\.[0-9]+).*/\1/')"
    local major minor patch
    IFS=. read -r major minor patch <<< "$core"
    case "$part" in
        patch) patch=$((patch + 1)) ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        major) major=$((major + 1)); minor=0; patch=0 ;;
        *) echo "ERROR: unknown bump '$part'" >&2; exit 64 ;;
    esac
    echo "$major.$minor.$patch"
}

resolve_target() {
    local arg="$1"
    case "$arg" in
        patch|minor|major)
            bump_part "$(current_version)" "$arg"
            ;;
        v*|V*)
            echo "${arg#[vV]}"
            ;;
        *)
            # Treat any other arg as an explicit version literal; validate shape.
            if [[ ! "$arg" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-].+)?$ ]]; then
                echo "ERROR: '$arg' is not a recognised version" >&2
                usage
            fi
            echo "$arg"
            ;;
    esac
}

rewrite_package_json() {
    local next="$1"
    node -e "
        const fs = require('fs');
        const path = '$package_json';
        const pkg = JSON.parse(fs.readFileSync(path, 'utf8'));
        pkg.version = '$next';
        fs.writeFileSync(path, JSON.stringify(pkg, null, 2) + '\n');
    "
}

rewrite_tauri_conf() {
    local next="$1"
    node -e "
        const fs = require('fs');
        const path = '$tauri_conf';
        const conf = JSON.parse(fs.readFileSync(path, 'utf8'));
        conf.version = '$next';
        fs.writeFileSync(path, JSON.stringify(conf, null, 2) + '\n');
    "
}

rewrite_cargo_toml() {
    local next="$1"
    # Restrict the rewrite to the [package] block's first `version =` so a
    # `version = "..."` line in a `[dependencies]` entry can never be hit.
    # GNU sed and BSD sed both accept `0,/.../{ s/old/new/ }` for "first
    # match only", so this works on Linux and macOS without `sed -i`.
    local tmp
    tmp="$(mktemp)"
    awk -v next="$next" '
        BEGIN { done = 0 }
        /^version = "/ && !done { print "version = \"" next "\""; done = 1; next }
        { print }
    ' "$cargo_toml" > "$tmp"
    mv "$tmp" "$cargo_toml"
}

regen_cargo_lock() {
    # Rewriting Cargo.toml drops the package version out of sync with the
    # lock. `cargo update --workspace` keeps Cargo.lock authoritative
    # without bumping any third-party deps.
    (cd src-tauri && cargo update --workspace --quiet)
}

run_checks() {
    echo "==> pnpm check"
    pnpm check
    echo "==> cargo check (Tauri)"
    (cd src-tauri && cargo check --quiet)
}

current="$(current_version)"
target="$(resolve_target "$1")"
tag="v$target"

if [ "$target" = "$current" ]; then
    echo "ERROR: target version ($target) equals current ($current); nothing to bump" >&2
    exit 1
fi

require_clean_tree
require_on_master
require_up_to_date

if git rev-parse --verify --quiet "refs/tags/$tag" >/dev/null; then
    echo "ERROR: tag $tag already exists locally" >&2
    exit 1
fi

echo "Bumping $current -> $target"

rewrite_package_json "$target"
rewrite_tauri_conf "$target"
rewrite_cargo_toml "$target"
regen_cargo_lock

run_checks

git add "$package_json" "$cargo_toml" "$tauri_conf" "src-tauri/Cargo.lock"
git commit -m "$tag"
git tag "$tag"

echo "==> pushing master + tag $tag"
git push origin master
git push origin "$tag"

echo
echo "Released $tag."
echo "The Release workflow will pick up the tag and build the bundles."
