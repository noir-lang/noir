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
Currently only 6 class of integers (i.e 'big' prime numbers) are available throughout the module, namely:
- BN254 Fq: Bn254Fq
- BN254 Fr: Bn254Fr
- Secp256k1 Fq: Secpk1Fq
- Secp256k1 Fr: Secpk1Fr
- Secp256r1 Fr: Secpr1Fr
- Secp256r1 Fq: Secpr1Fq

Where XXX Fq and XXX Fr denote respectively the order of the base and scalar field of the (usual) elliptic curve XXX.
For instance the big integer 'Secpk1Fq' in the standard library refers to integers modulo $2^{256}-2^{32}-977$

In order to use it, you need first to import the module:
```rust
use dep::std::bigint::Secpk1Fq;
```

Then you can construct a big integer from its bytes representation, and perform arithmetic operations on it:
```rust
 let a = Secpk1Fq::from_le_bytes([x, y, 0, 45, 2]);
 let b = Secpk1Fq::from_le_bytes([y, x, 9]);
 let c = (a + b)*b/a;
 let d = c.to_le_bytes();
 dep::std::println(d[0]);
```

Here is the list of available operations:
- from_le_bytes: construct a big integer from its little-endian bytes representation
```rust
 let a = Secpk1Fq::from_le_bytes([x, y, 0, 45, 2]);
 ```
- to_le_bytes: return the little-endian bytes representation of a big integer
```rust
 a.to_le_bytes();
 ```
- add: add two big integers
```rust
a + b;
 ```
- sub: subtract two big integers
 ```rust
a - b;
 ```
- mul: multiply two big integers
```rust
a * b;
 ```
- div: divide two big integers
Note that division is field division and not euclidean division.
```rust
a / b;
 ```
- eq: compare two big integers
```rust
a == b;
 ```
