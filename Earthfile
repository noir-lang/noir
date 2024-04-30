VERSION 0.8
FROM ubuntu:lunar

build-ci:
    BUILD ./avm-transpiler/+build
    BUILD ./barretenberg/cpp/+preset-release
    BUILD ./barretenberg/cpp/+preset-wasm
    BUILD ./barretenberg/cpp/+preset-gcc
    BUILD ./barretenberg/cpp/+preset-fuzzing
    BUILD ./barretenberg/cpp/+preset-clang-assert
    BUILD ./barretenberg/cpp/+test-clang-format
    BUILD ./boxes/+build
    BUILD ./noir/+packages
    BUILD ./noir/+nargo
    BUILD ./noir-projects/+build
    BUILD ./yarn-project/+end-to-end
    BUILD ./yarn-project/+aztec

build-ci-small:
    BUILD ./yarn-project/end-to-end/+e2e-escrow-contract

build:
    # yarn-project has the entry point to Aztec
    BUILD ./yarn-project/+build

test-end-to-end:
    BUILD ./yarn-project/end-to-end/+test-all

bench:
  RUN echo hi

release-meta:
    COPY .release-please-manifest.json /usr/src/.release-please-manifest.json
    SAVE ARTIFACT /usr/src /usr/src
