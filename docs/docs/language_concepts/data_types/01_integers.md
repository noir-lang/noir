---
title: Integers
description: Explore the Integer data type in Noir. Learn about its methods, see real-world examples, and grasp how to efficiently use Integers in your Noir code.
keywords: [noir, integer types, methods, examples, arithmetic]
---

An integer type is a range constrained field type. The Noir frontend supports arbitrarily-sized, both unsigned and signed integer types.

> **Note:** When an integer is defined in Noir without a specific type, it will default to `Field`. The one exception is for loop indices which default to `u64` since comparisons on `Field`s are not possible.

## Unsigned Integers

An unsigned integer type is specified first with the letter `u`, indicating its unsigned nature, followed by its length in bits (e.g. `8`):

```rust
fn main() {
    let x : u8 = 1;
    let y : u8 = 1;
    let z = x + y;
    assert (z == 2);
}
```

The length in bits determines the boundaries the integer type can store. For example, a `u8` variable can store a value in the range of 0 to 255 (i.e. $\\2^{8}-1\\$).

Computations that exceed the type boundaries would result in overflow errors. For example, attempting to prove:

```rust
fn main(x : u8, y : u8) {
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

> **Note:** The default proving backend supports both even (e.g. _u2_) and odd (e.g. _u3_) arbitrarily-sized integer types up to _u127_.

## Signed Integers

A signed integer type is specified first with the letter `i`, stands for integer, followed by its length in bits (e.g. `8`):

```rust
fn main() {
    let x : i8 = -1;
    let y : i8 = -1;
    let z = x + y;
    assert (z == -2);
}
```

The length in bits determines the boundaries the integer type can store. For example, a `i8` variable can store a value in the range of -128 to 127 (i.e. $\\-2^{7}\\$ to $\\2^{7}-1\\$).

Computations that exceed the type boundaries would result in overflow errors. For example, attempting to prove:

```rust
fn main() {
    let x : i8 = -118;
    let y : i8 = -11;
    let z = x + y;
}
```

Would result in:

```
$ nargo prove
error: Assertion failed: 'attempt to add with overflow'
┌─ ~/src/main.nr:4:13
│
│     let z = x + y;
│             -----
│
= Call stack:
  ...
```

> **Note:** The default proving backend supports both even (e.g. _i2_) and odd (e.g. _i3_) arbitrarily-sized integer types up to _i127_.
