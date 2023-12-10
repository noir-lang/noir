#!/usr/bin/env bash
set -eu

cd "$(dirname "$0")"

(cd cpp && ./bootstrap.sh $@)
(cd ts && ./bootstrap.sh $@)
