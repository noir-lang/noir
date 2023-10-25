#!/bin/bash
# Env var overrides:
#   BIN: to specify a different binary to test with (e.g. bb.js or bb.js-dev).
set -eu

BIN=${BIN:-../cpp/build/bin/bb}
WRITE_VK_FLOW=${FLOW:-write_vk}
VK_FIELDS_FLOW=${FLOW:-vk_as_fields}
PROVE_FLOW=${FLOW:-prove}
PROOF_FIELDS_FLOW=${FLOW:-proof_as_fields}
CRS_PATH=~/.bb-crs
BRANCH=master
VERBOSE=${VERBOSE:-}
RECURSIVE=true
PROOF_NAME="proof_a"

WRITE_VK_FLOW_SCRIPT=$(realpath ./flows/${WRITE_VK_FLOW}.sh)
VK_FIELDS_FLOW_SCRIPT=$(realpath ./flows/${VK_FIELDS_FLOW}.sh)
PROVE_FLOW_SCRIPT=$(realpath ./flows/${PROVE_FLOW}.sh)
PROOF_FIELDS_FLOW_SCRIPT=$(realpath ./flows/${PROOF_FIELDS_FLOW}.sh)

if [ -f $BIN ]; then
    BIN=$(realpath $BIN)
else
    BIN=$(realpath $(which $BIN))
fi

export BIN CRS_PATH VERBOSE RECURSIVE PROOF_NAME

# Pull down the test vectors from the noir repo, if we don't have the folder already.
if [ ! -d acir_tests ]; then
  if [ -n "${TEST_SRC:-}" ]; then
    cp -R $TEST_SRC acir_tests
  else
    rm -rf noir
    git clone -b $BRANCH --filter=blob:none --no-checkout https://github.com/noir-lang/noir.git
    cd noir
    git sparse-checkout init --cone
    git sparse-checkout set tooling/nargo_cli/tests/acir_artifacts
    git checkout
    cd ..
    mv noir/tooling/nargo_cli/tests/acir_artifacts acir_tests
    rm -rf noir
  fi
fi

cd acir_tests

cd assert_statement

PROOF_DIR=$PWD/proofs
PROOF_PATH=$PROOF_DIR/$PROOF_NAME

echo -e "Write VK to file for assert_statement..\n"
set +e
$WRITE_VK_FLOW_SCRIPT
set -eu

echo -e "Write VK as fields for recursion...\n"
set +e
$VK_FIELDS_FLOW_SCRIPT
set -eu

echo -e "Generate proof to file...\n"
set +e
if [ ! -d "$PROOF_DIR" ]; then
  mkdir $PWD/proofs
fi
if [ ! -e "$PROOF_PATH" ]; then
  touch $PROOF_PATH
fi
$PROVE_FLOW_SCRIPT
set -eu

echo -e "Write proof as fields for recursion...\n"
set +e
$PROOF_FIELDS_FLOW_SCRIPT
set -eu
