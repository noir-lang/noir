## Barretenberg, an optimized elliptic curve library for the bn128 curve, and PLONK SNARK prover

**this code is highly experimental, use at your own risk!**

The structured reference string contains monomials up to x^{2^20}. This SRS was generated locally and is for testing and development purposes only!

### Dependencies

cmake version 3.16 or later
clang 9 or gcc 9 or later

### Installing Dependencies (Linux)

(you may need to `sudo` to move clang into `/usr/local`. An alternative is to update PATH to point to a different directory)

```
apt-get update && apt-get install -y \
  xz-utils \
  build-essential \
  curl \
  && curl -SL http://releases.llvm.org/9.0.0/clang+llvm-9.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz | tar -xJC . \
  && mv clang+llvm-9.0.0-x86_64-linux-gnu-ubuntu-18.04 /usr/local/clang_9.0.0

export PATH="/usr/local/clang_9.0.0/bin:$PATH"
export LD_LIBRARY_PATH="/usr/local/clang_9.0.0/lib:$LD_LIBRARY_PATH"
```

### Installing Dependencies (Mac)

```
brew install cmake
brew install llvm
```

### Getting started

Just run the bootstrap script.

```
./bootstrap.sh
```

### Parallelise the build

Make sure your MAKEFLAGS environment variable is set to run jobs equal to number of cores. e.g. `MAKEFLAGS=-j32`.

### Formatting

If you've installed the C++ Vscode extension you should configure it to format on save.
If you've run the bootstrap script, a pre-commit hook is installed to ensure code is formatted before committing.
Ensure `clang-format` is installed. You can also format manually using the format script.
To format the entire codebase:

```
./format.sh
```

To format only staged changes:

```
./format.sh staged
```

### Tests

Each module has its own tests. To build and run, for example `ecc` tests:

```
make ecc_tests
./src/aztec/ecc/ecc_tests
```

Running the entire suite of tests using `ctest`:

```
make test
```

To compile without tests and benchmarks, use `cmake .. -DTESTING=OFF -DBENCHMARKS=OFF`

To select a test, run `<path_to_module_tests> --gtest_filter=<test_filter>*`

### Benchmarks

Some modules have benchmarks. The build targets are named `<module_name>_bench`. To build and run, for example `ecc` benchmarks.

```
make ecc_bench
./src/aztec/ecc/ecc_bench
```

A shorthand for the above is:

```
make run_ecc_bench
```

### Debug build

```
mkdir build && cd build
cmake -DCMAKE_BUILD_TYPE="Debug" ..
make
```

### Build without x64 assembly:

```
mkdir build && cd build
cmake -DDISABLE_ASM=ON ..
make
```

### WASM build

To build:

```
mkdir build-wasm && cd build-wasm
cmake -DWASM=ON ..
make barretenberg.wasm
```

There will be a binary at `./src/aztec/barretenberg.wasm` that can be copied to `barretenberg.js` for use in node and the browser.

#### Testing

To run the tests, you'll need to install `wasmtime`.

```
curl https://wasmtime.dev/install.sh -sSf | bash
```

The WASM build does not currently support `ctest` meaning you must run each modules tests individually.
Tests can be run like:

```
wasmtime --dir=.. ./src/aztec/ecc/ecc_tests
```
