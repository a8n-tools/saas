#!/usr/bin/env nu

# Get image tags from git describe.
# Returns a list of tags to publish:
# - Tagged commit (e.g. v0.1.0):          [v0.1.0, latest]
# - After a tag (e.g. v0.1.0-1-g1b66909): [v0.1.0, latest]
# - No tag at all:                         [latest]
export def get-tags []: nothing -> list<string> {
    use std log
    let describe = (^git describe --tags --always | str trim)
    log info $"[get-tags] git describe: ($describe)"

    # Try to parse as <tag>-<N>-g<hash> format (commits after a tag)
    let parts = ($describe | parse --regex '^(?<tag>.+)-\d+-g[0-9a-f]+$')

    if ($parts | is-not-empty) {
        let tag = $parts.tag.0
        log info $"[get-tags] Resolved tags: [($tag), latest]"
        return [$tag, "latest"]
    }

    # Exact tag match (no commits after tag)
    if ($describe | str starts-with "v") {
        log info $"[get-tags] Exact tag. Resolved tags: [($describe), latest]"
        return [$describe, "latest"]
    }

    # No tag — just latest
    log info $"[get-tags] No tag. Resolved tags: [latest]"
    return ["latest"]
}

# When run directly, output comma-separated tags
def main [] {
    get-tags | str join ","
}
