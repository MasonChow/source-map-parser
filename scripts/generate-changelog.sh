#!/usr/bin/env bash
set -euo pipefail

# Usage: scripts/generate-changelog.sh <new_version> [repo_url]
# Reads git history since last tag, parses Conventional Commits, groups by type & scope,
# detects BREAKING CHANGES, generates compare links for GitHub/GitLab.

NEW_VERSION="$1"
REPO_URL="${2:-}"  # e.g. https://github.com/OWNER/REPO or https://gitlab.com/group/project
DATE=$(date +%Y-%m-%d)
CHANGELOG_FILE=CHANGELOG.md

LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [ -z "$LAST_TAG" ]; then
  RANGE=""
else
  RANGE="${LAST_TAG}..HEAD"
fi

echo "Generating changelog for version $NEW_VERSION (since $LAST_TAG)" >&2

# Collect commits
COMMITS=$(git log --pretty=format:'%s||%h||%b' $RANGE)

declare -A sections
sections[feat]=""
sections[fix]=""
sections[perf]=""
sections[refactor]=""
sections[docs]=""
sections[test]=""
sections[chore]=""
sections[build]=""
sections[ci]=""
sections[style]=""
sections[other]=""
breaking_section=""

while IFS= read -r line; do
  [ -z "$line" ] && continue
  subject_part=${line%%||*}
  rest=${line#*||}
  hash=${rest%%||*}
  body=${rest#*||}

  # Conventional Commit regex: type(scope)!: description
  if [[ $subject_part =~ ^([a-zA-Z]+)(\(([a-zA-Z0-9_-]+)\))?(!)?:[[:space:]](.+) ]]; then
    type=${BASH_REMATCH[1],,}
    scope=${BASH_REMATCH[3]:-}
    bang=${BASH_REMATCH[4]:-}
    desc=${BASH_REMATCH[5]}
  else
    type="other"
    scope=""
    desc="$subject_part"
    bang=""
  fi

  short_ref=$hash
  if [ -n "$REPO_URL" ]; then
    # Normalize repo URL (remove .git suffix if present)
    repo_clean=${REPO_URL%.git}
    if [[ $repo_clean == *github.com* || $repo_clean == *gitlab.com* ]]; then
      short_ref="[$hash]($repo_clean/commit/$hash)"
    fi
  fi

  scope_prefix=""
  [ -n "$scope" ] && scope_prefix="**$scope:** "
  formatted="- ${scope_prefix}${desc} (${short_ref})"

  # Detect breaking in subject or body
  if [ -n "$bang" ] || grep -qi '^BREAKING CHANGE' <<< "$body"; then
    breaking_section+="$formatted\n"
  fi

  # Append to section
  if [[ -z ${sections[$type]+_} ]]; then
    sections[other]+="$formatted\n"
  else
    sections[$type]+="$formatted\n"
  fi
done <<< "$COMMITS"

build_section() {
  local title="$1"; shift
  local body="$1"; shift || true
  if [ -n "$body" ]; then
    echo "### $title"
    echo -e "$body"
    echo
  fi
}

TMP_FILE=$(mktemp)
{
  echo "## v$NEW_VERSION - $DATE"
  if [ -n "$REPO_URL" ] && [ -n "$LAST_TAG" ]; then
    repo_clean=${REPO_URL%.git}
    if [[ $repo_clean == *github.com* || $repo_clean == *gitlab.com* ]]; then
      echo "[Compare changes]($repo_clean/compare/$LAST_TAG...v$NEW_VERSION)"
      echo
    fi
  fi
  build_section "Breaking Changes" "$breaking_section"
  build_section "Features" "${sections[feat]}"
  build_section "Fixes" "${sections[fix]}"
  build_section "Performance" "${sections[perf]}"
  build_section "Refactors" "${sections[refactor]}"
  build_section "Docs" "${sections[docs]}"
  build_section "Tests" "${sections[test]}"
  build_section "Build" "${sections[build]}"
  build_section "CI" "${sections[ci]}"
  build_section "Style" "${sections[style]}"
  build_section "Chore" "${sections[chore]}"
  build_section "Other" "${sections[other]}"
} > "$TMP_FILE"

if [ -f "$CHANGELOG_FILE" ]; then
  cat "$CHANGELOG_FILE" >> "$TMP_FILE"
fi
mv "$TMP_FILE" "$CHANGELOG_FILE"

echo "Changelog updated: $CHANGELOG_FILE" >&2
