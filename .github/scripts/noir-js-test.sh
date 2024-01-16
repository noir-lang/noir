#!/bin/bash
set -eu

./scripts/nargo_compile_noir_js_assert_lt.sh
rm -rf /usr/src/noir/tooling/noir_js/test/noir_compiled_examples/assert_lt/target/debug_assert_lt.json
yarn workspace @noir-lang/noir_js test