#!/usr/bin/env bash
# Validate that relative links in the design/ docs point to files that exist.
#
# The design docs link to source files with relative paths (e.g. `../../compiler/...`).
# When a source file is moved or renamed those links silently rot; this check fails
# loudly instead. External URLs (http/https/mailto) and pure `#anchor` links are ignored.
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
design_dir="$repo_root/design"

dead=0

while IFS= read -r md; do
  md_dir="$(dirname "$md")"

  # Link targets come from reference-style definitions (`[label]: target`) and
  # inline links (`[text](target)`).
  while IFS= read -r target; do
    [ -n "$target" ] || continue

    # Skip external URLs and pure anchors.
    case "$target" in
    *://* | mailto:* | '#'*) continue ;;
    esac

    # Drop any `#anchor` suffix and optional `"title"`.
    path="${target%%#*}"
    path="${path%% *}"
    [ -n "$path" ] || continue

    if [ ! -e "$md_dir/$path" ]; then
      printf 'Dead link in %s -> %s\n' "${md#"$repo_root"/}" "$target" >&2
      dead=1
    fi
  done < <(
    {
      sed -nE 's/^\[[^]]+\]:[[:space:]]+(.+)$/\1/p' "$md"
      grep -oE '\]\([^)]+\)' "$md" | sed -E 's/^\]\(//; s/\)$//'
    } || true
  )
done < <(find "$design_dir" -type f -name '*.md' | sort)

if [ "$dead" -ne 0 ]; then
  echo "Found dead links in design/ (see above)." >&2
  exit 1
fi

echo "All design/ links resolve."
