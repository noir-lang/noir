cd ~/aztec-packages/noir/noir-repo
cargo clean
noirup -p .
cd test_programs && ./rebuild.sh

cd ~/aztec-packages/barretenberg/acir_tests
rm -rf acir_tests
