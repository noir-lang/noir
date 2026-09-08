#!/usr/bin/env bash
# Prints the Playwright version that yarn.lock resolves.
#
# yarn.lock is the single source of truth for Playwright's version: the browser
# binaries, the driver in `node_modules`, and the `mcr.microsoft.com/playwright`
# container image must all agree, and a mismatch surfaces as a red-herring
# browser-launch error rather than a version error. Every other site that needs
# the version calls this script instead of hard-coding it, so a dependabot bump
# of the lockfile carries them all.
#
# Two direct dependants pull Playwright in: `@playwright/test`, which pins an
# exact `playwright`, and `@web/test-runner-playwright`, which takes a range.
# When the exact pin moves, yarn leaves the range on its existing resolution
# unless told otherwise, and the lockfile ends up holding two Playwright
# versions with no single right answer for the browsers to install. This script
# refuses to guess in that case — see the error path below.
set -euo pipefail

lockfile="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/yarn.lock"

# Read the resolved `version:` field of every `playwright@npm:` entry. The
# descriptor list in an entry header is unordered ("playwright@npm:1.62.1,
# playwright@npm:^1.53.0"), so the header is not a reliable place to read a
# version from. `playwright-core@npm:` does not match the `@npm:` anchor and so
# cannot be picked up by mistake.
versions=$(awk '
    /^"?playwright@npm:/ { in_entry = 1; next }
    in_entry && $1 == "version:" { print $2; in_entry = 0; next }
    /^[^[:space:]]/ { in_entry = 0 }
' "$lockfile" | sort -u)

count=$(printf '%s' "$versions" | grep -c . || true)

if [[ "$count" -gt 1 ]]; then
    echo "playwright_version.sh: $lockfile resolves $count different Playwright versions:" >&2
    printf '  %s\n' $versions >&2
    echo "Browsers, driver and container image can only agree on one. Run" >&2
    echo "\`yarn dedupe playwright playwright-core\` and commit the lockfile." >&2
    exit 1
fi

if [[ ! "$versions" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "playwright_version.sh: could not resolve playwright's version from $lockfile" >&2
    exit 1
fi

echo "$versions"
