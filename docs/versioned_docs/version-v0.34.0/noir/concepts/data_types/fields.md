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

```rust title="to_le_bits" showLineNumbers 
pub fn to_le_bits<let N: u32>(self: Self) -> [u1; N] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L32-L34" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L32-L34</a></sub></sup>


example:

```rust title="to_le_bits_example" showLineNumbers 
fn test_to_le_bits() {
        let field = 2;
        let bits: [u1; 8] = field.to_le_bits();
        assert_eq(bits, [0, 1, 0, 0, 0, 0, 0, 0]);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L196-L202" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L196-L202</a></sub></sup>



### to_be_bits

Transforms the field into an array of bits, Big Endian.

```rust title="to_be_bits" showLineNumbers 
pub fn to_be_bits<let N: u32>(self: Self) -> [u1; N] {}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L48-L50" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L48-L50</a></sub></sup>


example:

```rust title="to_be_bits_example" showLineNumbers 
fn test_to_be_bits() {
        let field = 2;
        let bits: [u1; 8] = field.to_be_bits();
        assert_eq(bits, [0, 0, 0, 0, 0, 0, 1, 0]);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L187-L193" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L187-L193</a></sub></sup>



### to_le_bytes

Transforms into an array of bytes, Little Endian

```rust title="to_le_bytes" showLineNumbers 
pub fn to_le_bytes<let N: u32>(self: Self) -> [u8; N] {
        self.to_le_radix(256)
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L63-L67" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L63-L67</a></sub></sup>


example:

```rust title="to_le_bytes_example" showLineNumbers 
fn test_to_le_bytes() {
        let field = 2;
        let bits: [u8; 8] = field.to_le_bytes();
        assert_eq(bits, [2, 0, 0, 0, 0, 0, 0, 0]);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L214-L220" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L214-L220</a></sub></sup>


### to_be_bytes

Transforms into an array of bytes, Big Endian

```rust title="to_be_bytes" showLineNumbers 
pub fn to_be_bytes<let N: u32>(self: Self) -> [u8; N] {
        self.to_be_radix(256)
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L80-L84" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L80-L84</a></sub></sup>


example:

```rust title="to_be_bytes_example" showLineNumbers 
fn test_to_be_bytes() {
        let field = 2;
        let bits: [u8; 8] = field.to_be_bytes();
        assert_eq(bits, [0, 0, 0, 0, 0, 0, 0, 2]);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L205-L211" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L205-L211</a></sub></sup>



### to_le_radix

Decomposes into an array over the specified base, Little Endian

```rust title="to_le_radix" showLineNumbers 
pub fn to_le_radix<let N: u32>(self: Self, radix: u32) -> [u8; N] {
        crate::assert_constant(radix);
        self.__to_le_radix(radix)
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L86-L91" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L86-L91</a></sub></sup>



example:

```rust title="to_le_radix_example" showLineNumbers 
fn test_to_le_radix() {
        let field = 2;
        let bits: [u8; 8] = field.to_le_radix(256);
        assert_eq(bits, [2, 0, 0, 0, 0, 0, 0, 0]);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L232-L238" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L232-L238</a></sub></sup>



### to_be_radix

Decomposes into an array over the specified base, Big Endian

```rust title="to_be_radix" showLineNumbers 
pub fn to_be_radix<let N: u32>(self: Self, radix: u32) -> [u8; N] {
        crate::assert_constant(radix);
        self.__to_be_radix(radix)
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L93-L98" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L93-L98</a></sub></sup>


example:

```rust title="to_be_radix_example" showLineNumbers 
fn test_to_be_radix() {
        let field = 2;
        let bits: [u8; 8] = field.to_be_radix(256);
        assert_eq(bits, [0, 0, 0, 0, 0, 0, 0, 2]);
    }
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L223-L229" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L223-L229</a></sub></sup>



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

```rust title="assert_max_bit_size" showLineNumbers 
pub fn assert_max_bit_size(self, bit_size: u32) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L9-L11" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L9-L11</a></sub></sup>


example:

```rust
fn main() {
    let field = 2
    field.assert_max_bit_size(32);
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
