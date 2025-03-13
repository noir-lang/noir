# SSA Fuzzer

A fuzzing tool for testing and comparing ACIR and Brillig implementations.

## Overview

This fuzzer generates random sequences of arithmetic and logical operations to craft SSA from scratch and than verify that both ACIR and Brillig implementations produce identical results. It helps catch potential bugs and inconsistencies between the two implementations.


## Usage

1. Setup environment:
You need to delete `rust-toolchain.toml` from root of the project, because cargo-fuzz requires nightly compiler for address sanitizer.
```
rustup install nightly
rustup default nightly
cargo install cargo-fuzz
```

2. Run fuzzer:
```
cargo fuzz run TARGET --fuzz-dir ./fuzzer
```

or in 5 threads
```
cargo-fuzz run TARGET --fuzz-dir ./fuzzer -- -jobs=5 -workers=5
```

### Supported targets

1. uint
2. field