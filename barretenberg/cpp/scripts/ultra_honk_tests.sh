set -eu

# Move above script dir.
cd $(dirname $0)/..

cmake --preset clang16-dbg-fast
cmake --build --preset clang16-dbg-fast

cd build/

./bin/flavor_tests
./bin/relations_tests
./bin/transcript_tests
./bin/commitment_schemes_tests
./bin/sumcheck_tests
./bin/eccvm_tests
./bin/translator_vm_tests
./bin/protogalaxy_tests
./bin/ultra_honk_tests
./bin/stdlib_honk_recursion_tests --gtest_filter=Goblin*
./bin/stdlib_honk_recursion_tests --gtest_filter=Honk*
./bin/stdlib_honk_recursion_tests --gtest_filter=Proto*
./bin/stdlib_honk_recursion_tests --gtest_filter=RecursiveMerge*
./bin/goblin_tests
./bin/client_ivc_tests