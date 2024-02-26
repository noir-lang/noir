#!/usr/bin/env bash
set -eu

[ -z "${NO_CACHE:-}" ] && type docker &> /dev/null && [ -f ~/.aws/credentials ] || exit 1

cd "$(dirname "$0")"
source ../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving contracts from remote cache...\033[0m"
extract_repo l1-contracts /usr/src/l1-contracts/out .

remove_old_images l1-contracts
