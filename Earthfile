VERSION 0.8
FROM ubuntu:lunar

build-ci:
    BUILD ./avm-transpiler/+build
    BUILD ./barretenberg/cpp/+build-release
    BUILD ./barretenberg/cpp/+preset-wasm
    BUILD ./barretenberg/cpp/+build-gcc
    BUILD ./barretenberg/cpp/+build-fuzzing
    BUILD ./barretenberg/cpp/+build-clang-assert
    BUILD ./barretenberg/cpp/+test-clang-format
    BUILD ./barretenberg/cpp/+test-clang-format
    BUILD ./boxes/+build
    BUILD ./noir/+packages
    BUILD ./noir/+nargo
    BUILD ./noir-projects/+build
    BUILD ./yarn-project/+build
    BUILD +test-end-to-end

build-ci-small:
    BUILD ./yarn-project/end-to-end/+e2e-escrow-contract

build:
    # yarn-project has the entry point to Aztec
    BUILD ./yarn-project/+build

test-end-to-end:
    BUILD ./yarn-project/end-to-end/+test-all

bench:
  RUN echo hi
