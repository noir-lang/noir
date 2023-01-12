#!/bin/bash
set -e

# Clean.
rm -rf ./build
rm -rf ./build-wasm

# Install formatting git hook.
HOOKS_DIR=$(git rev-parse --git-path hooks)
# The pre-commit script will live in a barretenberg-specific hooks directory
# That may be just in the top level of this repository,
# or may be in a .git/modules/barretenberg subdirectory when this is actually a submodule
# Either way, running `git rev-parse --show-toplevel` from the hooks directory gives the path to barretenberg
echo "cd \$(git rev-parse --show-toplevel)/cpp && ./format.sh staged" > $HOOKS_DIR/pre-commit
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
./download_ignition_lagrange.sh 12
cd ..

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
make -j$(getconf _NPROCESSORS_ONLN) $@
cd ..

# Install the webassembly toolchain.
WASI_VERSION=12
rm -rf ./src/wasi-sdk-*
cd ./src
curl -s -L https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$WASI_VERSION/wasi-sdk-$WASI_VERSION.0-$OS.tar.gz | tar zxfv -
cd ..

# Build WASM.
mkdir -p build-wasm && cd build-wasm
cmake -DTOOLCHAIN=wasm-linux-clang ..
cmake --build . --parallel --target barretenberg.wasm
cd ..
