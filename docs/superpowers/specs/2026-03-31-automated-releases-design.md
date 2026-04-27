# Automated Release Workflow

## Summary

A Forgejo Actions workflow that automatically creates a git tag and Forgejo release when the version in `api/Cargo.toml` changes on `main`. Release notes are generated from commit messages between the previous tag and HEAD.

## Trigger

```yaml
on:
  push:
    branches:
      - main
    paths:
      - api/Cargo.toml
```

Fires when a commit landing on `main` touches `api/Cargo.toml`. The version in `api/Cargo.toml` is the single source of truth for the project version.

## Workflow Steps

### 1. Checkout with full history

`fetch-depth: 0` so all tags and commit history are available for `git log` and `git tag`.

### 2. Extract version from Cargo.toml

Use Nushell's native TOML parsing:

```nu
open api/Cargo.toml | get package.version
```

This returns only the `[package]` version, avoiding false matches on dependency version fields.

### 3. Validate semver format

Reject if the extracted version does not match the `X.Y.Z` pattern (digits only, three components). This prevents creating tags from malformed or empty version strings.

### 4. Tag-exists guard

If the tag `vX.Y.Z` already exists in the repo, exit early with success. This handles:
- Dependency-only edits to `Cargo.toml` (version unchanged)
- Workflow reruns
- Rapid successive pushes

### 5. Version-greater-than guard

Compare the extracted version against the latest existing tag. If the new version is not strictly greater (by semver ordering), exit with failure. This prevents:
- Accidental version downgrades
- Typos that produce a lower version number

### 6. Find previous tag

The most recent tag by version sort, used to determine the commit range for release notes.

### 7. Build release notes

```
git log <prev_tag>..HEAD --no-merges --pretty=format:"- %s (%h)"
```

- `--no-merges` excludes merge commits, keeping only work commits
- Format: `- <commit message> (<short hash>)` as a bulleted list
- If no previous tag exists, include all commits

### 8. Create tag and release via API

Single API call to `POST /api/v1/repos/{owner}/{repo}/releases` with:
- `tag_name`: the `vX.Y.Z` tag (created implicitly on HEAD by the API)
- `name`: same as tag
- `body`: the generated release notes
- `draft`: false
- `prerelease`: false

## Authentication

Uses the built-in `${{ secrets.FORGEJO_TOKEN }}` workflow token. No PAT required. This token has write access to the repository and is scoped to the workflow run.

The built-in token will not trigger other workflows as a side effect of creating the tag. However, the image build workflows (`build-api.yml`, `build-frontend.yml`) will already fire from the `main` push that triggered this release workflow, so images are built regardless.

## Runner and Tooling

- Runner: `${{ vars.RUNS_ON_OPENSUSE_BASE_LATEST }}` (consistent with existing workflows)
- All scripting in Nushell (`shell: nu {0}`)
- JSON payload construction uses Nushell's native `to json` instead of Python

## File

`.forgejo/workflows/release.yml`

## Failure Modes Considered

| Scenario | Outcome |
|---|---|
| Dependency-only Cargo.toml edit | Tag exists guard exits early |
| Wrong version line matched | Impossible: TOML-native parsing targets `package.version` |
| Malformed version string | Semver validation rejects it |
| Version downgrade (typo) | Greater-than guard rejects it |
| Premature version bump | Accepted: relies on process discipline |
| Force-push replaying old commits | Tag exists guard exits early |
| No previous tags exist | Release notes include all commits |
