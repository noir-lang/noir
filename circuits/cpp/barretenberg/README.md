### Barretenberg, an optimized elliptic curve library for the bn128 curve, and PLONK SNARK prover

**this code is highly experimental, use at your own risk!**

The structured reference string contains monomials up to x^{2^20}. This SRS was generated locally and is for testing and development purposes only!

### Getting started

```
git clone https://github.com/AztecProtocol/barretenberg

mkdir build && cd build
cmake ..
cmake --build .
```

To run tests, in the /build directory run

```
test/barretenberg_tests
```

To run benchmarks, in the /build directory run

```
test/barretenberg_bench
```

To compile without tests and benchmarks, use `cmake .. -DTESTING=OFF -DBENCHMARKS=OFF`

To select a test, run `./test/barretenberg_tests --gtest_filter=<test_filter>*`

To build in debug mode:

```
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE="Debug"
cmake --build .
```

To build without x64 assembly:

```
mkdir build && cd build
cmake .. -DDISABLE_ASM=ON
cmake --build .
```

