#!/bin/bash
set -eu

BB_TARGETS=(
  libenv.a
  libcommon.a
  libcrypto_blake2s.a
  libcrypto_pedersen_hash.a
  libcrypto_pedersen_commitment.a
  libcrypto_keccak.a
  libcrypto_schnorr.a
  libcrypto_generators.a
  libnumeric.a
)

CMD="cmake --preset wasm && cmake --build --preset wasm"
for target in "${BB_TARGETS[@]}"; do CMD="$CMD --target $target"; done
eval $CMD

cd ./build-wasm/lib

LIBS=(
  $PWD/libenv.a
  $PWD/libcommon.a
  $PWD/libcrypto_blake2s.a
  $PWD/libcrypto_pedersen_hash.a
  $PWD/libcrypto_pedersen_commitment.a
  $PWD/libcrypto_keccak.a
  $PWD/libcrypto_schnorr.a
  $PWD/libcrypto_generators.a
  $PWD/libnumeric.a
  $PWD/../../src/wasi-sdk-20.0/share/wasi-sysroot/lib/wasm32-wasi/libc++.a
  $PWD/../../src/wasi-sdk-20.0/share/wasi-sysroot/lib/wasm32-wasi/libc++abi.a
)

rm -rf scratch
mkdir -p scratch
cd scratch

for LIB_FILE_PATH in "${LIBS[@]}"; do
  LIB=$(basename $LIB_FILE_PATH)
  echo Extracting lib: $LIB
  mkdir $LIB
  cd $LIB
  ar x $LIB_FILE_PATH
  cd ..
done

rm -f ../libacvm_backend.a
#../../../src/wasi-sdk-12.0/bin/ar rcs ../libxyz.a libcrypto_blake2s.a/* libc++.a/* libc++abi.a/* libcrypto_pedersen_commitment.a/*
find . -type f -print0 | xargs -0 ../../../src/wasi-sdk-20.0/bin/ar rcs ../libacvm_backend.a
