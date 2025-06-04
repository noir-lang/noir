# SSA Fuzzer

A fuzzing tool for testing and comparing ACIR and Brillig implementations based on [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).

## Overview

This fuzzer generates random sequences of arithmetic and logical operations to craft SSA from scratch and than verify that both ACIR and Brillig implementations produce identical results. It helps catch potential bugs and inconsistencies between the two implementations.


## Usage

1. Setup environment:
```
cargo install cargo-fuzz
```

2. Run fuzzer:
```
cargo +nightly fuzz run base_target --fuzz-dir ./fuzzer
```

or in 5 threads
```
cargo +nightly fuzz run base_target --fuzz-dir ./fuzzer -- -jobs=5 -workers=5
```
