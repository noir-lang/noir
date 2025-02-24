---
title: Fields
description:
  Dive deep into the Field data type in Noir. Understand its methods, practical examples, and best practices to effectively use Fields in your Noir programs.
keywords:
  [
    noir,
    field type,
    methods,
    examples,
    best practices,
  ]
sidebar_position: 0
---

The field type corresponds to the native field type of the proving backend.

The size of a Noir field depends on the elliptic curve's finite field for the proving backend
adopted. For example, a field would be a 254-bit integer when paired with the default backend that
spans the Grumpkin curve.

Fields support integer arithmetic and are often used as the default numeric type in Noir:

```rust
fn main(x : Field, y : Field)  {
    let z = x + y;
}
```

`x`, `y` and `z` are all private fields in this example. Using the `let` keyword we defined a new
private value `z` constrained to be equal to `x + y`.

If proving efficiency is of priority, fields should be used as a default for solving problems.
Smaller integer types (e.g. `u64`) incur extra range constraints.

## Methods

After declaring a Field, you can use these common methods on it:

### to_le_bits

Transforms the field into an array of bits, Little Endian.

#include_code to_le_bits noir_stdlib/src/field/mod.nr rust

example:

#include_code to_le_bits_example noir_stdlib/src/field/mod.nr rust


### to_be_bits

Transforms the field into an array of bits, Big Endian.

#include_code to_be_bits noir_stdlib/src/field/mod.nr rust

example:

#include_code to_be_bits_example noir_stdlib/src/field/mod.nr rust


### to_le_bytes

Transforms into an array of bytes, Little Endian

#include_code to_le_bytes noir_stdlib/src/field/mod.nr rust

example:

#include_code to_le_bytes_example noir_stdlib/src/field/mod.nr rust

### to_be_bytes

Transforms into an array of bytes, Big Endian

#include_code to_be_bytes noir_stdlib/src/field/mod.nr rust

example:

#include_code to_be_bytes_example noir_stdlib/src/field/mod.nr rust


### to_le_radix

Decomposes into an array over the specified base, Little Endian

#include_code to_le_radix noir_stdlib/src/field/mod.nr rust


example:

#include_code to_le_radix_example noir_stdlib/src/field/mod.nr rust


### to_be_radix

Decomposes into an array over the specified base, Big Endian

#include_code to_be_radix noir_stdlib/src/field/mod.nr rust

example:

#include_code to_be_radix_example noir_stdlib/src/field/mod.nr rust


### pow_32

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

### assert_max_bit_size

Adds a constraint to specify that the field can be represented with `bit_size` number of bits

#include_code assert_max_bit_size noir_stdlib/src/field/mod.nr rust

example:

```rust
fn main() {
    let field = 2
    field.assert_max_bit_size::<32>();
}
```

### sgn0

Parity of (prime) Field element, i.e. sgn0(x mod p) = 0 if x ∈ \{0, ..., p-1\} is even, otherwise sgn0(x mod p) = 1.

```rust
fn sgn0(self) -> u1
```


### lt

Returns true if the field is less than the other field

```rust
pub fn lt(self, another: Field) -> bool
```
