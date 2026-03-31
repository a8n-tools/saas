# Automated Release Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a Forgejo Actions workflow that automatically tags and releases when the version in `api/Cargo.toml` bumps on `main`.

**Architecture:** Single workflow file triggered by `api/Cargo.toml` changes on `main`. Nushell scripts extract and validate the version, build release notes from `git log`, and call the Forgejo release API. Three guards (semver format, tag uniqueness, version increase) prevent accidental releases.

**Tech Stack:** Forgejo Actions, Nushell, Forgejo REST API

---

### Task 1: Create the release workflow with trigger and checkout

**Files:**
- Create: `.forgejo/workflows/release.yml`

- [ ] **Step 1: Create the workflow file with trigger, job definition, and checkout step**

```yaml
---

# Automatically create a git tag and Forgejo release when the version
# in api/Cargo.toml changes on main.
name: Create release

on:
  push:
    branches:
      - main
    paths:
      - api/Cargo.toml

jobs:
  release:
    name: Tag and release
    runs-on: ${{ vars.RUNS_ON_OPENSUSE_BASE_LATEST }}
    permissions:
      contents: write

    steps:
      - name: Checkout full history
        uses: https://code.forgejo.org/actions/checkout@v3
        with:
          fetch-depth: 0
```

- [ ] **Step 2: Commit**

```bash
git add .forgejo/workflows/release.yml
git commit -m "ci: add release workflow skeleton with trigger and checkout"
```

---

### Task 2: Add version extraction and semver validation

**Files:**
- Modify: `.forgejo/workflows/release.yml`

- [ ] **Step 1: Add the version extraction step**

Append after the checkout step:

```yaml
      - name: Extract version from Cargo.toml
        id: version
        shell: nu {0}
        run: |
          let version = (open api/Cargo.toml | get package.version)
          print $"Extracted version: ($version)"
          $"version=($version)\n" | save --append $env.GITHUB_OUTPUT
```

- [ ] **Step 2: Add the semver validation step**

Append after the version extraction step:

```yaml
      - name: Validate semver format
        shell: nu {0}
        run: |
          let version = "${{ steps.version.outputs.version }}"
          if ($version | parse --regex '^(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)$' | is-empty) {
            print $"::error::Invalid semver format: ($version)"
            exit 1
          }
          print $"Valid semver: ($version)"
```

- [ ] **Step 3: Commit**

```bash
git add .forgejo/workflows/release.yml
git commit -m "ci: add version extraction and semver validation to release workflow"
```

---

### Task 3: Add tag-exists and version-greater-than guards

**Files:**
- Modify: `.forgejo/workflows/release.yml`

- [ ] **Step 1: Add the tag-exists guard step**

Append after the semver validation step:

```yaml
      - name: Check if tag already exists
        id: tag_check
        shell: nu {0}
        run: |
          let tag = $"v${{ steps.version.outputs.version }}"
          let exists = (^git tag --list | lines | any {|t| $t == $tag })
          if $exists {
            print $"::notice::Tag ($tag) already exists, skipping release"
          }
          $"exists=($exists)\n" | save --append $env.GITHUB_OUTPUT
```

- [ ] **Step 2: Add the version-greater-than guard step**

Append after the tag-exists check. This step only runs when the tag does not exist:

```yaml
      - name: Verify version is greater than latest tag
        id: version_check
        if: steps.tag_check.outputs.exists == 'false'
        shell: nu {0}
        run: |
          let new_version = "${{ steps.version.outputs.version }}"
          let tags = (^git tag --list | lines | where {|t| $t starts-with "v" })
          if ($tags | is-empty) {
            print "No existing tags, first release"
            $"prev_tag=\n" | save --append $env.GITHUB_OUTPUT
          } else {
            let latest = ($tags | sort-by --natural --reverse | first)
            let latest_version = ($latest | str substring 1..)
            print $"Latest tag: ($latest), new version: ($new_version)"
            let new_parts = ($new_version | split row "." | each {|p| $p | into int })
            let old_parts = ($latest_version | split row "." | each {|p| $p | into int })
            let is_greater = (
              ($new_parts.0 > $old_parts.0) or
              ($new_parts.0 == $old_parts.0 and $new_parts.1 > $old_parts.1) or
              ($new_parts.0 == $old_parts.0 and $new_parts.1 == $old_parts.1 and $new_parts.2 > $old_parts.2)
            )
            if not $is_greater {
              print $"::error::Version ($new_version) is not greater than ($latest_version)"
              exit 1
            }
            $"prev_tag=($latest)\n" | save --append $env.GITHUB_OUTPUT
          }
```

- [ ] **Step 3: Commit**

```bash
git add .forgejo/workflows/release.yml
git commit -m "ci: add tag-exists and version-greater-than guards to release workflow"
```

---

### Task 4: Add release notes generation and API call

**Files:**
- Modify: `.forgejo/workflows/release.yml`

- [ ] **Step 1: Add the release notes generation step**

Append after the version check step:

```yaml
      - name: Build release notes
        id: notes
        if: steps.tag_check.outputs.exists == 'false'
        shell: nu {0}
        run: |
          let prev_tag = "${{ steps.version_check.outputs.prev_tag }}"
          let range = if ($prev_tag | is-empty) { "HEAD" } else { $"($prev_tag)..HEAD" }
          let notes = (^git log $range --no-merges --pretty=format:"- %s (%h)")
          print "Release notes:"
          print $notes
          $notes | save --force /tmp/release-notes.txt
```

- [ ] **Step 2: Add the API call step to create the tag and release**

Append after the release notes step:

```yaml
      - name: Create tag and release
        if: steps.tag_check.outputs.exists == 'false'
        shell: nu {0}
        env:
          FORGEJO_URL: ${{ forgejo.server_url }}
          REPO: ${{ forgejo.repository }}
          TOKEN: ${{ secrets.FORGEJO_TOKEN }}
        run: |
          let tag = $"v${{ steps.version.outputs.version }}"
          let notes = (open /tmp/release-notes.txt)
          let payload = {
            tag_name: $tag,
            name: $tag,
            body: $notes,
            draft: false,
            prerelease: false,
          }
          let url = $"($env.FORGEJO_URL)/api/v1/repos/($env.REPO)/releases"
          print $"Creating release ($tag) at ($url)"
          let response = (
            http post
              --content-type application/json
              --headers [Authorization $"token ($env.TOKEN)"]
              $url
              $payload
          )
          print $"Release created: ($response.html_url)"
```

- [ ] **Step 3: Commit**

```bash
git add .forgejo/workflows/release.yml
git commit -m "ci: add release notes generation and Forgejo API call to release workflow"
```

---

### Task 5: Review the complete workflow file

**Files:**
- Read: `.forgejo/workflows/release.yml`

- [ ] **Step 1: Read the complete workflow and verify all steps are present and correctly ordered**

The final file should contain these steps in order:
1. Checkout full history
2. Extract version from Cargo.toml
3. Validate semver format
4. Check if tag already exists
5. Verify version is greater than latest tag (conditional: tag does not exist)
6. Build release notes (conditional: tag does not exist)
7. Create tag and release (conditional: tag does not exist)

Verify that:
- All `if` conditions reference `steps.tag_check.outputs.exists == 'false'`
- Step IDs (`version`, `tag_check`, `version_check`, `notes`) are referenced correctly
- `GITHUB_OUTPUT` is used for step outputs (Forgejo uses the same mechanism as GitHub Actions)

- [ ] **Step 2: Commit any fixes if needed**

```bash
git add .forgejo/workflows/release.yml
git commit -m "ci: fix release workflow issues found during review"
```
