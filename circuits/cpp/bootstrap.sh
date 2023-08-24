#!/bin/bash
# Takes a list of targets from commandline
# Takes CLEAN as an environment variable. If passed, cleans build artifacts
set -eu

export WASI_VERSION=20

# Navigate to script folder
cd "$(dirname "$0")"

# Update the submodule
git submodule update --init --recursive

# Remove all untracked files and directories.
if [ -n "${CLEAN:-}" ]; then
  # Clean.
  rm -rf ./build
  rm -rf ./build-wasm

  # Clean barretenberg.
  rm -rf ./barretenberg/cpp/build
  rm -rf ./barretenberg/cpp/build-wasm
  rm -rf ./barretenberg/cpp/src/wasi-sdk-*
fi

# Install formatting git hook.
HOOKS_DIR=$(git rev-parse --git-path hooks)
echo "cd \$(git rev-parse --show-toplevel)/circuits/cpp && ./format.sh staged" > $HOOKS_DIR/pre-commit
chmod +x $HOOKS_DIR/pre-commit

# Determine system.
if [[ "$OSTYPE" == "darwin"* ]]; then
  OS=macos
elif [[ "$OSTYPE" == "linux-gnu" ]]; then
  OS=linux
else
  echo "Unknown OS: $OSTYPE"
  exit 1
fi

# Download ignition transcripts.
(cd barretenberg/cpp/srs_db && ./download_ignition.sh 3)

# Pick native toolchain file.
ARCH=$(uname -m)
if [ "$OS" == "macos" ]; then
  if [ "$(which brew)" != "" ]; then
    export BREW_PREFIX=$(brew --prefix)

    # Ensure we have toolchain.
    if [ ! "$?" -eq 0 ] || [ ! -f "$BREW_PREFIX/opt/llvm/bin/clang++" ]; then
      echo "Default clang not sufficient. Install homebrew, and then: brew install llvm libomp clang-format"
      exit 1
    fi

    PRESET=homebrew
  else
    PRESET=default
  fi
else
  if [ "$(which clang++-15)" != "" ]; then
    PRESET=clang15
  else
    PRESET=default
  fi
fi

echo "#################################"
echo "# Building with preset: $PRESET"
echo "# When running cmake directly, remember to use: --build --preset $PRESET"
echo "#################################"

# Build native.
cmake --preset $PRESET -DCMAKE_BUILD_TYPE=RelWithAssert
cmake --build --preset $PRESET ${@/#/--target }

# Install the webassembly toolchain.
(cd ./barretenberg/cpp && ./scripts/install-wasi-sdk.sh)

# Build WASM.
cmake --preset wasm
cmake --build --preset wasm
