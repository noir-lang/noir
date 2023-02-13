#!/bin/bash
set -e

# Update the submodule
git submodule update --init --recursive --remote

# Clean.
rm -rf ./build
rm -rf ./build-wasm
rm -rf ./src/wasi-sdk-*

# Clean barretenberg.
rm -rf ../barretenberg/cpp/build
rm -rf ../barretenberg/cpp/build-wasm
rm -rf ../barretenberg/cpp/src/wasi-sdk-*

# Install formatting git hook.
HOOKS_DIR=$(git rev-parse --git-path hooks)
echo "cd \$(git rev-parse --show-toplevel) && ./format.sh staged" > $HOOKS_DIR/pre-commit
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
if [ "$OS" == "macos" ]; then
    export BREW_PREFIX=$(brew --prefix)
    # Ensure we have toolchain.
    if [ ! "$?" -eq 0 ] || [ ! -f "$BREW_PREFIX/opt/llvm/bin/clang++" ]; then
        echo "Default clang not sufficient. Install homebrew, and then: brew install llvm libomp clang-format"
        exit 1
    fi
    ARCH=$(uname -m)
    if [ "$ARCH" = "arm64" ]; then
        TOOLCHAIN=arm-apple-clang
    else
        TOOLCHAIN=x86_64-apple-clang
    fi
else
    TOOLCHAIN=x86_64-linux-clang
fi

# Build native.
mkdir -p build && cd build
cmake -DCMAKE_BUILD_TYPE=RelWithAssert -DTOOLCHAIN=$TOOLCHAIN ..
cmake --build . --parallel ${@/#/--target }
cd ..