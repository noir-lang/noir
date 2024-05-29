FROM aztecprotocol/bb.js
FROM aztecprotocol/noir-compile-acir-tests as noir-acir-tests

FROM node:18.19.0
COPY --from=0 /usr/src/barretenberg/ts-build /usr/src/barretenberg/ts
COPY --from=noir-acir-tests /usr/src/noir/noir-repo/test_programs /usr/src/noir/noir-repo/test_programs
RUN apt update && apt install -y lsof jq
WORKDIR /usr/src/barretenberg/acir_tests
# Build/install ts apps.
COPY browser-test-app browser-test-app
COPY headless-test headless-test
RUN cd browser-test-app && yarn && yarn build
RUN cd headless-test && yarn && npx playwright install && npx playwright install-deps
COPY . .
ENV VERBOSE=1
# TODO(https://github.com/noir-lang/noir/issues/5106)
# TODO(https://github.com/AztecProtocol/aztec-packages/issues/6672)
# Run ecdsa_secp256r1_3x through bb.js on node to check 256k support.
RUN BIN=../ts/dest/node/main.js FLOW=prove_then_verify ./run_acir_tests.sh ecdsa_secp256r1_3x
# Run a single arbitrary test not involving recursion through bb.js for UltraHonk
RUN BIN=../ts/dest/node/main.js FLOW=prove_then_verify_ultra_honk ./run_acir_tests.sh nested_array_dynamic
# Run a single arbitrary test not involving recursion through bb.js for Plonk
RUN BIN=../ts/dest/node/main.js FLOW=prove_and_verify ./run_acir_tests.sh poseidon_bn254_hash
# Run a single arbitrary test not involving recursion through bb.js for UltraHonk
RUN BIN=../ts/dest/node/main.js FLOW=prove_and_verify_ultra_honk ./run_acir_tests.sh closures_mut_ref
# Run a single arbitrary test for separate prove and verify for MegaHonk
RUN BIN=../ts/dest/node/main.js FLOW=prove_and_verify_mega_honk ./run_acir_tests.sh 6_array
# Fold and verify an ACIR program stack
RUN BIN=../ts/dest/node/main.js FLOW=fold_and_verify_program ./run_acir_tests.sh fold_basic
# Run 1_mul through bb.js build, all_cmds flow, to test all cli args.
RUN BIN=../ts/dest/node/main.js FLOW=all_cmds ./run_acir_tests.sh 1_mul
# TODO(https://github.com/AztecProtocol/aztec-packages/issues/6672)
# Run 6_array through bb.js on chrome testing multi-threaded browser support.
# TODO: Currently headless webkit doesn't seem to have shared memory so skipping multi-threaded test.
RUN BROWSER=chrome THREAD_MODEL=mt ./run_acir_tests_browser.sh 6_array
# Run 1_mul through bb.js on chrome/webkit testing single threaded browser support.
RUN BROWSER=chrome THREAD_MODEL=st ./run_acir_tests_browser.sh 1_mul
# Commenting for now as fails intermittently. Unreproducable on mainframe.
# See https://github.com/AztecProtocol/aztec-packages/issues/2104
#RUN BROWSER=webkit THREAD_MODEL=st ./run_acir_tests_browser.sh 1_mul
