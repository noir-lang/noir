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
fi

# Determine system.
if [[ "$OSTYPE" == "darwin"* ]]; then
  OS=macos
elif [[ "$OSTYPE" == "linux-gnu" ]]; then
  OS=linux
else
  echo "Unknown OS: $OSTYPE"
  exit 1
fi

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
  if [ "$(which clang++-16)" != "" ]; then
    PRESET=clang16
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

# Build WASM.
if [ -n "${WASM_DEBUG:-}" ] ; then
  cmake --preset wasm-dbg
  cmake --build --preset wasm-dbg
else
  cmake --preset wasm
  cmake --build --preset wasm
fi
