#!/bin/bash
set -eu

cd /usr/src/noir
./scripts/nargo_compile_noir_codegen_assert_lt.sh
rm -rf /usr/src/noir/tooling/noir_codegen/test/assert_lt/target/debug_assert_lt.json
yarn workspace @noir-lang/noir_codegen test