#!/bin/bash
set -eu

.github/scripts/noirc-abi-build.sh

yarn workspace @noir-lang/noir_wasm build
