#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

PROJECTS=(
  noir-contracts
  noir-protocol-circuits
)

for PROJECT in "${PROJECTS[@]}"; do
  (cd "./$PROJECT" && ./bootstrap.sh "$@")
done
