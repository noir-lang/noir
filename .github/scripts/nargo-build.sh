#!/bin/bash
set -eu

export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=$(git rev-parse --verify HEAD)

cargo build --release
