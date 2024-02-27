#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"
source ../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving noir projects from remote cache...\033[0m"
extract_repo noir-projects \
  /usr/src/noir-projects/noir-contracts/target ./noir-contracts \
  /usr/src/noir-projects/noir-protocol-circuits/src/target ./noir-protocol-circuits/src

remove_old_images noir-projects
