FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/bb.js
FROM 278380418400.dkr.ecr.eu-west-2.amazonaws.com/noir-compile-acir-tests as noir-acir-tests

FROM node:18.19.0
COPY --from=0 /usr/src/barretenberg/ts-build /usr/src/barretenberg/ts
COPY --from=noir-acir-tests /usr/src/noir/test_programs /usr/src/noir/test_programs
RUN apt update && apt install -y lsof jq
WORKDIR /usr/src/barretenberg/acir_tests
# Build/install ts apps.
COPY browser-test-app browser-test-app
COPY headless-test headless-test
RUN (cd browser-test-app && yarn && yarn build) && (cd headless-test && yarn && npx playwright install && npx playwright install-deps)
COPY . .
ENV VERBOSE=1
# Run double_verify_proof through bb.js on node to check 512k support.
RUN BIN=../ts/dest/node/main.js FLOW=prove_then_verify ./run_acir_tests.sh double_verify_proof
# Run a single arbitrary test not involving recursion through bb.js for GoblinUltraHonk
RUN BIN=../ts/dest/node/main.js FLOW=accumulate_and_verify_goblin ./run_acir_tests.sh 6_array
# Run a single arbitrary test not involving recursion through bb.js for full Goblin
RUN BIN=../ts/dest/node/main.js FLOW=prove_and_verify_goblin ./run_acir_tests.sh 6_array
# Run 1_mul through bb.js build, all_cmds flow, to test all cli args.
RUN BIN=../ts/dest/node/main.js FLOW=all_cmds ./run_acir_tests.sh 1_mul
# Run double_verify_proof through bb.js on chrome testing multi-threaded browser support.
# TODO: Currently headless webkit doesn't seem to have shared memory so skipping multi-threaded test.
RUN BROWSER=chrome THREAD_MODEL=mt ./run_acir_tests_browser.sh double_verify_proof
# Run 1_mul through bb.js on chrome/webkit testing single threaded browser support.
RUN BROWSER=chrome THREAD_MODEL=st ./run_acir_tests_browser.sh 1_mul
# Commenting for now as fails intermittently. Unreproducable on mainframe.
# See https://github.com/AztecProtocol/aztec-packages/issues/2104
#RUN BROWSER=webkit THREAD_MODEL=st ./run_acir_tests_browser.sh 1_mul
