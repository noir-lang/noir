FROM aztecprotocol/barretenberg-x86_64-linux-clang-assert
FROM aztecprotocol/barretenberg-x86_64-linux-clang-sol
FROM aztecprotocol/noir-compile-acir-tests as noir-acir-tests

FROM node:18.19.0-alpine
RUN apk update && apk add git bash curl jq
COPY --from=0 /usr/src/barretenberg/cpp/build /usr/src/barretenberg/cpp/build
COPY --from=1 /usr/src/barretenberg/sol/src/ultra/BaseUltraVerifier.sol /usr/src/barretenberg/sol/src/ultra/BaseUltraVerifier.sol
COPY --from=noir-acir-tests /usr/src/noir/test_programs /usr/src/noir/test_programs
COPY --from=ghcr.io/foundry-rs/foundry:latest /usr/local/bin/anvil /usr/local/bin/anvil
WORKDIR /usr/src/barretenberg/acir_tests
COPY . .
# Run every acir test through a solidity verifier.
RUN (cd sol-test && yarn)
RUN PARALLEL=1 FLOW=sol ./run_acir_tests.sh
