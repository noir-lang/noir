---
title: Field Methods
description:
  Learn about common methods on Noir Field, including to_le_bits, to_le_bytes, to_le_radix,
  to_be_radix, pow_32, etc, and see code examples.
keywords:
  [
    Noir Field,
    to_le_bits,
    to_le_bytes,
    to_le_radix,
    to_be_radix,
    pow_32,
    Little Endian,
    Big Endian,
    Vector,
    Exponent,
  ]
---

After declaring a Field, you can use these common methods on it:

## to_le_bits

Transforms the field into an array of bits, Little Endian.

```rust
fn to_le_bits<N>(_x : Field, _bit_size: u32) -> [u1; N]
```

example:

```rust
fn main() {
    let field = 2
    let bits = field.to_le_bits(32);
}
```

## to_be_bits

Transforms the field into an array of bits, Big Endian.

```rust
fn to_be_bits<N>(_x : Field, _bit_size: u32) -> [u1; N]
```

example:

```rust
fn main() {
    let field = 2
    let bits = field.to_be_bits(32);
}
```

## to_le_bytes

Transforms into an array of bytes, Little Endian

```rust
fn to_le_bytes(_x : Field, byte_size: u32) -> [u8]
```

example:

```rust
fn main() {
    let field = 2
    let bytes = field.to_le_bytes(4);
}
```

## to_be_bytes

Transforms into an array of bytes, Big Endian

```rust
fn to_be_bytes(_x : Field, byte_size: u32) -> [u8]
```

example:

```rust
fn main() {
    let field = 2
    let bytes = field.to_be_bytes(4);
}
```

## to_le_radix

Decomposes into a vector over the specified base, Little Endian

```rust
fn to_le_radix(_x : Field, _radix: u32, _result_len: u32) -> [u8]
```

example:

```rust
fn main() {
    let field = 2
    let radix = field.to_le_radix(256, 4);
}
```

## to_be_radix

Decomposes into a vector over the specified base, Big Endian

```rust
fn to_be_radix(_x : Field, _radix: u32, _result_len: u32) -> [u8]
```

example:

```rust
fn main() {
    let field = 2
    let radix = field.to_be_radix(256, 4);
}
```

## pow_32

Returns the value to the power of the specified exponent

```rust
fn pow_32(self, exponent: Field) -> Field
```

example:

```rust
fn main() {
    let field = 2
    let pow = field.pow_32(4);
    assert(pow == 16);
}
```

## sgn0

Parity of (prime) Field element, i.e. sgn0(x mod p) = 0 if x ∈ {0, ..., p-1} is even, otherwise sgn0(x mod p) = 1.

```rust
fn sgn0(self) -> u1
```
