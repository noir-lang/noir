#!/usr/bin/env bash

function require_command {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed." >&2
        exit 1
    fi
}
function test_directory_exists() {
  if [ ! -d "$1" ]; then
    echo "Error: Directory '$1' does not exist. Run this script from Workspace Root."
    exit 1
  fi
}

require_command toml2json
require_command jq
require_command cargo
require_command wasm-bindgen
require_command wasm-opt
test_directory_exists crates/noirc_abi_wasm

export pname=$(toml2json < crates/noirc_abi_wasm/Cargo.toml | jq -r .package.name)

echo Building package \"$pname\"

crates/noirc_abi_wasm/preBuild.sh
cargo build --lib --release --package $pname --target wasm32-unknown-unknown
crates/noirc_abi_wasm/postBuild.sh
crates/noirc_abi_wasm/installPhase.sh

