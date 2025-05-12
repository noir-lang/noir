#!/bin/bash

NARGO=${NARGO:-nargo}

rm -rf ./test/noir_compiled_examples/**/target
$NARGO --program-dir ./test/noir_compiled_examples/assert_lt compile --force --pedantic-solving
$NARGO --program-dir ./test/noir_compiled_examples/assert_msg_runtime compile --force --pedantic-solving
$NARGO --program-dir ./test/noir_compiled_examples/fold_fibonacci compile --force --pedantic-solving
$NARGO --program-dir ./test/noir_compiled_examples/assert_raw_payload compile --force --pedantic-solving
$NARGO --program-dir ./test/noir_compiled_examples/databus compile --force --pedantic-solving
# Compile with no inlining to test runtime call stacks
$NARGO --program-dir ./test/noir_compiled_examples/assert_inside_brillig_nested compile --force --pedantic-solving --inliner-aggressiveness -9223372036854775808