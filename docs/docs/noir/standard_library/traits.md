---
title: Traits
description: Noir's stdlib provides a few commonly used traits.
keywords: [traits, trait, interface, protocol, default, add, eq]
---

## `std::default`

### `std::default::Default`

#include_code default-trait noir_stdlib/src/default.nr rust

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
types such as arrays are filled with default values of their element type.


## `std::convert`

### `std::convert::From`

#include_code from-trait noir_stdlib/src/convert.nr rust

The `From` trait defines how to convert from a given type `T` to the type on which the trait is implemented.

The Noir standard library provides a number of implementations of `From` between primitive types.
#include_code from-impls noir_stdlib/src/convert.nr rust

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

#include_code into-trait noir_stdlib/src/convert.nr rust

`Into` is most useful when passing function arguments where the types don't quite match up with what the function expects. In this case, the compiler has enough type information to perform the necessary conversion by just appending `.into()` onto the arguments in question.


## `std::cmp`

### `std::cmp::Eq`

#include_code eq-trait noir_stdlib/src/cmp.nr rust

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

#include_code ord-trait noir_stdlib/src/cmp.nr rust

`a.cmp(b)` compares two values returning `Ordering::less()` if `a < b`,
`Ordering::equal()` if `a == b`, or `Ordering::greater()` if `a > b`.
Implementing this trait on a type allows `<`, `<=`, `>`, and `>=` to be
used on values of the type.

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

impl<A, B> Ord for (A, B)
    where A: Ord, B: Ord { .. }

impl<A, B, C> Ord for (A, B, C)
    where A: Ord, B: Ord, C: Ord { .. }

impl<A, B, C, D> Ord for (A, B, C, D)
    where A: Ord, B: Ord, C: Ord, D: Ord { .. }

impl<A, B, C, D, E> Ord for (A, B, C, D, E)
    where A: Ord, B: Ord, C: Ord, D: Ord, E: Ord { .. }
```

## `std::ops`

### `std::ops::Add`, `std::ops::Sub`, `std::ops::Mul`, and `std::ops::Div`

These traits abstract over addition, subtraction, multiplication, and division respectively.
Implementing these traits for a given type will also allow that type to be used with the corresponding operator
for that trait (`+` for Add, etc) in addition to the normal method names.

#include_code add-trait noir_stdlib/src/ops.nr rust
#include_code sub-trait noir_stdlib/src/ops.nr rust
#include_code mul-trait noir_stdlib/src/ops.nr rust
#include_code div-trait noir_stdlib/src/ops.nr rust

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

#include_code rem-trait noir_stdlib/src/ops.nr rust

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

### `std::ops::{ BitOr, BitAnd, BitXor }`

#include_code bitor-trait noir_stdlib/src/ops.nr rust
#include_code bitand-trait noir_stdlib/src/ops.nr rust
#include_code bitxor-trait noir_stdlib/src/ops.nr rust

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

#include_code shl-trait noir_stdlib/src/ops.nr rust
#include_code shr-trait noir_stdlib/src/ops.nr rust

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
