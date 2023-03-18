#!/bin/bash
# Source this file to define the PROJECTS variable.
# PROJECT elements have structure PROJECT:WORKING_DIR:DOCKERFILE:REPO.
#
# TODO: Generate this from build_manifest.json

# Commenting out a few projects, as the main use case is now to build the images needed to run end-to-end tests.
# If wanting to just see if docker images actually build, you can temporarily uncomment required projects.
PROJECTS=(
  # circuits-x86_64:circuits:./dockerfiles/Dockerfile.x86_64-linux-clang:circuits-x86_64-linux-clang
  # circuits-wasm:circuits:./dockerfiles/Dockerfile.wasm-linux-clang:circuits-wasm-linux-clang
  # l1-contracts:l1-contracts
  # yarn-project-base:yarn-project
  # acir-simulator:yarn-project
  # archiver:yarn-project
  # aztec-cli:yarn-project
  # aztec.js:yarn-project
  # end-to-end:yarn-project
  # ethereum.js:yarn-project
  # kernel-simulator:yarn-project
  # key-store:yarn-project
  # p2p:yarn-project
  # prover-client:yarn-project
  # public-client:yarn-project
  # sequencer-client:yarn-project
  # wallet:yarn-project
)
