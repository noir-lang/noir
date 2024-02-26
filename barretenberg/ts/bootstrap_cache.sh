#!/usr/bin/env bash
set -eu

[ -z "${NO_CACHE:-}" ] && type docker &> /dev/null && [ -f ~/.aws/credentials ] || exit 1

cd "$(dirname "$0")"
source ../../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving bb.js from remote cache...\033[0m"
extract_repo bb.js /usr/src/barretenberg/ts/dest .
# Annoyingly we still need to install modules, so they can be found as part of module resolution when portalled.
yarn install

remove_old_images bb.js
