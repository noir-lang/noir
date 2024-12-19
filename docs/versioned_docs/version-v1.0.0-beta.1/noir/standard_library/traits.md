---
title: Traits
description: Noir's stdlib provides a few commonly used traits.
keywords: [traits, trait, interface, protocol, default, add, eq]
---

## `std::default`

### `std::default::Default`

```rust title="default-trait" showLineNumbers 
pub trait Default {
    fn default() -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/default.nr#L4-L8" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/default.nr#L4-L8</a></sub></sup>


Constructs a default value of a type.

Implementations:
```rust
impl Default for Field { .. }

impl Default for i8 { .. }
impl Default for i16 { .. }
impl Default for i32 { .. }
impl Default for i64 { .. }

impl Default for u8 { .. }
impl Default for u16 { .. }
impl Default for u32 { .. }
impl Default for u64 { .. }

impl Default for () { .. }
impl Default for bool { .. }

impl<T, N> Default for [T; N]
    where T: Default { .. }

impl<T> Default for [T] { .. }

impl<A, B> Default for (A, B)
    where A: Default, B: Default { .. }

impl<A, B, C> Default for (A, B, C)
    where A: Default, B: Default, C: Default { .. }

impl<A, B, C, D> Default for (A, B, C, D)
    where A: Default, B: Default, C: Default, D: Default { .. }

impl<A, B, C, D, E> Default for (A, B, C, D, E)
    where A: Default, B: Default, C: Default, D: Default, E: Default { .. }
```

For primitive integer types, the return value of `default` is `0`. Container
types such as arrays are filled with default values of their element type,
except slices whose length is unknown and thus defaulted to zero.

---

## `std::convert`

### `std::convert::From`

```rust title="from-trait" showLineNumbers 
pub trait From<T> {
    fn from(input: T) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/convert.nr#L1-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/convert.nr#L1-L5</a></sub></sup>


The `From` trait defines how to convert from a given type `T` to the type on which the trait is implemented.

The Noir standard library provides a number of implementations of `From` between primitive types.
```rust title="from-impls" showLineNumbers 
// Unsigned integers

impl From<u8> for u32 {
    fn from(value: u8) -> u32 {
        value as u32
    }
}

impl From<u8> for u64 {
    fn from(value: u8) -> u64 {
        value as u64
    }
}
impl From<u32> for u64 {
    fn from(value: u32) -> u64 {
        value as u64
    }
}

impl From<u8> for Field {
    fn from(value: u8) -> Field {
        value as Field
    }
}
impl From<u32> for Field {
    fn from(value: u32) -> Field {
        value as Field
    }
}
impl From<u64> for Field {
    fn from(value: u64) -> Field {
        value as Field
    }
}

// Signed integers

impl From<i8> for i32 {
    fn from(value: i8) -> i32 {
        value as i32
    }
}

impl From<i8> for i64 {
    fn from(value: i8) -> i64 {
        value as i64
    }
}
impl From<i32> for i64 {
    fn from(value: i32) -> i64 {
        value as i64
    }
}

// Booleans
impl From<bool> for u8 {
    fn from(value: bool) -> u8 {
        value as u8
    }
}
impl From<bool> for u32 {
    fn from(value: bool) -> u32 {
        value as u32
    }
}
impl From<bool> for u64 {
    fn from(value: bool) -> u64 {
        value as u64
    }
}
impl From<bool> for i8 {
    fn from(value: bool) -> i8 {
        value as i8
    }
}
impl From<bool> for i32 {
    fn from(value: bool) -> i32 {
        value as i32
    }
}
impl From<bool> for i64 {
    fn from(value: bool) -> i64 {
        value as i64
    }
}
impl From<bool> for Field {
    fn from(value: bool) -> Field {
        value as Field
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/convert.nr#L28-L119" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/convert.nr#L28-L119</a></sub></sup>


#### When to implement `From`

As a general rule of thumb, `From` may be implemented in the [situations where it would be suitable in Rust](https://doc.rust-lang.org/std/convert/trait.From.html#when-to-implement-from):

- The conversion is *infallible*: Noir does not provide an equivalent to Rust's `TryFrom`, if the conversion can fail then provide a named method instead.
- The conversion is *lossless*: semantically, it should not lose or discard information. For example, `u32: From<u16>` can losslessly convert any `u16` into a valid `u32` such that the original `u16` can be recovered. On the other hand, `u16: From<u32>` should not be implemented as `2**16` is a `u32` which cannot be losslessly converted into a `u16`.
- The conversion is *value-preserving*: the conceptual kind and meaning of the resulting value is the same, even though the Noir type and technical representation might be different. While it's possible to infallibly and losslessly convert a `u8` into a `str<2>` hex representation, `4u8` and `"04"` are too different for `str<2>: From<u8>` to be implemented.
- The conversion is *obvious*: it's the only reasonable conversion between the two types. If there's ambiguity on how to convert between them such that the same input could potentially map to two different values then a named method should be used. For instance rather than implementing `U128: From<[u8; 16]>`, the methods `U128::from_le_bytes` and `U128::from_be_bytes` are used as otherwise the endianness of the array would be ambiguous, resulting in two potential values of `U128` from the same byte array.

One additional recommendation specific to Noir is:
- The conversion is *efficient*: it's relatively cheap to convert between the two types. Due to being a ZK DSL, it's more important to avoid unnecessary computation compared to Rust. If the implementation of `From` would encourage users to perform unnecessary conversion, resulting in additional proving time, then it may be preferable to expose functionality such that this conversion may be avoided.

### `std::convert::Into`

The `Into` trait is defined as the reciprocal of `From`. It should be easy to convince yourself that if we can convert to type `A` from type `B`, then it's possible to convert type `B` into type `A`.

For this reason, implementing `From` on a type will automatically generate a matching `Into` implementation. One should always prefer implementing `From` over `Into` as implementing `Into` will not generate a matching `From` implementation.

```rust title="into-trait" showLineNumbers 
pub trait Into<T> {
    fn into(self) -> T;
}

impl<T, U> Into<T> for U
where
    T: From<U>,
{
    fn into(self) -> T {
        T::from(self)
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/convert.nr#L13-L26" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/convert.nr#L13-L26</a></sub></sup>


`Into` is most useful when passing function arguments where the types don't quite match up with what the function expects. In this case, the compiler has enough type information to perform the necessary conversion by just appending `.into()` onto the arguments in question.

---

## `std::cmp`

### `std::cmp::Eq`

```rust title="eq-trait" showLineNumbers 
pub trait Eq {
    fn eq(self, other: Self) -> bool;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/cmp.nr#L4-L8" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/cmp.nr#L4-L8</a></sub></sup>


Returns `true` if `self` is equal to `other`. Implementing this trait on a type
allows the type to be used with `==` and `!=`.

Implementations:
```rust
impl Eq for Field { .. }

impl Eq for i8 { .. }
impl Eq for i16 { .. }
impl Eq for i32 { .. }
impl Eq for i64 { .. }

impl Eq for u8 { .. }
impl Eq for u16 { .. }
impl Eq for u32 { .. }
impl Eq for u64 { .. }

impl Eq for () { .. }
impl Eq for bool { .. }

impl<T, N> Eq for [T; N]
    where T: Eq { .. }

impl<T> Eq for [T]
    where T: Eq { .. }

impl<A, B> Eq for (A, B)
    where A: Eq, B: Eq { .. }

impl<A, B, C> Eq for (A, B, C)
    where A: Eq, B: Eq, C: Eq { .. }

impl<A, B, C, D> Eq for (A, B, C, D)
    where A: Eq, B: Eq, C: Eq, D: Eq { .. }

impl<A, B, C, D, E> Eq for (A, B, C, D, E)
    where A: Eq, B: Eq, C: Eq, D: Eq, E: Eq { .. }
```

### `std::cmp::Ord`

```rust title="ord-trait" showLineNumbers 
pub trait Ord {
    fn cmp(self, other: Self) -> Ordering;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/cmp.nr#L210-L214" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/cmp.nr#L210-L214</a></sub></sup>


`a.cmp(b)` compares two values returning `Ordering::less()` if `a < b`,
`Ordering::equal()` if `a == b`, or `Ordering::greater()` if `a > b`.
Implementing this trait on a type allows `<`, `<=`, `>`, and `>=` to be
used on values of the type.

`std::cmp` also provides `max` and `min` functions for any type which implements the `Ord` trait.

Implementations:

```rust
impl Ord for u8 { .. }
impl Ord for u16 { .. }
impl Ord for u32 { .. }
impl Ord for u64 { .. }

impl Ord for i8 { .. }
impl Ord for i16 { .. }
impl Ord for i32 { .. }

impl Ord for i64 { .. }

impl Ord for () { .. }
impl Ord for bool { .. }

impl<T, N> Ord for [T; N]
    where T: Ord { .. }

impl<T> Ord for [T]
    where T: Ord { .. }

impl<A, B> Ord for (A, B)
    where A: Ord, B: Ord { .. }

impl<A, B, C> Ord for (A, B, C)
    where A: Ord, B: Ord, C: Ord { .. }

impl<A, B, C, D> Ord for (A, B, C, D)
    where A: Ord, B: Ord, C: Ord, D: Ord { .. }

impl<A, B, C, D, E> Ord for (A, B, C, D, E)
    where A: Ord, B: Ord, C: Ord, D: Ord, E: Ord { .. }
```

---

## `std::ops`

### `std::ops::Add`, `std::ops::Sub`, `std::ops::Mul`, and `std::ops::Div`

These traits abstract over addition, subtraction, multiplication, and division respectively.
Implementing these traits for a given type will also allow that type to be used with the corresponding operator
for that trait (`+` for Add, etc) in addition to the normal method names.

```rust title="add-trait" showLineNumbers 
pub trait Add {
    fn add(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L1-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L1-L5</a></sub></sup>

```rust title="sub-trait" showLineNumbers 
pub trait Sub {
    fn sub(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L60-L64" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L60-L64</a></sub></sup>

```rust title="mul-trait" showLineNumbers 
pub trait Mul {
    fn mul(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L119-L123" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L119-L123</a></sub></sup>

```rust title="div-trait" showLineNumbers 
pub trait Div {
    fn div(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L178-L182" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L178-L182</a></sub></sup>


The implementations block below is given for the `Add` trait, but the same types that implement
`Add` also implement `Sub`, `Mul`, and `Div`.

Implementations:
```rust
impl Add for Field { .. }

impl Add for i8 { .. }
impl Add for i16 { .. }
impl Add for i32 { .. }
impl Add for i64 { .. }

impl Add for u8 { .. }
impl Add for u16 { .. }
impl Add for u32 { .. }
impl Add for u64 { .. }
```

### `std::ops::Rem`

```rust title="rem-trait" showLineNumbers 
pub trait Rem {
    fn rem(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L237-L241" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L237-L241</a></sub></sup>


`Rem::rem(a, b)` is the remainder function returning the result of what is
left after dividing `a` and `b`. Implementing `Rem` allows the `%` operator
to be used with the implementation type.

Unlike other numeric traits, `Rem` is not implemented for `Field`.

Implementations:
```rust
impl Rem for u8 { fn rem(self, other: u8) -> u8 { self % other } }
impl Rem for u16 { fn rem(self, other: u16) -> u16 { self % other } }
impl Rem for u32 { fn rem(self, other: u32) -> u32 { self % other } }
impl Rem for u64 { fn rem(self, other: u64) -> u64 { self % other } }

impl Rem for i8 { fn rem(self, other: i8) -> i8 { self % other } }
impl Rem for i16 { fn rem(self, other: i16) -> i16 { self % other } }
impl Rem for i32 { fn rem(self, other: i32) -> i32 { self % other } }
impl Rem for i64 { fn rem(self, other: i64) -> i64 { self % other } }
```

### `std::ops::Neg`

```rust title="neg-trait" showLineNumbers 
pub trait Neg {
    fn neg(self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L290-L294" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L290-L294</a></sub></sup>


`Neg::neg` is equivalent to the unary negation operator `-`.

Implementations:
```rust title="neg-trait-impls" showLineNumbers 
impl Neg for Field {
    fn neg(self) -> Field {
        -self
    }
}

impl Neg for i8 {
    fn neg(self) -> i8 {
        -self
    }
}
impl Neg for i16 {
    fn neg(self) -> i16 {
        -self
    }
}
impl Neg for i32 {
    fn neg(self) -> i32 {
        -self
    }
}
impl Neg for i64 {
    fn neg(self) -> i64 {
        -self
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/arith.nr#L296-L323" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/arith.nr#L296-L323</a></sub></sup>


### `std::ops::Not`

```rust title="not-trait" showLineNumbers 
pub trait Not {
    fn not(self: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L1-L5" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L1-L5</a></sub></sup>


`Not::not` is equivalent to the unary bitwise NOT operator `!`.

Implementations:
```rust title="not-trait-impls" showLineNumbers 
impl Not for bool {
    fn not(self) -> bool {
        !self
    }
}

impl Not for u64 {
    fn not(self) -> u64 {
        !self
    }
}
impl Not for u32 {
    fn not(self) -> u32 {
        !self
    }
}
impl Not for u16 {
    fn not(self) -> u16 {
        !self
    }
}
impl Not for u8 {
    fn not(self) -> u8 {
        !self
    }
}
impl Not for u1 {
    fn not(self) -> u1 {
        !self
    }
}

impl Not for i8 {
    fn not(self) -> i8 {
        !self
    }
}
impl Not for i16 {
    fn not(self) -> i16 {
        !self
    }
}
impl Not for i32 {
    fn not(self) -> i32 {
        !self
    }
}
impl Not for i64 {
    fn not(self) -> i64 {
        !self
    }
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L7-L60" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L7-L60</a></sub></sup>


### `std::ops::{ BitOr, BitAnd, BitXor }`

```rust title="bitor-trait" showLineNumbers 
pub trait BitOr {
    fn bitor(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L62-L66" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L62-L66</a></sub></sup>

```rust title="bitand-trait" showLineNumbers 
pub trait BitAnd {
    fn bitand(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L121-L125" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L121-L125</a></sub></sup>

```rust title="bitxor-trait" showLineNumbers 
pub trait BitXor {
    fn bitxor(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L180-L184" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L180-L184</a></sub></sup>


Traits for the bitwise operations `|`, `&`, and `^`.

Implementing `BitOr`, `BitAnd` or `BitXor` for a type allows the `|`, `&`, or `^` operator respectively
to be used with the type.

The implementations block below is given for the `BitOr` trait, but the same types that implement
`BitOr` also implement `BitAnd` and `BitXor`.

Implementations:
```rust
impl BitOr for bool { fn bitor(self, other: bool) -> bool { self | other } }

impl BitOr for u8 { fn bitor(self, other: u8) -> u8 { self | other } }
impl BitOr for u16 { fn bitor(self, other: u16) -> u16 { self | other } }
impl BitOr for u32 { fn bitor(self, other: u32) -> u32 { self | other } }
impl BitOr for u64 { fn bitor(self, other: u64) -> u64 { self | other } }

impl BitOr for i8 { fn bitor(self, other: i8) -> i8 { self | other } }
impl BitOr for i16 { fn bitor(self, other: i16) -> i16 { self | other } }
impl BitOr for i32 { fn bitor(self, other: i32) -> i32 { self | other } }
impl BitOr for i64 { fn bitor(self, other: i64) -> i64 { self | other } }
```

### `std::ops::{ Shl, Shr }`

```rust title="shl-trait" showLineNumbers 
pub trait Shl {
    fn shl(self, other: u8) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L239-L243" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L239-L243</a></sub></sup>

```rust title="shr-trait" showLineNumbers 
pub trait Shr {
    fn shr(self, other: u8) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/ops/bit.nr#L292-L296" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/ops/bit.nr#L292-L296</a></sub></sup>


Traits for a bit shift left and bit shift right.

Implementing `Shl` for a type allows the left shift operator (`<<`) to be used with the implementation type.
Similarly, implementing `Shr` allows the right shift operator (`>>`) to be used with the type.

Note that bit shifting is not currently implemented for signed types.

The implementations block below is given for the `Shl` trait, but the same types that implement
`Shl` also implement `Shr`.

Implementations:
```rust
impl Shl for u8 { fn shl(self, other: u8) -> u8 { self << other } }
impl Shl for u16 { fn shl(self, other: u16) -> u16 { self << other } }
impl Shl for u32 { fn shl(self, other: u32) -> u32 { self << other } }
impl Shl for u64 { fn shl(self, other: u64) -> u64 { self << other } }
```

---

## `std::append`

### `std::append::Append`

`Append` can abstract over types that can be appended to - usually container types:

```rust title="append-trait" showLineNumbers 
pub trait Append {
    fn empty() -> Self;
    fn append(self, other: Self) -> Self;
}
```
> <sup><sub><a href="https://github.com/noir-lang/noir/blob/master/noir_stdlib/src/append.nr#L9-L14" target="_blank" rel="noopener noreferrer">Source code: noir_stdlib/src/append.nr#L9-L14</a></sub></sup>


`Append` requires two methods:

- `empty`: Constructs an empty value of `Self`.
- `append`: Append two values together, returning the result.

Additionally, it is expected that for any implementation:

- `T::empty().append(x) == x`
- `x.append(T::empty()) == x`

Implementations:
```rust
impl<T> Append for [T]
impl Append for Quoted
```
