#!/usr/bin/env bash

function require_command {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "Error: $1 is required but not installed." >&2
        exit 1
    fi
}

require_command toml2json
require_command jq
require_command cargo
require_command wasm-bindgen
require_command wasm-opt

export pname=$(toml2json < Cargo.toml | jq -r .package.name)

./preBuild.sh
cargo build --lib --release --package noir_wasm --target wasm32-unknown-unknown
./postBuild.sh
./installPhase.sh

