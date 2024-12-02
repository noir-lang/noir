---
title: Integers
description: Explore the Integer data type in Noir. Learn about its methods, see real-world examples, and grasp how to efficiently use Integers in your Noir code.
keywords: [noir, integer types, methods, examples, arithmetic]
sidebar_position: 1
---

An integer type is a range constrained field type.
The Noir frontend supports both unsigned and signed integer types.
The allowed sizes are 1, 8, 16, 32 and 64 bits.

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


```rust
fn main(x: i16, y: i16) {
    // modulo
    let c = x % y;
    let c = x % -13;
}
```

Modulo operation is defined for negative integers thanks to integer division, so that the equality `x = (x/y)*y + (x%y)` holds.

## 128 bits Unsigned Integers

The built-in structure `U128` allows you to use 128-bit unsigned integers almost like a native integer type. However, there are some differences to keep in mind:
- You cannot cast between a native integer and `U128`
- There is a higher performance cost when using `U128`, compared to a native type.

Conversion between unsigned integer types and U128 are done through the use of `from_integer` and `to_integer` functions. `from_integer` also accepts the `Field` type as input.

```rust
fn main() {
    let x = U128::from_integer(23);
    let y = U128::from_hex("0x7");
    let z = x + y;
    assert(z.to_integer() == 30);
}
```

`U128` is implemented with two 64 bits limbs, representing the low and high bits, which explains the performance cost. You should expect `U128` to be twice more costly for addition and four times more costly for multiplication.
You can construct a U128 from its limbs:
```rust
fn main(x: u64, y: u64) {
    let x = U128::from_u64s_be(x,y);
    assert(z.hi == x as Field);
    assert(z.lo == y as Field);
}
```

Note that the limbs are stored as Field elements in order to avoid unnecessary conversions.
Apart from this, most operations will work as usual:

```rust
fn main(x: U128, y: U128) {
    // multiplication
    let c = x * y;
    // addition and subtraction
    let c = c - x + y;
    // division
    let c = x / y;
    // bit operation;
    let c = x & y | y;
    // bit shift
    let c = x << y;
    // comparisons;
    let c = x < y;
    let c = x == y;
}
```

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

fn main(x: u8, y: u8) -> pub u8 {
    std::wrapping_add(x, y)
}
```
