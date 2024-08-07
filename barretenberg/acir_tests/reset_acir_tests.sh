# Run from within barretenberg/acir_tests

# clean and rebuild noir then compile the test programs
cd ../../noir/noir-repo
cargo clean
noirup -p .
cd test_programs && ./rebuild.sh

# remove and repopulate the test artifacts in bberg
cd ../../../barretenberg/acir_tests
rm -rf acir_tests
./clone_test_vectors.sh