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

## `std::ops`

### `std::ops::Eq`

```rust
trait Eq {
    fn eq(self, other: Self) -> bool;
}
```
Returns `true` if `self` is equal to `other`.

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

### `std::ops::Add`, `std::ops::Sub`, `std::ops::Mul`, and `std::ops::Div`

These traits abstract over addition, subtraction, multiplication, and division respectively.
Although Noir does not currently have operator overloading, in the future implementing these
traits for a given type will also allow that type to be used with the corresponding operator
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
