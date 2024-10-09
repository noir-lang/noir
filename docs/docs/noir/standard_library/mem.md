---
title: Memory Module
description:
  This module contains functions which manipulate memory in a low-level way
keywords:
  [
    mem, memory, zeroed, transmute, checked_transmute
  ]
---

# `std::mem::zeroed`

```rust
fn zeroed<T>() -> T
```

Returns a zeroed value of any type.
This function is generally unsafe to use as the zeroed bit pattern is not guaranteed to be valid for all types.
It can however, be useful in cases when the value is guaranteed not to be used such as in a BoundedVec library implementing a growable vector, up to a certain length, backed by an array.
The array can be initialized with zeroed values which are guaranteed to be inaccessible until the vector is pushed to.
Similarly, enumerations in noir can be implemented using this method by providing zeroed values for the unused variants.

This function currently supports the following types:

- Field
- Bool
- Uint
- Array
- Slice
- String
- Tuple
- Functions
  
Using it on other types could result in unexpected behavior.

# `std::mem::checked_transmute`

```rust
fn checked_transmute<T, U>(value: T) -> U
```

Transmutes a value of one type into the same value but with a new type `U`.

This function is safe to use since both types are asserted to be equal later during compilation after the concrete values for generic types become known.
This function is useful for cases where the compiler may fails a type check that is expected to pass where
a user knows the two types to be equal. For example, when using arithmetic generics there are cases the compiler
does not see as equal, such as `[Field; N*(A + B)]` and `[Field; N*A + N*B]`, which users may know to be equal.
In these cases, `checked_transmute` can be used to cast the value to the desired type while also preserving safety
by checking this equality once `N`, `A`, `B` are fully resolved.

Note that since this safety check is performed after type checking rather than during, no error is issued if the function
containing `checked_transmute` is never called.
