#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"
source ../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving noir packages from remote cache...\033[0m"
extract_repo noir-packages /usr/src/noir/packages ./
echo -e "\033[1mRetrieving nargo from remote cache...\033[0m"
extract_repo noir /usr/src/noir/target/release ./target/

