#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"
source ../build-system/scripts/setup_env '' '' mainframe_$USER > /dev/null

echo -e "\033[1mRetrieving avm-transpiler from remote cache...\033[0m"
extract_repo avm-transpiler \
  /usr/src/avm-transpiler/target/release/avm-transpiler ./target/release/

remove_old_images avm-transpiler
