#!/bin/bash
# Source this file to define the PROJECTS variable.
# PROJECT elements have structure PROJECT:WORKING_DIR:DOCKERFILE:REPO.
#
# TODO: Generate this from build_manifest.json

PROJECTS=(
  circuits-wasm:circuits/cpp:./dockerfiles/Dockerfile.wasm-linux-clang:circuits-wasm-linux-clang
  yarn-project-base:yarn-project
  barretenberg.js:yarn-project
  # end-to-end:yarn-project
)
