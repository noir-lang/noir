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

The size of a Noir field depends on the elliptic curve's finite field for the proving backend  adopted. For example, a field would be a 254-bit integer when paired with the default backend that spans the Grumpkin curve.

Fields support integer arithmetic:

```rust
fn main(x : Field, y : Field)  {
    let z = x + y;
}
```

`x`, `y` and `z` are all private fields in this example. Using the `let` keyword we defined a new private value `z` constrained to be equal to `x + y`.

If proving efficiency is of priority, fields should be used as a default for solving problems. Smaller integer types (e.g. `u64`) incur extra range constraints.

## Security Considerations for Field Arithmetic

Field arithmetic wraps around the field modulus without any overflow checks. This means operations like addition and multiplication can silently produce unexpected results if the values exceed the field modulus. For example, adding `1` to the maximum field value wraps back to `0` with no error.

Many use cases require standard integer arithmetic (e.g., enforcing that balance does not go negative). In these programs using explicitly sized integer types is necessary to catch overflows when proving a Noir program.

## Methods

After declaring a Field, you can use these common methods on it:

### to_le_bits

Transforms the field into an array of bits, Little Endian.

```rust title="to_le_bits" showLineNumbers 
pub fn to_le_bits<let N: u32>(self: Self) -> [bool; N] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L29-L31" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L29-L31</a></sub></sup>


example:

```rust title="to_le_bits_example" showLineNumbers 
fn test_to_le_bits() {
    let field = 2;
    let bits: [bool; 8] = field.to_le_bits();
    assert_eq(bits, [false, true, false, false, false, false, false, false]);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L403-L409" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L403-L409</a></sub></sup>



### to_be_bits

Transforms the field into an array of bits, Big Endian.

```rust title="to_be_bits" showLineNumbers 
pub fn to_be_bits<let N: u32>(self: Self) -> [bool; N] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L61-L63" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L61-L63</a></sub></sup>


example:

```rust title="to_be_bits_example" showLineNumbers 
fn test_to_be_bits() {
    let field = 2;
    let bits: [bool; 8] = field.to_be_bits();
    assert_eq(bits, [false, false, false, false, false, false, true, false]);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L394-L400" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L394-L400</a></sub></sup>



### to_le_bytes

Transforms into an array of bytes, Little Endian

```rust title="to_le_bytes" showLineNumbers 
pub fn to_le_bytes<let N: u32>(self: Self) -> [u8; N] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L93-L95" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L93-L95</a></sub></sup>


example:

```rust title="to_le_bytes_example" showLineNumbers 
fn test_to_le_bytes() {
    let field = 2;
    let bytes: [u8; 8] = field.to_le_bytes();
    assert_eq(bytes, [2, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq(Field::from_le_bytes::<8>(bytes), field);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L422-L429" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L422-L429</a></sub></sup>


### to_be_bytes

Transforms into an array of bytes, Big Endian

```rust title="to_be_bytes" showLineNumbers 
pub fn to_be_bytes<let N: u32>(self: Self) -> [u8; N] {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L130-L132" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L130-L132</a></sub></sup>


example:

```rust title="to_be_bytes_example" showLineNumbers 
fn test_to_be_bytes() {
    let field = 2;
    let bytes: [u8; 8] = field.to_be_bytes();
    assert_eq(bytes, [0, 0, 0, 0, 0, 0, 0, 2]);
    assert_eq(Field::from_be_bytes::<8>(bytes), field);
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L412-L419" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L412-L419</a></sub></sup>


### from_le_bytes

Parses a Little Endian byte array into a field element. If the byte array represents a value
greater than or equal to the field modulus then the result will silently wrap around. Use
[`from_le_bytes_checked`](#from_le_bytes_checked) when canonical inputs are required.

```rust title="from_le_bytes" showLineNumbers 
pub fn from_le_bytes<let N: u32>(bytes: [u8; N]) -> Field {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L210-L212" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L210-L212</a></sub></sup>


### from_be_bytes

Parses a Big Endian byte array into a field element. If the byte array represents a value
greater than or equal to the field modulus then the result will silently wrap around. Use
[`from_be_bytes_checked`](#from_be_bytes_checked) when canonical inputs are required.

```rust title="from_be_bytes" showLineNumbers 
pub fn from_be_bytes<let N: u32>(bytes: [u8; N]) -> Field {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L232-L234" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L232-L234</a></sub></sup>


### from_le_bytes_checked

Like [`from_le_bytes`](#from_le_bytes) but asserts that the input is a canonical representation,
i.e. that the value encoded by `bytes` is strictly less than the field modulus. Triggers a
constraint failure otherwise.

```rust title="from_le_bytes_checked" showLineNumbers 
pub fn from_le_bytes_checked<let N: u32>(bytes: [u8; N]) -> Field {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L255-L257" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L255-L257</a></sub></sup>


### from_be_bytes_checked

Like [`from_be_bytes`](#from_be_bytes) but asserts that the input is a canonical representation,
i.e. that the value encoded by `bytes` is strictly less than the field modulus. Triggers a
constraint failure otherwise.

```rust title="from_be_bytes_checked" showLineNumbers 
pub fn from_be_bytes_checked<let N: u32>(bytes: [u8; N]) -> Field {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L281-L283" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L281-L283</a></sub></sup>


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
pub fn assert_max_bit_size<let BIT_SIZE: u32>(self) {
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/field/mod.nr#L10-L12" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/field/mod.nr#L10-L12</a></sub></sup>


example:

```rust
fn main() {
    let field = 2
    field.assert_max_bit_size::<32>();
}
```

### sgn0

Parity of (prime) Field element, i.e. sgn0(x mod p) = false if x ∈ \{0, ..., p-1\} is even, otherwise sgn0(x mod p) = true.

```rust
fn sgn0(self) -> bool
```


### lt

Returns true if the field is less than the other field

```rust
pub fn lt(self, another: Field) -> bool
```
