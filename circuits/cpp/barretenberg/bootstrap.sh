#!/bin/bash
set -e

# Clean.
rm -rf ./build
rm -rf ./build-wasm

# Install formatting git hook.
echo "cd ./barretenberg && ./format.sh staged" > ../.git/hooks/pre-commit
chmod +x ../.git/hooks/pre-commit

# Download ignition transcript 0.
cd ./srs_db
./download_ignition.sh 3
cd ..

# Build native.
mkdir -p build && cd build
cmake -DCMAKE_BUILD_TYPE=RelWithAssert ..
cmake --build . --parallel
cd ..

# Install the webassembly toolchain and patch runtime.
rm -rf ./src/wasi-sdk-12.0
cd ./src
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS=macos
elif [[ "$OSTYPE" == "linux-gnu" ]]; then
    OS=linux
else
    echo "Unknown OS: $OSTYPE"
    exit 1
fi
curl -s -L https://github.com/CraneStation/wasi-sdk/releases/download/wasi-sdk-12/wasi-sdk-12.0-$OS.tar.gz | tar zxfv -
cd ..

# Build WASM.
mkdir -p build-wasm && cd build-wasm
cmake -DTOOLCHAIN=wasm-linux-clang ..
cmake --build . --parallel --target barretenberg.wasm
cd ..
