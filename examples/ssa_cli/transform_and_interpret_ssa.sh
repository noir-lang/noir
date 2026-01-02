#!/bin/bash
set -eu

cd $(dirname $0)

# Sanity check that we can parse an SSA, pipe it through some transformations
# and interpret it with values from a TOML file.
cat ./test.ssa \
  | noir-ssa transform --ssa-pass "step 1" \
  | noir-ssa transform --ssa-pass "Dead Instruction Elimination" \
  | noir-ssa interpret --input-path ./test.toml