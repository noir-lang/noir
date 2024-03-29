FROM aztecprotocol/barretenberg-x86_64-linux-clang-assert
FROM aztecprotocol/noir-compile-acir-tests as noir-acir-tests

FROM node:18.19.0
RUN apt update && apt install git bash curl jq coreutils -y
COPY --from=0 /usr/src/barretenberg/cpp/build /usr/src/barretenberg/cpp/build
COPY --from=noir-acir-tests /usr/src/noir/noir-repo/test_programs /usr/src/noir/noir-repo/test_programs
WORKDIR /usr/src/barretenberg/acir_tests
COPY . .
# Run every acir test through native bb build prove_then_verify flow for UltraPlonk.
# This ensures we test independent pk construction through real/garbage witness data paths.
RUN FLOW=prove_then_verify ./run_acir_tests.sh
# Construct and verify a UltraHonk proof for all acir programs
RUN FLOW=prove_and_verify_ultra_honk ./run_acir_tests.sh
# Construct and verify a Goblin UltraHonk (GUH) proof for a single arbitrary program
RUN FLOW=prove_and_verify_goblin_ultra_honk ./run_acir_tests.sh 6_array
# Construct and verify a UltraHonk proof for all ACIR programs using the new witness stack workflow
RUN FLOW=prove_and_verify_ultra_honk_program ./run_acir_tests.sh
# This is a "full" Goblin flow. It constructs and verifies four proofs: GoblinUltraHonk, ECCVM, Translator, and merge
RUN FLOW=prove_and_verify_goblin ./run_acir_tests.sh 6_array
# Run 1_mul through native bb build, all_cmds flow, to test all cli args.
RUN VERBOSE=1 FLOW=all_cmds ./run_acir_tests.sh 1_mul
