## Barretenberg, an optimized elliptic curve library for the bn128 curve, and PLONK SNARK prover

**This code is highly experimental, use at your own risk!**

The structured reference string contains monomials up to x^{2^20}. This SRS was generated locally and is for testing and development purposes only!

### Dependencies

- cmake >= 3.16
- clang >= 10 or gcc >= 10
- clang-format
- libomp (if multithreading is required. Multithreading can be disabled using the compiler flag `-DMULTITHREADING 0`)

### Installing openMP (Linux)

```
RUN git clone -b release/10.x --depth 1 https://github.com/llvm/llvm-project.git \
  && cd llvm-project && mkdir build-openmp && cd build-openmp \
  && cmake ../openmp -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -DLIBOMP_ENABLE_SHARED=OFF \
  && make -j$(nproc) \
  && make install \
  && cd ../.. && rm -rf llvm-project
```

### Getting started

Run the bootstrap script. (The bootstrap script will build both the native and wasm versions of barretenberg)

```
./bootstrap.sh
```

### Parallelise the build

Make sure your MAKEFLAGS environment variable is set to run jobs equal to number of cores. e.g. `MAKEFLAGS=-j$(nproc)`.

### Formatting

Code is formatted using `clang-format` and the `./format.sh` script which is called via a git pre-commit hook.
If you've installed the C++ Vscode extension you should configure it to format on save.

### Testing

Each module has its own tests. e.g. To build and run `ecc` tests:

```
make ecc_tests
./bin/ecc_tests
```

A shorthand for the above is:

```
make run_ecc_tests
```

Running the entire suite of tests using `ctest`:

```
make test
```

You can run specific tests, e.g.

```
./bin/ecc_tests --gtest_filter=scalar_multiplication.*
```

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

### CMake Build Options

CMake can be passed various build options on it's command line:

- `-DCMAKE_BUILD_TYPE=Debug | Release | RelWithAssert`: Build types.
- `-DDISABLE_ASM=ON | OFF`: Enable/disable x86 assembly.
- `-DDISABLE_ADX=ON | OFF`: Enable/disable ADX assembly instructions (for older cpu support).
- `-DMULTITHREADING=ON | OFF`: Enable/disable multithreading using OpenMP.
- `-DTESTING=ON | OFF`: Enable/disable building of tests.
- `-DBENCHMARK=ON | OFF`: Enable/disable building of benchmarks.
- `-DTOOLCHAIN=<filename in ./cmake/toolchains>`: Use one of the preconfigured toolchains.

### WASM build

To build:

```
mkdir build-wasm && cd build-wasm
cmake -DTOOLCHAIN=wasm-linux-clang ..
make barretenberg.wasm
```

The resulting wasm binary will be at `./src/aztec/barretenberg.wasm`.

To run the tests, you'll need to install `wasmtime`.

```
curl https://wasmtime.dev/install.sh -sSf | bash
```

Tests can be built and run like:

```
make ecc_tests
wasmtime --dir=.. ./bin/ecc_tests
```
