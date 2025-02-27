#!/bin/bash

NARGO=${NARGO:-nargo}

rm -rf ./test/noir_compiled_examples/**/target
$NARGO --program-dir ./test/noir_compiled_examples/assert_lt compile --force
$NARGO --program-dir ./test/noir_compiled_examples/assert_msg_runtime compile --force
$NARGO --program-dir ./test/noir_compiled_examples/fold_fibonacci compile --force
$NARGO --program-dir ./test/noir_compiled_examples/assert_raw_payload compile --force
$NARGO --program-dir ./test/noir_compiled_examples/databus compile --force
