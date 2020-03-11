### Barretenberg, an optimized elliptic curve library for the bn128 curve, and PLONK SNARK prover

**this code is highly experimental, use at your own risk!**

The structured reference string contains monomials up to x^{2^20}. This SRS was generated locally and is for testing and development purposes only!

### Getting started

```
git clone https://github.com/AztecProtocol/barretenberg

mkdir build && cd build
cmake ..
make -j$(nproc) [optional target name]
```

### Tests

Each module has its own tests. To run, for example `ecc` tests:

```
./src/ecc/ecc_tests
```

Running the entire suite of tests using `ctest`:

```
make test
```

To compile without tests and benchmarks, use `cmake .. -DTESTING=OFF -DBENCHMARKS=OFF`

To select a test, run `<path_to_module_tests> --gtest_filter=<test_filter>*`

### Debug build

```
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE="Debug"
make -j$(nproc)
```

### Build without x64 assembly:

```
mkdir build && cd build
cmake .. -DDISABLE_ASM=ON
make -j$(nproc)
```

### WASM build

```
mkdir build-wasm && cd build-wasm
cmake .. -DWASM=ON
make -j$(nproc) barretenberg.wasm
```

The resulting binary will be at `./src/barretenberg.wasm` and can be copied to `barretenberg.js` for use in node and the browser.

Wasm build does not currently support `ctest` meaning you must run each modules tests individually.

### Modules

The following is the tree of module targets. e.g. to build `keccak` just `make keccack` (ignore the path).

```
crypto
  |_keccak
    blake2s
    pedersen
    sha256
    schnorr
ecc
noir
  |_noir_cli
    noir
numeric
plonk
  |_composer
    pippenger_bench
    proof_system
    reference_string
    transcript
polynomials
rollup
srs
stdlib
  |_stdlib_bit_array
    stdlib_bool
    stdlib_byte_array
    stdlib_field
    stdlib_uint
```

Their test targets are at `<module_name>_tests`.