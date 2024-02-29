#!/bin/bash
set -eu

cd $(dirname "$0")/../noir-repo

# Set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=${COMMIT_HASH:-$(git rev-parse --verify HEAD)}

cargo fmt --all --check
cargo clippy --workspace --locked --release
cargo test --workspace --locked --release
