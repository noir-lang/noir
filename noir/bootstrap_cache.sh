#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"
source ../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving noir packages from remote cache...\033[0m"
extract_repo_if_working_copy_clean noir-packages /usr/src/noir/packages ./
echo -e "\033[1mRetrieving nargo from remote cache...\033[0m"
extract_repo_if_working_copy_clean noir /usr/src/noir/noir-repo/target/release ./noir-repo/target/

remove_old_images noir-packages
remove_old_images noir
