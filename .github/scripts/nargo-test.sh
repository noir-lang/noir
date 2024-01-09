#!/bin/bash
set -eu

apt-get install -y curl libc++-dev

export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=$(git rev-parse --verify HEAD)

cargo test --workspace --locked --release