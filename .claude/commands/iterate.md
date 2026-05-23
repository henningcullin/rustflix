---
description: Iterate through a list of changes — one branch + PR + merge per item
---

You will execute a series of changes provided by the user, one at a time. Each item becomes its own branch, PR, and merge commit on master.

## Preflight (run once, before the first item)

1. Confirm `gh` is installed and authenticated: `gh auth status`. If not, abort and tell the user to run `sudo apt install gh && gh auth login`.
2. Confirm clean working tree: `git status --porcelain` must be empty. If dirty, abort and ask the user how to proceed.
3. Sync master: `git checkout master && git pull`.
4. Determine the starting branch number: `git branch -a --list 'origin/fix/*' 'fix/*'` and parse the highest N from `fix/N-*` patterns. Next item uses N+1. (If no `fix/*` branches exist, start at 1.)

## Per-item loop

For each item in the list, in order:

1. **Slug** — generate a kebab-case slug from the item title (3–5 words, lowercase, hyphens, no punctuation).
2. **Branch** — `git checkout -b fix/N-slug` from latest master. Increment N for the next item.
3. **Implement** — make the changes. **If the item has logically separate parts, make multiple focused commits** (one per part). Don't lump unrelated work into one commit.
4. **Push** — `git push -u origin fix/N-slug`.
5. **Open PR** — `gh pr create` using `.github/pull_request_template.md`. Title under 70 chars, imperative mood (e.g. "Fix Monocraft toggle persistence"). Body fills in Summary / Why / Test plan from the item context.
6. **Merge** — `gh pr merge --merge --delete-branch`. **Always `--merge`** (keeps the per-part commits and adds a merge commit). Never `--squash` or `--rebase`.
7. **Return to master** — `git checkout master && git pull`.
8. **Continue** to the next item.

## Hard rules

- Never skip the master sync between items — every branch must stem from the latest master.
- Never use `--squash` or `--rebase` on merge — always `--merge`.
- If any step fails (push rejected, PR check failure, merge conflict, etc.), **STOP** and report the failure. Do not silently continue to the next item.
- Don't batch multiple items into one branch, even if they look related. One item = one branch = one PR.
- Follow the project's CLAUDE.md coding rules for every change.

## The list

$ARGUMENTS
