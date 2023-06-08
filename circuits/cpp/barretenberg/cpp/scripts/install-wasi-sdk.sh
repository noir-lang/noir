#!/bin/bash
set -eu

# Clean.
rm -rf ./src/wasi-sdk-*

# Determine system.
if [[ "$OSTYPE" == "darwin"* ]]; then
  OS=macos
elif [[ "$OSTYPE" == "linux-gnu" ]]; then
  OS=linux
else
  echo "Unknown OS: $OSTYPE"
  exit 1
fi

# Install the webassembly toolchain.
mkdir -p src
cd ./src
curl -s -L https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0-$OS.tar.gz | tar zxfv -
if [ "$OS" == "linux" ]; then
  # Exceptional linux case that provides an llvm build that works on Ubuntu 20.
  curl -s -L https://wasi-sdk.s3.eu-west-2.amazonaws.com/yamt-wasi-sdk-20.0.threads.tgz | tar zxfv -
else
  # For other operating systems, first download the standard release (this is to get the llvm build).
  curl -s -L https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20%2Bthreads/wasi-sdk-20.0.threads-$OS.tar.gz | tar zxfv -
  # Replace wasi-sysroot in wasi-sdk-20.0+threads with our custom build.
  # It contains libc++ and a patch by yamt to improve thread join stability.
  # Can remove once future releases are more stable.
  curl -s -L https://wasi-sdk.s3.eu-west-2.amazonaws.com/yamt-wasi-sysroot-20.0.threads.tgz | tar zxfv -
fi
