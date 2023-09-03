#!/bin/bash
# This script runs all test suites that have not been broken out into their own jobs for parallelisation.
# Might be better to list exclusions here rather than inclusions as risky to maintain.
set -eu

$(aws ecr get-login --region us-east-2 --no-include-email) 2> /dev/null
REPOSITORY=barretenberg-x86_64-linux-clang-assert
# use the image rebuild patterns to compute a content hash, use this to get a URI
IMAGE_URI="278380418400.dkr.ecr.us-east-2.amazonaws.com/$REPOSITORY:cache-$CONTENT_HASH"
docker pull $IMAGE_URI

TESTS=(
  crypto_aes128_tests
  crypto_blake2s_tests
  crypto_blake3s_tests
  crypto_ecdsa_tests
  crypto_pedersen_commitment_tests
  crypto_schnorr_tests
  crypto_sha256_tests
  ecc_tests
  numeric_tests
  plonk_tests
  polynomials_tests
  join_split_example_proofs_inner_proof_data_tests
  join_split_example_proofs_notes_tests
  srs_tests
  transcript_tests
  dsl_tests
)
TESTS_STR="${TESTS[@]}"

docker run --rm -t $IMAGE_URI /bin/sh -c "\
  set -xe; \
  cd /usr/src/barretenberg/cpp; \
  (cd srs_db && ./download_ignition.sh 1); \
  cd build; \
  ./bin/grumpkin_srs_gen 1048576; \
  for BIN in $TESTS_STR; do ./bin/\$BIN; done"
