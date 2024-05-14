#!/usr/bin/env bash
set -eu

cd $(dirname "$0")/..

command -v $1 >/dev/null 2>&1 && echo "true" || { echo >&2 "$1 is not installed" && echo "false"; }