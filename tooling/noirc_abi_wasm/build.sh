#!/usr/bin/env bash

function require_command {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed." >&2
        exit 1
    fi
}
function check_installed {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "$1 is not installed. Please install it." >&2
    return 1
  fi
  return 0
}
function run_or_fail {
  "$@"
  local status=$?
  if [ $status -ne 0 ]; then
    echo "Command '$*' failed with exit code $status" >&2
    exit $status
  fi
}

require_command toml2json
require_command jq
require_command cargo
require_command wasm-bindgen
require_command wasm-opt

self_path=$(dirname "$(readlink -f "$0")")
export pname=$(toml2json < ${self_path}/Cargo.toml | jq -r .package.name)
export CARGO_TARGET_DIR=$self_path/target

rm -rf $self_path/outputs >/dev/null 2>&1
rm -rf $self_path/result >/dev/null 2>&1

if [ -v out ]; then
  echo "Will install package to $out (defined outside installPhase.sh script)"
else
  out="$self_path/outputs/out"
  echo "Will install package to $out"
fi

run_or_fail ${self_path}/buildPhaseCargoCommand.sh release
run_or_fail ${self_path}/installPhase.sh

ln -s $out $self_path/result
