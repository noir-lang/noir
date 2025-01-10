---
title: Big Integers
description: How to use big integers from Noir standard library
keywords:
  [
    Big Integer,
    Noir programming language,
    Noir libraries,
  ]
---

The BigInt module in the standard library exposes some class of integers which do not fit (well) into a Noir native field. It implements modulo arithmetic, modulo a 'big' prime number.

:::note

The module can currently be considered as `Field`s with fixed modulo sizes used by a set of elliptic curves, in addition to just the native curve. [More work](https://github.com/noir-lang/noir/issues/510) is needed to achieve arbitrarily sized big integers.

:::note

`nargo` can be built with `--profile release-pedantic` to enable extra overflow checks which may affect `BigInt` results in some cases.
Consider the [`noir-bignum`](https://github.com/noir-lang/noir-bignum) library for an optimized alternative approach.

:::

Currently 6 classes of integers (i.e 'big' prime numbers) are available in the module, namely:

- BN254 Fq: Bn254Fq
- BN254 Fr: Bn254Fr
- Secp256k1 Fq: Secpk1Fq
- Secp256k1 Fr: Secpk1Fr
- Secp256r1 Fr: Secpr1Fr
- Secp256r1 Fq: Secpr1Fq

Where XXX Fq and XXX Fr denote respectively the order of the base and scalar field of the (usual) elliptic curve XXX.
For instance the big integer 'Secpk1Fq' in the standard library refers to integers modulo $2^{256}-2^{32}-977$.

Feel free to explore the source code for the other primes:

```rust title="big_int_definition" showLineNumbers 
pub struct BigInt {
    pointer: u32,
    modulus: u32,
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/bigint.nr#L14-L19" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/bigint.nr#L14-L19</a></sub></sup>


## Example usage

A common use-case is when constructing a big integer from its bytes representation, and performing arithmetic operations on it:

```rust title="big_int_example" showLineNumbers 
fn big_int_example(x: u8, y: u8) {
    let a = Secpk1Fq::from_le_bytes(&[x, y, 0, 45, 2]);
    let b = Secpk1Fq::from_le_bytes(&[y, x, 9]);
    let c = (a + b) * b / a;
    let d = c.to_le_bytes();
    println(d[0]);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/test_programs/execution_success/bigint/src/main.nr#L72-L80" target="_blank" rel="noopener noreferrer">Source code: test_programs/execution_success/bigint/src/main.nr#L72-L80</a></sub></sup>


## Methods

The available operations for each big integer are:

### from_le_bytes

Construct a big integer from its little-endian bytes representation. Example:

```rust
 // Construct a big integer from a slice of bytes
 let a = Secpk1Fq::from_le_bytes(&[x, y, 0, 45, 2]);
 // Construct a big integer from an array of 32 bytes
 let a = Secpk1Fq::from_le_bytes_32([1;32]);
 ```

Sure, here's the formatted version of the remaining methods:

### to_le_bytes

Return the little-endian bytes representation of a big integer. Example:

```rust
let bytes = a.to_le_bytes();
```

### add

Add two big integers. Example:

```rust
let sum = a + b;
```

### sub

Subtract two big integers. Example:

```rust
let difference = a - b;
```

### mul

Multiply two big integers. Example:

```rust
let product = a * b;
```

### div

Divide two big integers. Note that division is field division and not euclidean division. Example:

```rust
let quotient = a / b;
```

### eq

Compare two big integers. Example:

```rust
let are_equal = a == b;
```
