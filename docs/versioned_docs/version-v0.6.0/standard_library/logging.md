---
title: Logging
description:
  Learn how to use the println statement for debugging in Noir with this tutorial. Understand the
  basics of logging in Noir and how to implement it in your code.
keywords:
  [
    noir logging,
    println statement,
    debugging in noir,
    noir std library,
    logging tutorial,
    basic logging in noir,
    noir logging implementation,
    noir debugging techniques,
    rust,
  ]
---

# Logging

The standard library provides a familiar `println` statement you can use. Despite being a limited
implementation of rust's `println!` macro, this construct can be useful for debugging.

The `println` statement only works for fields, integers and arrays (including strings).

```rust
use dep::std;

fn main(string: pub str<5>) {
    let x = 5;
    std::println(x)
}

```

To view the output of the `println` statement you need to set the `--show-output` flag.

```
$ nargo prove --help
Create proof for this program. The proof is returned as a hex encoded string

Usage: nargo prove [OPTIONS] [PROOF_NAME] [CIRCUIT_NAME]

Arguments:
  [PROOF_NAME]    The name of the proof
  [CIRCUIT_NAME]  The name of the circuit build files (ACIR, proving and verification keys)

Options:
  -v, --verify          Verify proof after proving
  -s, --show-ssa        Emit debug information for the intermediate SSA IR
  -d, --deny-warnings   Quit execution when warnings are emitted
      --show-output     Display output of `println` statements during tests
  -h, --help            Print help
```
