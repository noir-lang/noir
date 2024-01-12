---
title: Traits
description: Noir's stdlib provides a few commonly used traits.
keywords: [traits, trait, interface, protocol, default, add, eq]
---

## `std::default`

### `std::default::Default`

```rust
trait Default {
    fn default() -> Self;
}
```

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

## `std::cmp`

### `std::cmp::Eq`

```rust
trait Eq {
    fn eq(self, other: Self) -> bool;
}
```
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

### `std::cmp::Cmp`

```rust
trait Cmp {
    fn cmp(self, other: Self) -> Ordering;
}
```

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

```rust
trait Add {
    fn add(self, other: Self) -> Self;
}

trait Sub {
    fn sub(self, other: Self) -> Self;
}

trait Mul {
    fn mul(self, other: Self) -> Self;
}

trait Div {
    fn div(self, other: Self) -> Self;
}
```

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

```rust
trait Rem {
    fn rem(self, other: Self) -> Self;
}
```

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

```rust
trait BitOr {
    fn bitor(self, other: Self) -> Self;
}

trait BitAnd {
    fn bitand(self, other: Self) -> Self;
}

trait BitXor {
    fn bitxor(self, other: Self) -> Self;
}
```

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

```rust
trait Shl {
    fn shl(self, other: Self) -> Self;
}

trait Shr {
    fn shr(self, other: Self) -> Self;
}
```

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
