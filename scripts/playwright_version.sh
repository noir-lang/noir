#!/usr/bin/env bash
# Prints the Playwright version that yarn.lock resolves.
#
# yarn.lock is the single source of truth for Playwright's version: the browser
# binaries, the driver in `node_modules`, and the `mcr.microsoft.com/playwright`
# container image must all agree, and a mismatch surfaces as a red-herring
# browser-launch error rather than a version error. Every other site that needs
# the version calls this script instead of hard-coding it, so a dependabot bump
# of the lockfile carries them all.
set -euo pipefail

lockfile="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/yarn.lock"

# The entry's descriptor list is unordered ("playwright@npm:1.58.2,
# playwright@npm:^1.53.0"), so read the resolved `version:` field of the entry
# rather than a version out of the header. `playwright-core@npm:` does not match
# the `@npm:` anchor and so cannot be picked up by mistake.
version=$(awk '
    /^"?playwright@npm:/ { in_entry = 1; next }
    in_entry && $1 == "version:" { print $2; exit }
    /^[^[:space:]]/ { in_entry = 0 }
' "$lockfile")

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "playwright_version.sh: could not resolve playwright's version from $lockfile" >&2
    exit 1
fi

echo "$version"
