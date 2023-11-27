#!/bin/bash
set -eu

cd $(dirname "$0")/..

# If this project has been subrepod into another project, set build data manually.
if [ -f ".gitrepo" ]; then
  export SOURCE_DATE_EPOCH=$(date +%s)
  export GIT_DIRTY=false
  export GIT_COMMIT=$(awk '/commit =/ {print $3}' .gitrepo)
fi

# Build native.
cargo build --features="noirc_frontend/aztec" --release