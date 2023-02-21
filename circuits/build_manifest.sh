#!/bin/bash
# Source this file to define the PROJECTS variable.
# PROJECT elements have structure PROJECT:WORKING_DIR:DOCKERFILE:REPO.
#
# TODO: Generate this from build_manifest.json

# Commenting out a few projects, as the main use case is now to build the images needed to run end-to-end tests.
# If wanting to just see if docker images actually build, you can temporarily uncomment required projects.
PROJECTS=(
  aztec3-circuits-wasm:./:./dockerfiles/Dockerfile.wasm-linux-clang:aztec3-circuits-wasm-linux-clang
  aztec3-circuits-wasm-assert:./:./dockerfiles/Dockerfile.wasm-linux-clang-assert:aztec3-circuits-wasm-linux-clang-assert
  aztec3-circuits-x86_64-clang:./:./dockerfiles/Dockerfile.x86_64-linux-clang:aztec3-circuits-x86_64-linux-clang
  aztec3-circuits-x86_64-clang-assert:./:./dockerfiles/Dockerfile.x86_64-linux-clang-assert:aztec3-circuits-x86_64-linux-clang-assert
)