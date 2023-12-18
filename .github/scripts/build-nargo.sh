#!/bin/bash
set -eu

cd /usr/src/noir
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=$(git rev-parse --verify HEAD)

cargo build --features="noirc_driver/aztec" --release
