FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/barretenberg-x86_64-linux-clang-assert
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/noir-compile-acir-tests as noir-acir-tests

FROM node:18.19.0-alpine
RUN apk update && apk add git bash curl jq coreutils
COPY --from=0 /usr/src/barretenberg/cpp/build /usr/src/barretenberg/cpp/build
COPY --from=noir-acir-tests /usr/src/noir/test_programs /usr/src/noir/test_programs
WORKDIR /usr/src/barretenberg/acir_tests
COPY . .
# Run every acir test through native bb build prove_then_verify flow for UltraPlonk.
# This ensures we test independent pk construction through real/garbage witness data paths.
RUN FLOW=prove_then_verify ./run_acir_tests.sh
# This flow is essentially the GoblinUltraHonk equivalent to the UltraPlonk "prove and verify". (This functionality is 
# accessed via the goblin "accumulate" mechanism).
RUN FLOW=accumulate_and_verify_goblin ./run_acir_tests.sh
# This is a "full" Goblin flow. It constructs and verifies four proofs: GoblinUltraHonk, ECCVM, Translator, and merge
RUN FLOW=prove_and_verify_goblin ./run_acir_tests.sh 6_array
# Run 1_mul through native bb build, all_cmds flow, to test all cli args.
RUN VERBOSE=1 FLOW=all_cmds ./run_acir_tests.sh 1_mul
