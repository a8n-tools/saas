# Releasing

Releases are automated via Forgejo Actions. When a version bump in `api/Cargo.toml` lands on `main`, the workflow creates a git tag and Forgejo release with auto-generated release notes.

---

## How to Create a Release

1. Bump the `version` field in `api/Cargo.toml` (e.g. `0.5.0` → `0.6.0`)
2. Commit, push, and merge to `main`
3. The release workflow runs automatically and creates:
   - A git tag (`v0.6.0`)
   - A Forgejo release with notes from all non-merge commits since the last tag

That's it. No manual tag creation or release UI interaction needed.

---

## Version Rules

The workflow enforces these checks before creating a release:

| Check | What happens if it fails |
|-------|--------------------------|
| **Semver format** | Version must be `X.Y.Z` (digits only). Malformed versions are rejected. |
| **Tag uniqueness** | If the tag already exists, the workflow exits early (no error). |
| **Version must increase** | New version must be strictly greater than the latest tag. Downgrades are rejected. |

---

## Release Notes

Generated automatically from `git log` between the previous tag and `HEAD`:

- Only non-merge commits are included
- Format: `- <commit message> (<short hash>)`
- Conventional commit messages (`feat:`, `fix:`, etc.) produce the clearest notes

---

## What Triggers Image Builds?

Image builds are **not** triggered by the release workflow. They are triggered independently by the same `main` push that changes `api/` or `frontend/` files. See `build-api.yml` and `build-frontend.yml`.

---

## Workflow File

`.forgejo/workflows/release.yml`
