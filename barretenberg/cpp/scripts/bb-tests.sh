#!/usr/bin/env bash
# This script runs all test suites that have not been broken out into their own jobs for parallelisation.
# Might be better to list exclusions here rather than inclusions as risky to maintain.
set -eu

$(aws ecr get-login --region us-east-2 --no-include-email) 2> /dev/null
export PATH="$PATH:$(git rev-parse --show-toplevel)/build-system/scripts"
REPOSITORY=barretenberg-x86_64-linux-clang-assert
# use the image rebuild patterns to compute a content hash, use this to get a URI
IMAGE_URI=$(calculate_image_uri $REPOSITORY)
retry docker pull $IMAGE_URI

TESTS=(
  commitment_schemes_tests
  crypto_aes128_tests
  crypto_blake2s_tests
  crypto_blake3s_tests
  crypto_ecdsa_tests
  crypto_pedersen_commitment_tests
  crypto_pedersen_hash_tests
  crypto_poseidon2_tests
  crypto_schnorr_tests
  crypto_sha256_tests
  dsl_tests
  ecc_tests
  eccvm_tests
  flavor_tests
  goblin_tests
  join_split_example_proofs_inner_proof_data_tests
  join_split_example_proofs_notes_tests
  numeric_tests
  plonk_tests
  polynomials_tests
  protogalaxy_tests
  relations_tests
  srs_tests
  sumcheck_tests
  transcript_tests
  translator_vm_tests
  ultra_honk_tests
  vm_tests
)
TESTS_STR="${TESTS[@]}"

docker run --rm -t $IMAGE_URI /bin/sh -c "\
  set -xe; \
  cd /usr/src/barretenberg/cpp; \
  srs_db/download_ignition.sh 1; \
  srs_db/download_grumpkin.sh; \
  cd build; \
  for BIN in $TESTS_STR; do ./bin/\$BIN; done"
