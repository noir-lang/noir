# wasm-worker

Exposes functionality to create asynchronous workers that host webassembly.
The package is generic, but its tests require a barretenberg.wasm.
They assume that you have a file structure as follows:

- aztec3-packages/yarn-project/wasm-worker
- barretenberg
  clone of https://github.com/AztecProtocol/barretenberg
  which has ./cpp/build-wasm/bin/barretenberg.wasm from running cpp/bootstrap.sh
