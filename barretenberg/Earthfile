VERSION 0.8

acir-tests:
    FROM ../build-images+build
    WORKDIR /usr/src/barretenberg
    COPY ./acir_tests .
    SAVE ARTIFACT ./*

sol:
  FROM ../build-images+build
  WORKDIR /usr/src/barretenberg
  COPY ./sol .
  SAVE ARTIFACT ./*


barretenberg-acir-tests-bb:
    FROM ../build-images/+build

    COPY ./cpp/+preset-clang-assert/bin/bb /usr/src/barretenberg/cpp/build/bin/bb
    COPY +acir-tests/ /usr/src/barretenberg/acir_tests
    COPY ../noir/+build-acir-tests/ /usr/src/acir_artifacts

    WORKDIR /usr/src/barretenberg/acir_tests
    RUN rm -rf ./acir_tests

    ENV TEST_SRC /usr/src/acir_artifacts
    ENV VERBOSE=1
    # Run every acir test through native bb build prove_then_verify flow for UltraPlonk.
    # This ensures we test independent pk construction through real/garbage witness data paths.
    RUN FLOW=prove_then_verify ./run_acir_tests.sh
    # Construct and separately verify a UltraHonk proof for a single program
    RUN FLOW=prove_then_verify_ultra_honk ./run_acir_tests.sh sha256
    # Construct and separately verify a MegaHonk proof for all acir programs
    RUN FLOW=prove_then_verify_mega_honk ./run_acir_tests.sh
    # Construct and verify a UltraHonk proof for a single program
    RUN FLOW=prove_and_verify_ultra_honk ./run_acir_tests.sh pedersen_hash
    # Construct and verify a MegaHonk proof for a single arbitrary program
    RUN FLOW=prove_and_verify_mega_honk ./run_acir_tests.sh 6_array
    # Construct and verify a MegaHonk proof on one non-recursive program using the new witness stack workflow
    RUN FLOW=prove_and_verify_ultra_honk_program ./run_acir_tests.sh merkle_insert
    # Construct and verify a MegaHonk proof for all ACIR programs using the new witness stack workflow
    RUN FLOW=prove_and_verify_mega_honk_program ./run_acir_tests.sh
    # Fold and verify an ACIR program stack using ClientIvc
    RUN FLOW=fold_and_verify_program ./run_acir_tests.sh fold_basic
    # Fold and verify an ACIR program stack using ClientIvc, then natively verify the ClientIVC proof.
    RUN FLOW=prove_then_verify_client_ivc ./run_acir_tests.sh fold_basic
    # Fold and verify an ACIR program stack using ClientIvc, recursively verify as part of the Tube circuit and produce and verify a Honk proof
    RUN FLOW=prove_then_verify_tube ./run_acir_tests.sh fold_basic
    # Construct and separately verify a UltraHonk proof for a single program that recursively verifies a Honk proof
    RUN FLOW=prove_then_verify_ultra_honk ./run_acir_tests.sh verify_honk_proof
    # Construct and verify a UltraHonk proof for a single program that recursively verifies a Honk proof
    RUN FLOW=prove_and_verify_ultra_honk ./run_acir_tests.sh verify_honk_proof
    # Run 1_mul through native bb build, all_cmds flow, to test all cli args.
    RUN FLOW=all_cmds ./run_acir_tests.sh 1_mul

barretenberg-acir-tests-sol:
    FROM ../build-images/+build

    COPY ./cpp/+preset-sol/ /usr/src/barretenberg/cpp/build
    COPY ./cpp/+preset-clang-assert/bin/bb /usr/src/barretenberg/cpp/build/bin/bb
    COPY ./+acir-tests/ /usr/src/barretenberg/acir_tests
    COPY ./+sol/ /usr/src/barretenberg/sol
    COPY ../noir/+build-acir-tests/ /usr/src/acir_artifacts

    WORKDIR /usr/src/barretenberg/acir_tests

    ENV TEST_SRC /usr/src/acir_artifacts
    ENV VERBOSE=1

    RUN (cd sol-test && yarn)
    RUN PARALLEL=1 FLOW=sol ./run_acir_tests.sh assert_statement double_verify_proof double_verify_nested_proof

barretenberg-acir-tests-bb.js:
    # Playwright not supported on base image ubuntu:noble, results in unmet dependencies
    FROM ../build-images/+base-slim-node
    RUN apt update && apt install -y curl jq lsof

    COPY ./ts/+build/build/ /usr/src/barretenberg/ts
    COPY ./+acir-tests/ /usr/src/barretenberg/acir_tests
    COPY ../noir/+build-acir-tests/ /usr/src/acir_artifacts

    WORKDIR /usr/src/barretenberg/acir_tests

    # Build/install ts apps.
    RUN cd browser-test-app && yarn && yarn build
    RUN cd headless-test && yarn && npx playwright install && npx playwright install-deps
    RUN cd ../ts && yarn
    ENV VERBOSE=1
    ENV TEST_SRC /usr/src/acir_artifacts

    # TODO(https://github.com/noir-lang/noir/issues/5106)
    # TODO(https://github.com/AztecProtocol/aztec-packages/issues/6672)c
    # Run ecdsa_secp256r1_3x through bb.js on node to check 256k support.
    RUN BIN=../ts/dest/node/main.js FLOW=prove_then_verify ./run_acir_tests.sh ecdsa_secp256r1_3x
    # Run a single arbitrary test not involving recursion through bb.js for UltraHonk
    RUN BIN=../ts/dest/node/main.js FLOW=prove_and_verify_ultra_honk ./run_acir_tests.sh 6_array
    # Run a single arbitrary test not involving recursion through bb.js for MegaHonk
    RUN BIN=../ts/dest/node/main.js FLOW=prove_and_verify_mega_honk ./run_acir_tests.sh 6_array
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
