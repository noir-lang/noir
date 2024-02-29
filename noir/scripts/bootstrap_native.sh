#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/../noir-repo

# Set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
export GIT_COMMIT=${COMMIT_HASH:-$(git rev-parse --verify HEAD)}

# Check if the 'cargo' command is available in the system
if ! command -v cargo > /dev/null; then
    echo "Cargo is not installed. Please install Cargo and the Rust toolchain."
    exit 1
fi

# Build native.
if [ -n "${DEBUG:-}" ]; then
  cargo build
else
  cargo build --release
fi