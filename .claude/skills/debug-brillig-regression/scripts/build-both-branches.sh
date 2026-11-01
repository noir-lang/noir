#!/usr/bin/env bash
# Build nargo for the current branch and a base branch, saving both binaries.
# Usage: build-both-branches.sh <base_branch> <work_dir>
#
# Outputs:
#   <work_dir>/nargo_base     - nargo built from the base branch
#   <work_dir>/nargo_current  - nargo built from the current branch

set -euo pipefail

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 <base_branch> <work_dir>" >&2
    exit 1
fi

base_branch="$1"
work_dir="$2"
current_branch=$(git branch --show-current)

mkdir -p "$work_dir"

echo "Building nargo for current branch ($current_branch)..."
cargo build --release -p nargo_cli 2>&1 | tail -3
cp target/release/nargo "$work_dir/nargo_current"

echo "Building nargo for base branch ($base_branch)..."
git checkout "$base_branch"
cargo build --release -p nargo_cli 2>&1 | tail -3
cp target/release/nargo "$work_dir/nargo_base"

git checkout "$current_branch"
echo "Saved binaries to $work_dir/nargo_base and $work_dir/nargo_current"
