#!/bin/zsh

cd build;
cmake ..;
cmake --build . -j
for file in  ./bin/plonk_tests  ./bin/rollup_proofs_account_tests  ./bin/rollup_proofs_claim_tests  ./bin/rollup_proofs_inner_proof_data_tests  ./bin/rollup_proofs_join_split_tests  ./bin/rollup_proofs_notes_tests  ./bin/srs_tests  ./bin/stdlib_aes128_tests  ./bin/stdlib_blake2s_tests  ./bin/stdlib_blake3s_tests  ./bin/stdlib_ecdsa_tests  ./bin/stdlib_merkle_tree_tests  ./bin/stdlib_primitives_tests  ./bin/stdlib_schnorr_tests  ./bin/stdlib_sha256_tests;
do ./$file; done;
./bin/stdlib_recursion_tests "--gtest_filter=*0*recursive_proof_composition"; # tests Turbo-Turbo and Ultra-Ultra
./bin/stdlib_pedersen_tests "--gtest_filter=*0*"; # only testing Ultra here
./bin/rollup_proofs_tx_rollup_tests "--gtest_filter=rollup_tests*1*1*"
./bin/rollup_proofs_tx_rollup_tests "--gtest_filter=rollup_tests*1*2*"
./bin/rollup_proofs_tx_rollup_tests "--gtest_filter=rollup_tests*2*2*"
./bin/rollup_proofs_root_rollup_tests  "--gtest_filter=*root_rollup*1_real_2*";
./bin/rollup_proofs_root_rollup_tests  "--gtest_filter=*root_rollup*2x3s*";