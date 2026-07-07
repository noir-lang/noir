---
title: Integers
description: Explore the Integer data type in Noir. Learn about its methods, see real-world examples, and grasp how to efficiently use Integers in your Noir code.
keywords: [noir, integer types, methods, examples, arithmetic]
sidebar_position: 1
---

An integer type is a range constrained field type.
The Noir frontend supports both unsigned and signed integer types.
The allowed sizes are 1, 8, 16, 32, 64 and 128 bits. ([currently only unsigned integers for 128 bits](https://github.com/noir-lang/noir/issues/7591))

:::info

When an integer is defined in Noir without a specific type, it will default to `Field`.

The one exception is for loop indices which default to `u32` since comparisons on `Field`s are not possible.

:::

## Unsigned Integers

An unsigned integer type is specified first with the letter `u` (indicating its unsigned nature) followed by its bit size (e.g. `8`):

```rust
fn main() {
    let x: u8 = 1;
    let y: u8 = 1;
    let z = x + y;
    assert (z == 2);
}
```

The bit size determines the maximum value the integer type can store. For example, a `u8` variable can store a value in the range of 0 to 255 (i.e. $\\2^{8}-1\\$).

## Signed Integers

A signed integer type is specified first with the letter `i` (which stands for integer) followed by its bit size (e.g. `8`):

```rust
fn main() {
    let x: i8 = -1;
    let y: i8 = -1;
    let z = x + y;
    assert (z == -2);
}
```

The bit size determines the maximum and minimum range of value the integer type can store. For example, an `i8` variable can store a value in the range of -128 to 127 (i.e. $\\-2^{7}\\$ to $\\2^{7}-1\\$).


```rust
fn main(x: i16, y: i16) {
    // modulo
    let c = x % y;
    let c = x % -13;
}
```

Modulo operation is defined for negative integers thanks to integer division, so that the equality `x = (x/y)*y + (x%y)` holds.

## Overflows

Computations that exceed the type boundaries will result in overflow errors. This happens with both signed and unsigned integers. For example, attempting to prove:

```rust
fn main(x: u8, y: u8) -> pub u8 {
    let z = x + y;
    z
}
```

With:

```toml
x = "255"
y = "1"
```

Would result in:

```
$ nargo execute
error: Assertion failed: 'attempt to add with overflow'
┌─ ~/src/main.nr:9:13
│
│     let z = x + y;
│             -----
│
= Call stack:
  ...
```

A similar error would happen with signed integers:

```rust
fn main() -> i8 {
    let x: i8 = -118;
    let y: i8 = -11;
    let z = x + y;
    z
}
```

Note that if a computation ends up being unused the compiler might remove it and it won't end up producing an overflow:

```rust
fn main() {
    // "255 + 1" would overflow, but `z` is unused so no computation happens
    let z: u8 = 255 + 1;
}
```

### Wrapping methods

Although integer overflow is expected to error, some use-cases rely on wrapping. For these use-cases, the standard library provides `wrapping` variants of certain common operations via Wrapping traits in `std::ops`

```rust
fn wrapping_add(self, y: Self) -> Self;
fn wrapping_sub(self, y: Self) -> Self;
fn wrapping_mul(self, y: Self) -> Self;
```

Example of how it is used:

```rust
use std::ops::WrappingAdd
fn main(x: u8, y: u8) -> pub u8 {
    x.wrapping_add(y)
}
```
