#!/bin/bash
set -eu

# Clean.
rm -rf ./build
rm -rf ./build-wasm

# Install formatting git hook.
HOOKS_DIR=$(git rev-parse --git-path hooks)
# The pre-commit script will live in a barretenberg-specific hooks directory
# Find it based on the script directory.
# TODO(AD) it is a bit unintuitive that bootstrap sets one up for a barretenberg-oriented workflow
SCRIPT_PWD=`pwd` # fix: capture pwd instead of writing into script
echo "cd $SCRIPT_PWD && ./format.sh staged" > $HOOKS_DIR/pre-commit
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
cd ./srs_db
./download_ignition.sh 3
cd ..

# Pick native toolchain file.
ARCH=$(uname -m)
if [ "$OS" == "macos" ]; then
  PRESET=default
else
  if [ "$(which clang++-15)" != "" ]; then
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

cd ./build
# The Grumpkin SRS is generated manually at the moment, only up to a large enough size for tests
# If tests require more points, the parameter can be increased here.
cmake --build . --parallel --target grumpkin_srs_gen
./bin/grumpkin_srs_gen 8192
echo "Generated Grumpkin SRS successfully"
cd ../

# Install wasi-sdk.
./scripts/install-wasi-sdk.sh

# Build WASM.
cmake --preset wasm
cmake --build --preset wasm

# Build WASM with new threading.
cmake --preset wasm-threads
cmake --build --preset wasm-threads
