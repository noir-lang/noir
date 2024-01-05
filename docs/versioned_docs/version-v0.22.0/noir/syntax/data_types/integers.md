---
title: Integers
description: Explore the Integer data type in Noir. Learn about its methods, see real-world examples, and grasp how to efficiently use Integers in your Noir code.
keywords: [noir, integer types, methods, examples, arithmetic]
sidebar_position: 1
---

An integer type is a range constrained field type. The Noir frontend supports arbitrarily-sized, both unsigned and signed integer types.

:::info

When an integer is defined in Noir without a specific type, it will default to `Field`.

The one exception is for loop indices which default to `u64` since comparisons on `Field`s are not possible.

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

:::tip

If you are using the default proving backend with Noir, both even (e.g. _u2_, _i2_) and odd (e.g. _u3_, _i3_) arbitrarily-sized integer types up to 127 bits (i.e. _u127_ and _i127_) are supported.

:::

## Overflows

Computations that exceed the type boundaries will result in overflow errors. This happens with both signed and unsigned integers. For example, attempting to prove:

```rust
fn main(x: u8, y: u8) {
    let z = x + y;
}
```

With:

```toml
x = "255"
y = "1"
```

Would result in:

```
$ nargo prove
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
fn main() {
    let x: i8 = -118;
    let y: i8 = -11;
    let z = x + y;
}
```

### Wrapping methods

Although integer overflow is expected to error, some use-cases rely on wrapping. For these use-cases, the standard library provides `wrapping` variants of certain common operations:

```rust
fn wrapping_add<T>(x: T, y: T) -> T;
fn wrapping_sub<T>(x: T, y: T) -> T;
fn wrapping_mul<T>(x: T, y: T) -> T;
```

Example of how it is used:

```rust
use dep::std;

fn main(x: u8, y: u8) -> pub u8 {
    std::wrapping_add(x + y)
}
```
