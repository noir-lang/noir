#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

# If this project has been subrepod into another project, set build data manually.
export SOURCE_DATE_EPOCH=$(date +%s)
export GIT_DIRTY=false
if [ -f ".gitrepo" ]; then
  export GIT_COMMIT=$(awk '/commit =/ {print $3}' .gitrepo)
else
  export GIT_COMMIT=$(git rev-parse --verify HEAD)
fi

# Build native.
if [ -n "${DEBUG:-}" ]; then
  cargo build
else
  cargo build --release
fi
