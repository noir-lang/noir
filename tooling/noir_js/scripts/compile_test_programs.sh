#!/bin/bash

NARGO=${NARGO:-nargo}

rm -rf ./test/noir_compiled_examples/**/target
$NARGO --program-dir ./test/noir_compiled_examples/assert_lt compile --force
$NARGO --program-dir ./test/noir_compiled_examples/assert_msg_runtime compile --force
$NARGO --program-dir ./test/noir_compiled_examples/fold_fibonacci compile --force
$NARGO --program-dir ./test/noir_compiled_examples/assert_raw_payload compile --force
$NARGO --program-dir ./test/noir_compiled_examples/databus compile --force
# Compile with no inlining to test runtime call stacks
$NARGO --program-dir ./test/noir_compiled_examples/assert_inside_brillig_nested compile --force --inliner-aggressiveness -9223372036854775808